use eyre::{Context, Result};
use ollama_rs::Ollama;
use std::path::Path;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::{
    settings::{self, DriaApiKeyKind},
    utils::{
        check_ollama, configure_fdlimit, pull_model_with_progress, spawn_ollama, ComputeInstance,
    },
    DriaEnv, DKN_LAUNCHER_VERSION,
};

/// An env key that compute node checks to get the path to the environment file.
/// This is set by the launcher when it spawns the compute node.
const DKN_COMPUTE_ENV_KEY: &str = "DKN_COMPUTE_ENV";

/// Starts the latest compute node version at the given path.
///
/// If the environment has Ollama models configured, it will check for Ollama as well
/// and spawn it optionally.
///
/// Automatic updates can be disabled optionally, which is used when we are running a specific version
/// via the `specific` command.
///
/// ### Arguments
/// - `exe_path`: path to the compute node binary
/// - `check_updates`: whether to check for updates or not
///
/// ### Returns
/// A [`ComputeInstance`] with the running compute node process.
///
/// ### Errors
/// - If the compute node process could not be spawned
/// - If the Ollama process is required but could not be spawned
/// - If the file-descriptor limits could not be set
pub async fn run_compute_node(
    exe_path: &Path,
    env_path: &Path,
    check_updates: bool,
) -> Result<ComputeInstance> {
    // get the executables directory back from the path
    let exe_dir = exe_path.parent().expect("must be a file");

    let mut dria_env = DriaEnv::new_from_env();
    dria_env.ask_for_key_if_required()?;
    dria_env.save_to_file(&env_path)?;
    let workflow_config = dria_env.get_model_config();

    // check the update if requested, similar to calling `update` command
    if check_updates {
        log::info!("Checking for updates.");
        super::update(exe_dir).await;
    }

    println!(
        "{:?}",
        workflow_config.get_models_for_provider(dkn_workflows::ModelProvider::Ollama)
    );

    // check if Ollama is running & run it if not
    let ollama_process = if !workflow_config
        .get_models_for_provider(dkn_workflows::ModelProvider::Ollama)
        .is_empty()
    {
        if check_ollama(&dria_env).await {
            None
        } else {
            Some(spawn_ollama(&dria_env).await?)
        }
    } else {
        None // no need for Ollama
    };

    // at this point, we have Ollama running if required, now we can check for models
    ensure_models_are_ready(&mut dria_env).await?;

    // set file-descriptor limits in Unix, not needed in Windows
    configure_fdlimit();

    // add cancellation check, note that this must run BEFORE the compute is spawned
    let cancellation = CancellationToken::new();
    let cancellation_clone = cancellation.clone();
    tokio::spawn(async move { crate::utils::wait_for_termination(cancellation_clone).await });

    // spawn compute node
    let compute_process = Command::new(exe_path)
        .env(DKN_COMPUTE_ENV_KEY, env_path)
        .spawn()
        .wrap_err("failed to spawn compute node")?;

    Ok(ComputeInstance {
        compute_dir: exe_dir.into(),
        launcher_version: DKN_LAUNCHER_VERSION.into(),
        compute_process,
        ollama_process,
        check_updates,
        cancellation,
    })
}

/// Asks for models if they are not already set in the environment.
///
/// - Asks for API keys on the respective model providers.
/// - Pulls local models if required.
async fn ensure_models_are_ready(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    while dria_env.get_model_config().models.is_empty() {
        log::warn!("No models configured. Please choose at least one model to run.");
        settings::edit_models(dria_env)?;
    }

    // FIXME: something weird about env here, todo test with Ollama

    let model_config = dria_env.get_model_config();

    // check API keys for the requied models
    let required_api_keys = DriaApiKeyKind::from_providers(&model_config.get_providers());
    for api_key in required_api_keys {
        log::info!("Provide {} because you are using its model", api_key);
        let new_value = api_key.prompt_api(dria_env)?;
        dria_env.set(api_key.name(), new_value);
    }

    // pull local models if used
    let ollama_models = model_config.get_models_for_provider(dkn_workflows::ModelProvider::Ollama);
    if !ollama_models.is_empty() {
        // create ollama instance
        let (host, port) = dria_env.get_ollama_config();
        let ollama = Ollama::new(host, port);

        // get local models
        let local_model_names = ollama
            .list_local_models()
            .await?
            .into_iter()
            .map(|m| m.name)
            .collect::<Vec<_>>();

        // find models that are not available locally
        let models_to_be_pulled = ollama_models
            .iter()
            .filter(|model| !local_model_names.contains(&model.to_string()))
            .collect::<Vec<_>>();

        // pull all selected & non-pulled models
        if !models_to_be_pulled.is_empty() {
            log::info!("The following models are selected but not found locally:");
            for model in &models_to_be_pulled {
                log::info!("  - {}", model);
            }

            log::info!("Pulling models from Ollama...");
            for model in models_to_be_pulled {
                pull_model_with_progress(&ollama, model.to_string()).await?;
            }
        }
    }

    Ok(())
}
