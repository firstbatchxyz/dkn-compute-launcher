use dkn_executor::ollama_rs::Ollama;
use inquire::MultiSelect;

use crate::{utils::check_ollama, DriaEnv};

/// Remove local models (same as `ollama rm`).
pub async fn remove_local_models(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    // ensure Ollama is available
    if !check_ollama(dria_env).await {
        eyre::bail!("Ollama is not available, please run Ollama server.");
    }

    // create ollama instance
    let (host, port) = dria_env.get_ollama_config();
    let ollama = Ollama::new(host, port);

    // get local models
    let local_models = ollama
        .list_local_models()
        .await?
        .into_iter()
        .map(|m| m.name)
        .collect::<Vec<_>>();

    // prompt the user to select models to be removed
    let selected_models = MultiSelect::new(
        "Choose the models that you would like to remove:",
        local_models.clone(),
    )
    .with_help_message(
        "↑↓ to move, SPACE to select one, ←/→ to select all/none, type to filter models, ENTER to confirm",
    )
    .prompt()?;
    if selected_models.is_empty() {
        log::info!("No models selected, exiting.");
        return Ok(());
    }

    // remove the selected models
    for model in selected_models {
        if let Err(e) = ollama.delete_model(model.clone()).await {
            log::error!("Failed to remove model {}: {}", model, e);
        } else {
            log::info!("Removed model {}", model);
        }
    }

    Ok(())
}
