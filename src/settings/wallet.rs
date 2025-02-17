use inquire::{validator::Validation, Password, PasswordDisplayMode};

use crate::DriaEnv;

const WALLET_KEY: &str = "DKN_WALLET_SECRET_KEY";

pub fn edit_wallet(dria_env: &mut DriaEnv, skippable: bool) -> eyre::Result<()> {
    // masks a string "abcdefgh" to something like "ab****gh"
    let mask = |s: &str| {
        const LEFT: usize = 2;
        const RIGHT: usize = 2;

        if s.len() <= LEFT + RIGHT {
            s.to_string()
        } else {
            format!(
                "{}{}{}",
                &s[..LEFT],
                "*".repeat(s.len() - LEFT - RIGHT),
                &s[s.len() - RIGHT..]
            )
        }
    };

    // validates the secret key to be 64 characters hexadecimal, with or without 0x prefix
    let validator = |secret_key: &str| {
        if secret_key.trim_start_matches("0x").len() != 64 {
            Ok(Validation::Invalid(
                "Key must be exactly 64 characters hexadecimal, with or without 0x prefix.".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    };

    let Some(new_key) = Password::new("Enter wallet secret key:")
        .with_validator(validator)
        .with_display_mode(PasswordDisplayMode::Masked)
        .without_confirmation()
        .with_help_message(&format!(
            "ESC to go back and keep using {}",
            mask(dria_env.get(WALLET_KEY).unwrap_or("N/A"))
        ))
        .prompt_skippable()?
    else {
        return Ok(());
    };

    log::info!("New secret key: {}", mask(&new_key));
    dria_env.set(WALLET_KEY, new_key);

    Ok(())
}
