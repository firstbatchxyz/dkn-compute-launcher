use libsecp256k1::{Message, SecretKey};
use sha2::{Digest, Sha256};

// const REFERRALS_API_BASE_URL: &str = "https://dkn.dria.co/referral/v0/";
const REFERRALS_API_BASE_URL: &str = "http://localhost:8080/referral/v0";

/// Simple healthcheck for the referrals API.
pub async fn healthcheck() -> bool {
    reqwest::get(format!("{}/health", REFERRALS_API_BASE_URL))
        .await
        .is_ok_and(|r| r.status().is_success())
}

/// Returns a list of addresses of the users referred by the given `address`.
pub async fn get_referrals(address: &str) -> Result<Option<Vec<String>>, reqwest::Error> {
    let response = reqwest::get(format!(
        "{}/get_referrals/{}",
        REFERRALS_API_BASE_URL, address
    ))
    .await?;

    if response.status().is_client_error() {
        return Ok(None);
    } else {
        response.json().await.map(Some)
    }
}

// get the user that referred you
pub async fn get_referred_by(address: &str) -> Result<Option<String>, reqwest::Error> {
    let response = reqwest::get(format!(
        "{}/get_referred_by/{}",
        REFERRALS_API_BASE_URL, address
    ))
    .await
    .unwrap();

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Res {
        referrer_address: String,
    }

    if response.status().is_client_error() {
        return Ok(None);
    } else {
        response
            .json::<Res>()
            .await
            .map(|r| Some(r.referrer_address))
    }
}

/// Requests a challenge from the referral API, and completes it to get a referral code.
///
/// If a code has been generated before, this will return the same code.
pub async fn get_referral_code(
    secret_key: &SecretKey,
    address: &str,
) -> Result<String, reqwest::Error> {
    log::debug!("Getting referral code for {}", address);
    // alice gets a challenge
    let req = reqwest::Client::new()
        .post(format!("{}/get_challenge", REFERRALS_API_BASE_URL))
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "address": address
            })
            .to_string(),
        )
        .send()
        .await?
        .error_for_status()?;
    let challenge = req.text().await?;
    log::debug!("Got challenge: {}", challenge);

    // alice signs the challenge and calls `get_code`
    let digest = Message::parse(&Sha256::digest(&challenge).into());
    let (sig, rec_id) = libsecp256k1::sign(&digest, &secret_key);
    let req = reqwest::Client::new()
        .post(format!("{}/get_code", REFERRALS_API_BASE_URL))
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "challenge": challenge,
                "sig": {
                    "signature": hex::encode(sig.serialize()),
                    "recoveryId": rec_id.serialize(),
                },
            })
            .to_string(),
        )
        .send()
        .await?
        .error_for_status()?;
    let code = req.text().await?;

    Ok(code)
}

/// Signs a code with the user's wallet secret key and sends it to the referral API.
pub async fn enter_referral_code(secret_key: &SecretKey, code: &str) -> Result<(), reqwest::Error> {
    let digest = Message::parse(&Sha256::digest(code).into());
    let (sig, rec_id) = libsecp256k1::sign(&digest, &secret_key);

    reqwest::Client::new()
        .post(format!("{}/refer", REFERRALS_API_BASE_URL))
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "code": code,
                "sig": {
                    "recoveryId": rec_id.serialize(),
                    "signature": hex::encode(sig.serialize()),
                },
            })
            .to_string(),
        )
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health() {
        let response = reqwest::get(format!("{}/health", REFERRALS_API_BASE_URL))
            .await
            .unwrap();
        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn test_get_referrals() {
        let response = get_referrals("3b64855e6f0cacca01089387c628e6540619ce07")
            .await
            .unwrap();
        // TODO: !!!
    }

    #[tokio::test]
    async fn test_get_referred_by() {
        let response = get_referred_by("3b64855e6f0cacca01089387c628e6540619ce07")
            .await
            .unwrap();
        // TODO: !!!
    }
}
