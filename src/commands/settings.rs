use eyre::{eyre, Result};
use inquire::{Confirm, Select};
use std::path::Path;

use crate::{settings::*, DriaEnv};

/// Starts the interactive settings editor for the given environment.
///
/// ### Arguments
/// - `env_path`: path to the environment file
///
/// ### Errors
/// - If the environment file is not a file
pub async fn change_settings(env_path: &Path) -> Result<()> {
    if !env_path.exists() {
        return Err(eyre!(
            "Environment file does not exist: {}",
            env_path.display()
        ));
    }

    // an environment object is created from the existing environment variables
    let mut dria_env = DriaEnv::new_from_env();

    loop {
        // prompt the user for which setting to change
        let Some(choice) = Select::new(
            &format!("Choose settings (for {})", env_path.display()),
            Settings::all(),
        )
        .with_help_message("↑↓ to move, ENTER to select")
        .with_page_size(Settings::all().len())
        .prompt_skippable()?
        else {
            if dria_env.is_changed() {
                // continue the loop if user returns `false` from confirmation
                if let Some(false) =
                    Confirm::new("You have unsaved changes, are you sure you want to quit (y/n)?")
                        .with_help_message("You will lose all unsaved changes!")
                        .prompt_skippable()?
                {
                    log::info!("Exiting, changes are reverted.");
                    continue;
                }
            } else {
                log::info!("Exiting without changes.");
            }

            break;
        };

        match choice {
            Settings::Wallet => {
                crate::settings::edit_wallet(&mut dria_env, true)?;
            }
            Settings::Port => {
                crate::settings::edit_port(&mut dria_env)?;
            }
            Settings::Models => {
                crate::settings::show_model_settings_menu(&mut dria_env).await?;
            }
            Settings::Ollama => {
                crate::settings::edit_ollama(&mut dria_env)?;
            }
            Settings::ApiKeys => {
                crate::settings::edit_api_keys(&mut dria_env)?;
            }
            Settings::LogLevels => {
                crate::settings::edit_log_level(&mut dria_env)?;
            }
            Settings::SaveExit => {
                if dria_env.is_changed() {
                    dria_env.save_to_file(env_path)?;
                } else {
                    log::info!("No changes made.");
                }
                break;
            }
            Settings::Abort => {
                log::info!("Aborting changes.");
                break;
            }
        }
    }

    Ok(())
}
