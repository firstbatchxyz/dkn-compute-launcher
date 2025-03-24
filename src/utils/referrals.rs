use eyre::{eyre, Context, Result};
use libsecp256k1::SecretKey;
use reqwest::Client;

use crate::utils::crypto::eip191_hash;

const REFERRALS_API_BASE_URL: &str = "https://dkn.dria.co/referral/v0";
// const REFERRALS_API_BASE_URL: &str = "http://localhost:8080/referral/v0";

pub struct ReferralsClient {
    base_url: String,
    client: reqwest::Client,
}

impl Default for ReferralsClient {
    fn default() -> Self {
        Self::new(REFERRALS_API_BASE_URL)
    }
}

impl ReferralsClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    /// Simple healthcheck for the referrals API.
    pub async fn healthcheck(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }

    /// Returns a list of addresses of the users referred by the given `address`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// if let Some(result) = ReferralsClient::default().get_referrals(your_addr).await? {
    ///    println!("You have referred the following users:\n{}", result.join("\n"));
    /// } else {
    ///   println!("You have not referred anyone yet.");
    /// }
    /// ```
    pub async fn get_referrals(&self, address: &str) -> Result<Option<Vec<String>>> {
        let res = self
            .client
            .get(format!("{}/get_referrals/{}", self.base_url, address))
            .send()
            .await?;

        if res.status().is_client_error() {
            Ok(None)
        } else {
            Ok(res.json().await.map(Some)?)
        }
    }

    /// Returns the user that referred the given `address`.
    ///
    /// ### Example
    ///
    /// ```rust
    /// if let Some(result) = ReferralsClient::default().get_referred_by(your_addr).await? {
    ///     println!("You are referred by: {}", result);
    /// } else {
    ///    println!("You are not referred by anyone.");
    /// }
    /// ```
    pub async fn get_referred_by(&self, address: &str) -> Result<Option<String>, reqwest::Error> {
        let res = self
            .client
            .get(format!("{}/get_referred_by/{}", self.base_url, address))
            .send()
            .await?;

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Res {
            referrer_address: String,
        }

        if res.status().is_client_error() {
            Ok(None)
        } else {
            res.json::<Res>().await.map(|r| Some(r.referrer_address))
        }
    }

    /// Returns the number of referral code uses for this address.
    pub async fn get_max_uses(&self, address: &str) -> Result<usize> {
        let res = self
            .client
            .get(format!("{}/get_max_uses/{}", self.base_url, address))
            .send()
            .await?
            .error_for_status()?;

        res.text()
            .await?
            .parse()
            .wrap_err("could not parse returned value")
    }
    /// Requests a challenge from the referral API, and completes it to get a referral code.
    pub async fn get_referral_code(&self, secret_key: &SecretKey, address: &str) -> Result<String> {
        log::debug!("Getting referral code for {}", address);
        let res = self
            .client
            .post(format!("{}/get_challenge", self.base_url))
            .header("Content-Type", "application/json")
            .body(
                serde_json::json!({
                  "address": address
                })
                .to_string(),
            )
            .send()
            .await?;
        let challenge = if res.status().is_success() {
            res.text().await?
        } else {
            return Err(eyre!("Failed to get challenge: {}", res.text().await?));
        };
        log::debug!("Got challenge: {}", challenge);

        // alice signs the challenge and calls `get_code`
        let digest = eip191_hash(&challenge);
        let (sig, rec_id) = libsecp256k1::sign(&digest, secret_key);
        let res = self
            .client
            .post(format!("{}/get_code", self.base_url))
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
            .await?;
        let code = if res.status().is_success() {
            res.text().await?
        } else {
            return Err(eyre!("Failed to get code: {}", res.text().await?));
        };

        Ok(code)
    }

    /// Signs a code with the user's wallet secret key and sends it to the referral API.
    pub async fn enter_referral_code(&self, secret_key: &SecretKey, code: &str) -> Result<()> {
        let digest = eip191_hash(code);
        let (sig, rec_id) = libsecp256k1::sign(&digest, secret_key);

        let res = self
            .client
            .post(format!("{}/refer", self.base_url))
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
            .await?;
        if res.status().is_success() {
            log::info!("Successfully entered referral code");
        } else {
            return Err(eyre!(
                "Failed to enter referral code: {}",
                res.text().await?
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health() {
        let ok = ReferralsClient::default().healthcheck().await;
        assert!(ok);
    }

    #[tokio::test]
    async fn test_get_referrals() {
        let _response = ReferralsClient::default()
            .get_referrals("3b64855e6f0cacca01089387c628e6540619ce07")
            .await
            .unwrap();
        // TODO: !!!
    }

    #[tokio::test]
    async fn test_get_referred_by() {
        let _response = ReferralsClient::default()
            .get_referred_by("3b64855e6f0cacca01089387c628e6540619ce07")
            .await
            .unwrap();
        // TODO: !!!
    }
}
