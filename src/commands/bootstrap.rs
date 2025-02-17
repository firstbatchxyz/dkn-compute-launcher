use eyre::Result;
use std::path::Path;

use crate::{
    settings::{self, DriaApiKeyKind},
    utils::DriaEnv,
};

/// Creates & bootstraps a new environment.
///
/// This also runs on first launch when an env file is not found.
pub fn bootstrap_env(env_path: &Path) -> Result<()> {
    std::fs::write(env_path, DriaEnv::EXAMPLE_ENV)?;

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
        log::info!("Provide an API key for: {}", api_key);
        let new_value = inquire::Text::new("Enter the new value:")
            .with_default(dria_env.get(api_key.name()).unwrap_or_default())
            .with_help_message("ESC to go back")
            .prompt()?;
        dria_env.set(api_key.name(), new_value);
    }

    dria_env.save_to_file(env_path)?;

    Ok(())
}
