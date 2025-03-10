use colored::Colorize;
use eyre::eyre;
use inquire::{Select, Text};

use crate::utils::{crypto::*, referrals::*, DriaEnv, Selectable};

const MAX_USES: usize = 5;

/// Referrals-related commands.
///
/// If you are referred by a user, it is shown on the logs. Otherwise, a command is shown.
/// If you have referred users, they are shown on the logs. Otherwise, a command is shown.
pub async fn handle_referrals() -> eyre::Result<()> {
    let client = ReferralsClient::default();

    // ensure system is healthy
    if !client.healthcheck().await {
        return Err(eyre!("Referrals API is offline."));
    }

    // get wallet secret from env
    let dria_env = DriaEnv::new_from_env();
    let Some(secret_key) = dria_env.get("DKN_WALLET_SECRET_KEY") else {
        return Err(eyre!("No wallet secret key found."));
    };

    // convert to address
    let (sk, _, addr) = parse_key_to_account(secret_key)?;

    loop {
        let Selectable::Some(choice) = Select::new(
            "Choose a command below:",
            Selectable::new(vec![
                ReferralCommands::GetReferralCode,
                ReferralCommands::EnterReferralCode,
                ReferralCommands::ShowReferrals,
                ReferralCommands::ShowReferredBy,
            ]),
        )
        .with_help_message("↑↓ to move, ENTER to select")
        .prompt()?
        else {
            break;
        };

        match choice {
            ReferralCommands::GetReferralCode => {
                // get the users that you have referred
                let referrals = client.get_referrals(&addr).await?.unwrap_or_default();
                if !referrals.is_empty() {
                    log::info!(
                        "You have referred the following users:\n{} ({}/{})",
                        referrals.join("\n"),
                        referrals.len(),
                        MAX_USES
                    );
                } else {
                    log::info!("You have not referred anyone yet.");
                }

                // get the referral code
                let code = client.get_referral_code(&sk, &addr).await?;
                log::info!("Your referral code is: {}", code.bold().blue());

                if referrals.len() >= MAX_USES {
                    log::warn!("You have reached the maximum number of referrals!");
                }
            }
            ReferralCommands::EnterReferralCode => {
                // get the user that referred you
                if let Some(referred_by) = client.get_referred_by(&addr).await? {
                    log::info!("You are already referred by 0x{}", referred_by);
                } else {
                    let code = Text::new("Enter the referral code:")
                        .with_validator(|code: &str| {
                            // code length here is hardcoded w.r.t referrals API
                            if code.len() == 20 {
                                Ok(inquire::validator::Validation::Valid)
                            } else {
                                Ok(inquire::validator::Validation::Invalid(
                                    "The referral code must be 20 characters long.".into(),
                                ))
                            }
                        })
                        .prompt()?;
                    client.enter_referral_code(&sk, &code).await?;
                }
            }
            ReferralCommands::ShowReferrals => {
                let referrals = client.get_referrals(&addr).await?.unwrap_or_default();
                if !referrals.is_empty() {
                    log::info!(
                        "You have referred the following users:\n{}",
                        referrals.join("\n")
                    );
                } else {
                    log::info!("You have not referred anyone yet.");
                }
            }
            ReferralCommands::ShowReferredBy => {
                if let Some(referred_by) = client.get_referred_by(&addr).await? {
                    log::info!("You are referred by 0x{}", referred_by);
                } else {
                    log::info!("You are not referred by anyone.");
                }
            }
        }
    }

    Ok(())
}

enum ReferralCommands {
    GetReferralCode,
    EnterReferralCode,
    ShowReferrals,
    ShowReferredBy,
}

impl std::fmt::Display for ReferralCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetReferralCode => write!(f, "Get referral code to refer someone"),
            Self::EnterReferralCode => write!(f, "Enter referral code to be referred"),
            Self::ShowReferrals => write!(f, "List addresses referred by you"),
            Self::ShowReferredBy => write!(f, "Show the address that referred you"),
        }
    }
}
