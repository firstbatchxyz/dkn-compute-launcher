use eyre::Context;
use serde::Deserialize;

use crate::utils::DriaEnv;

const POINTS_API_BASE_URL: &str = "https://dkn.dria.co/dashboard/supply/v0/leaderboard/steps";

#[derive(Debug, Deserialize)]
pub struct PointsRes {
    #[serde(deserialize_with = "deserialize_percentile")]
    /// Indicates in which top percentile your points are.
    ///
    /// It is serialized as stringified number in the API response, due to frontend issues.
    pub percentile: u64,
    /// The total number of points you have accumulated.
    pub score: f64,
}

// the API returns a stringified number due to frontend issues, so we need to parse it
fn deserialize_percentile<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let parsed = s.parse().map_err(serde::de::Error::custom)?;

    if parsed > 100 {
        return Err(serde::de::Error::custom(
            "percentile must be between 0 and 100",
        ));
    }

    Ok(parsed)
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
