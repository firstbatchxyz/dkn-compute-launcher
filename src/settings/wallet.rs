use inquire::{validator::Validation, Text};

use crate::DriaEnv;

const WALLET_KEY: &str = "DKN_WALLET_SECRET_KEY";

pub fn edit_wallet(dria_env: &mut DriaEnv, skippable: bool) -> eyre::Result<()> {
    let existing_secret_opt = dria_env.get(WALLET_KEY);

    // masks a string "abcdefgh" to something like "ab****gh"
    // also ignores the 0x at the start
    let mask = |s: &str| {
        const LEFT: usize = 2;
        const RIGHT: usize = 2;
        const MASK_CHAR: &str = "*";
        debug_assert!(MASK_CHAR.len() == 1);

        if s.len() <= LEFT + RIGHT {
            s.to_string()
        } else {
            format!(
                "{}{}{}",
                &s.trim_start_matches("0x")[..LEFT],
                MASK_CHAR.repeat(s.len() - LEFT - RIGHT),
                &s[s.len() - RIGHT..]
            )
        }
    };

    // validates the secret key to be 64 characters hexadecimal, with or without 0x prefix
    // empty string is ok, as it means the user wants to skip
    let validator = move |secret_key: &str| {
        if secret_key.trim_start_matches("0x").len() != 64 {
            if skippable & secret_key.is_empty() {
                // empty string is ok if skippable
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    "Key must be exactly 64 characters hexadecimal, with or without 0x prefix."
                        .into(),
                ))
            }
        } else {
            Ok(Validation::Valid)
        }
    };

    // custom confirmation message
    let formatter = |s: &str| mask(s);

    let new_secret = Text::new("Enter wallet secret key:")
        .with_validator(validator)
        .with_formatter(&formatter)
        .with_default(existing_secret_opt.unwrap_or_default())
        .with_help_message(&match existing_secret_opt {
            Some(secret) => format!("ENTER without typing to keep using {}", mask(secret)),
            None => "You can get your secret from a wallet like MetaMask.".to_string(),
        })
        .prompt()
        .map(|s| {
            // if the user skips by entering an empty input, we keep the existing secret key
            if s.is_empty() {
                existing_secret_opt.unwrap_or_default().to_string()
            } else {
                s
            }
        })?;

    if !new_secret.is_empty() {
        dria_env.set(WALLET_KEY, new_secret);
    }

    Ok(())
}
