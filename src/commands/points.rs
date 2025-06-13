use colored::Colorize;
use eyre::Context;

use crate::utils::{get_network_env, DriaEnv, LAUNCHER_USER_AGENT};

#[inline]
fn get_points_api_url(address: &str) -> String {
    let network = get_network_env();
    let address = address.trim_start_matches("0x");
    format!("https://{network}.dkn.dria.co/points/v0/total/node/0x{address}")
}

#[derive(Debug, serde::Deserialize)]
pub struct PointsRes {
    /// Indicates in which top percentile your points are.
    pub percentile: usize,
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

    let points = get_points(&address)
        .await
        .wrap_err("could not get points")?;

    if points.score == 0.0 {
        eprintln!(
            "You have not accumulated any {} yet.",
            "$DRIA points".purple()
        );
    } else {
        eprintln!(
            "You have accumulated {} {}, which puts you in the top {}%.",
            points.score,
            "$DRIA points".purple(),
            points.percentile
        );
    }

    Ok(())
}

async fn get_points(address: &str) -> eyre::Result<PointsRes> {
    let url = get_points_api_url(address);

    let client = reqwest::Client::builder()
        .user_agent(LAUNCHER_USER_AGENT)
        .build()
        .wrap_err("could not create reqwest client")?;

    let res = client
        .get(&url)
        .send()
        .await
        .wrap_err("could not make request")?;

    let points = res
        .json::<PointsRes>()
        .await
        .wrap_err("could not parse body")?;

    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_points() {
        let address = "0x1234567890abcdef1234567890abcdef12345678";
        let points = get_points(address).await.unwrap();
        assert!(points.score >= 0.0);
        assert!(points.percentile <= 100);
    }
}
