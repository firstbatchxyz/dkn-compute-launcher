use eyre::Result;
use std::path::Path;

use crate::{settings, utils::DriaEnv};

/// Asks for the following information for the user environment:
///
/// 1. Secret key (Wallet)
/// 2. Models
/// 3. Optional API Keys for Jina and Serper
///
/// ### Arguments
/// - `env_path`: path to the environment file
///
/// ### Errors
/// - If the environment file is not a file
pub fn setup_environment(env_path: &Path) -> Result<()> {
    let mut dria_env = DriaEnv::new_from_env();

    // ask for a wallet
    log::info!("Provide a secret key of your wallet.");
    settings::edit_wallet(&mut dria_env, false)?;

    // ask for models
    log::info!("Choose models that you would like to run.");
    settings::edit_models(&mut dria_env)?;

    // create directories if they dont exist
    DriaEnv::new_default_file(env_path)?;

    // then overwrite it with the new values
    dria_env.save_to_file(env_path)?;

    Ok(())
}
