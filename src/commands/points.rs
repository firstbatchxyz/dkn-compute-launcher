use eyre::Context;

use crate::utils::DriaEnv;

const POINTS_API_BASE_URL: &str =
    "https://mainnet.dkn.dria.co/dashboard/supply/v0/leaderboard/steps";

#[derive(Debug, serde::Deserialize)]
pub struct PointsRes {
    /// Indicates in which top percentile your points are.
    pub percentile: u32,
    /// The total number of points you have accumulated.
    pub score: f64,
}

/// Returns the $DRIA points for the users address.
///
/// - Will ask for user to enter their secret key if it is not set.
pub async fn show_points() -> eyre::Result<()> {
    let mut dria_env = DriaEnv::new_from_env();
    dria_env.ask_for_key_if_required()?;
    let (_, _, address) = dria_env.get_account()?;

    // the address can have 0x or not, we enforce it ourselves here
    let url = format!(
        "{}?address=0x{}",
        POINTS_API_BASE_URL,
        address.trim_start_matches("0x")
    );

    let points = reqwest::get(&url)
        .await
        .wrap_err("could not make request")?
        .json::<PointsRes>()
        .await
        .wrap_err("could not parse body")?;

    if points.score == 0.0 {
        eprintln!("You have not accumulated any $DRIA points yet.");
    } else {
        eprintln!(
            "You have accumulated {} $DRIA points, which puts you in the top {}%.",
            points.score, points.percentile
        );
    }

    Ok(())
}
