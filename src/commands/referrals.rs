use eyre::eyre;
use inquire::{Select, Text};

use crate::utils::{crypto::*, referrals::*, DriaEnv, Selectable};

/// Referrals-related commands.
///
/// If you are referred by a user, it is shown on the logs. Otherwise, a command is shown.
/// If you have referred users, they are shown on the logs. Otherwise, a command is shown.
pub async fn handle_referrals() -> eyre::Result<()> {
    // ensure system is healthy
    if !healthcheck().await {
        return Err(eyre!("Referrals API is offline."));
    }

    // get wallet secret from env
    let dria_env = DriaEnv::new_from_env();
    let Some(secret_key) = dria_env.get("DKN_WALLET_SECRET_KEY") else {
        return Err(eyre!("No wallet secret key found."));
    };

    // convert to address
    let (sk, _, addr) = parse_key_to_account(secret_key)?;

    let Selectable::Some(choice) = Select::new(
        "Choose a command below:",
        Selectable::new(vec![
            ReferralCommands::GetReferralCode,
            ReferralCommands::EnterReferralCode,
        ]),
    )
    .with_help_message("↑↓ to move, ENTER to select")
    .prompt()?
    else {
        return Ok(());
    };

    match choice {
        ReferralCommands::GetReferralCode => {
            // get the users that you have referred
            let referrals = get_referrals(&addr).await?;

            if let Some(referrals) = referrals {
                if !referrals.is_empty() {
                    log::info!(
                        "You have referred the following users:\n{}",
                        referrals.join("\n")
                    );
                } else {
                    log::info!("You have not referred anyone yet.");
                }
            }

            // get the referral code
            let code = get_referral_code(&sk, &addr).await?;
            log::info!("Your referral code is: {}", code);
        }
        ReferralCommands::EnterReferralCode => {
            // get the user that referred you
            if let Some(referred_by) = get_referred_by(&addr).await? {
                log::info!("You are already referred by 0x{}", referred_by);
            } else {
                let code = Text::new("Enter the referral code:").prompt()?;
                enter_referral_code(&sk, &code).await?;
            }
        }
    }

    Ok(())
}

enum ReferralCommands {
    GetReferralCode,
    EnterReferralCode,
}

impl std::fmt::Display for ReferralCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetReferralCode => write!(f, "Get Referral Code"),
            Self::EnterReferralCode => write!(f, "Enter Referral Code"),
        }
    }
}
