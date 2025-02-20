use eyre::Result;
use std::path::Path;

use crate::{
    settings::{self, DriaApiKeyKind},
    utils::DriaEnv,
};

/// Example env file content, used for creating a new env file.
const BASE_ENV_FILE_CONTENT: &str = include_str!("../../.env.example");

/// Creates & sets up a new environment. It specifically asks for the following:
///
/// 1. A wallet
/// 2. Models
/// 3. API Keys for respective model providers
/// 4. Optional API Keys for Jina and Serper
///
/// This also runs on first launch when an env file is not found.
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

    // ask for API keys w.r.t models
    let configured_providers = dria_env.get_model_config().get_providers();
    let required_api_keys = DriaApiKeyKind::from_providers(&configured_providers);
    for api_key in required_api_keys {
        log::info!("Provide {} because you are using its model", api_key);
        let new_value = api_key.prompt(&dria_env)?;
        dria_env.set(api_key.name(), new_value);
    }

    // ask for Jina and Serper api keys (optional)
    for optional_api_key in [DriaApiKeyKind::Jina, DriaApiKeyKind::Serper] {
        log::info!(
            "Optionally provide {} for better performance",
            optional_api_key
        );
        if let Some(new_value) = optional_api_key.prompt_skippable(&dria_env)? {
            dria_env.set(optional_api_key.name(), new_value);
        }
    }

    // first write the base environment
    std::fs::write(env_path, BASE_ENV_FILE_CONTENT)?;

    // then overwrite it with the new values
    dria_env.save_to_file(env_path)?;

    Ok(())
}
