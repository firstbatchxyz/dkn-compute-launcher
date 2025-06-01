use dkn_executor::{ollama_rs::Ollama, ModelProvider};
use eyre::{Context, Result};
use std::{collections::HashSet, path::Path};
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

    // check the update if requested, similar to calling `update` command
    if check_updates {
        super::update(exe_dir).await;
    }

    // read existing env
    let mut dria_env = DriaEnv::new_from_env();

    // ensure there are models
    let mut models = dria_env.get_models();
    while models.is_empty() {
        log::warn!("No models configured. Please choose at least one model to run.");
        settings::edit_models(&mut dria_env)?;
        models = dria_env.get_models();
    }

    // ensure key is set
    dria_env.ask_for_key_if_required()?;

    // check API keys for the providers that are used with the selected models
    let providers = models
        .iter()
        .map(|model| model.provider())
        .collect::<HashSet<_>>();
    for api_key in DriaApiKeyKind::from_providers(providers.into_iter()) {
        if dria_env.get(api_key.name()).is_none() {
            log::info!("Provide {} because you are using its model", api_key);
            let new_value = api_key.prompt_api(&dria_env)?;
            dria_env.set(api_key.name(), new_value);
        }
    }

    // check if Ollama is required & running, and run it if not
    let ollama_models = models
        .iter()
        .cloned()
        .filter(|m| m.provider() == ModelProvider::Ollama)
        .collect::<Vec<_>>();
    let ollama_process = if !ollama_models.is_empty() {
        // spawn Ollama if needed
        let ollama_process_opt = if check_ollama(&dria_env).await {
            None
        } else {
            Some(spawn_ollama(&dria_env).await?)
        };

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
            log::info!(
                "The following models are selected but not found locally:\n{}",
                models_to_be_pulled
                    .iter()
                    .map(|m| format!("  - {}", m))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            log::info!("Pulling models from Ollama...");
            for model in models_to_be_pulled {
                pull_model_with_progress(&ollama, model.to_string()).await?;
            }
        }

        ollama_process_opt
    } else {
        None // no need for Ollama
    };

    // save to file if there were any changes
    if dria_env.is_changed() {
        dria_env.save_to_file(env_path)?;

        // override the env file with the new values, needed for the compute node
        // as even if it reads from env again, it will not override existing values
        if let Err(err) = dotenvy::from_path_override(env_path) {
            log::warn!("Failed to override with env: {}", err);
        }
    }

    // set file-descriptor limits in Unix, not needed in Windows
    configure_fdlimit();

    // add cancellation check, note that this must run BEFORE the compute is spawned
    let cancellation = CancellationToken::new();
    let cancellation_clone = cancellation.clone();
    tokio::spawn(async move { crate::utils::wait_for_termination(cancellation_clone).await });

    // spawn compute node
    let compute_process = Command::new(exe_path)
        // add env variable for the path, respecting the `--profile` option
        .env(DKN_COMPUTE_ENV_KEY, env_path)
        // let compute node know that it is started by the launcher
        // see: https://github.com/firstbatchxyz/dkn-compute-node/blob/master/compute/src/config.rs#L126
        .env(
            "DKN_EXEC_PLATFORM",
            format!("launcher/v{DKN_LAUNCHER_VERSION}"),
        )
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
