use eyre::{Context, Result};
use std::path::Path;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::{
    utils::{check_ollama, spawn_ollama, ComputeInstance},
    DriaEnv, DKN_LAUNCHER_VERSION,
};

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
pub async fn run_compute(exe_path: &Path, check_updates: bool) -> Result<ComputeInstance> {
    // get the executables directory back from the path
    let exe_dir = exe_path.parent().expect("must be a file");

    // check the update if requested, similar to calling `update` command
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

        if let Err(e) = setrlimit(Resource::NOFILE, DEFAULT_SOFT_LIMIT, DEFAULT_HARD_LIMIT) {
            log::error!(
                "Failed to set file-descriptor limits: {}, you may need to run as administrator!",
                e
            );
        }

        let (soft, hard) = Resource::NOFILE.get().unwrap_or_default();
        log::warn!("Using resource limits (soft / hard): {} / {}", soft, hard);
    }

    // add cancellation check, note that this must run BEFORE the compute is spawned
    let cancellation = CancellationToken::new();
    let cancellation_clone = cancellation.clone();
    tokio::spawn(async move { crate::utils::wait_for_termination(cancellation_clone).await });

    // spawn compute node
    let compute_process = Command::new(exe_path)
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
