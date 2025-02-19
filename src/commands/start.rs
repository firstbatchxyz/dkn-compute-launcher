use eyre::{Context, Result};
use std::path::Path;
use tokio::process::Command;

use crate::{
    utils::{check_ollama, spawn_ollama, ComputeInstance, DKN_LATEST_COMPUTE_FILE},
    DriaEnv, DKN_LAUNCHER_VERSION,
};

/// Starts the latest compute node version.
///
/// The given directory is checked for the latest version of the compute node.
/// If the version is not found or differs from the latest version, the latest version is downloaded automatically.
pub async fn run_compute(exe_dir: &Path, specific: Option<&Path>) -> Result<ComputeInstance> {
    // check the update if requested, similar to calling `update` command
    let check_updates = specific.is_none();
    if check_updates {
        log::info!("Checking for updates.");
        super::update(exe_dir).await;
    }

    let dria_env = DriaEnv::new_from_env();
    let workflow_config = dria_env.get_model_config();

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

    // set file-descriptor limits in Unix, not needed in Windows
    #[cfg(unix)]
    {
        use rlimit::{setrlimit, Resource};

        const DEFAULT_SOFT_LIMIT: u64 = 4 * 1024 * 1024;
        const DEFAULT_HARD_LIMIT: u64 = 40 * 1024 * 1024;

        log::debug!("Resource limits before: {:?}", Resource::NOFILE.get());
        if let Err(e) = setrlimit(Resource::NOFILE, DEFAULT_SOFT_LIMIT, DEFAULT_HARD_LIMIT) {
            log::error!("Failed to set file-descriptor limits: {}", e);
            log::warn!("Resource limits: {:?}", Resource::NOFILE.get());
        } else {
            log::debug!("Resource limits after: {:?}", Resource::NOFILE.get());
        }
    }

    // spawn compute node
    let compute_process = Command::new(match specific {
        Some(path) => path.to_path_buf(),
        None => exe_dir.join(DKN_LATEST_COMPUTE_FILE),
    })
    .spawn()
    .wrap_err("failed to spawn compute node")?;

    Ok(ComputeInstance {
        compute_dir: exe_dir.into(),
        launcher_version: DKN_LAUNCHER_VERSION.into(),
        compute_process,
        ollama_process,
        check_updates,
    })
}
