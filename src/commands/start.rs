use eyre::{Context, Result};
use std::path::Path;
use tokio::process::Command;

use crate::{
    utils::{
        check_ollama, spawn_ollama, ComputeInstance, DriaRelease, DriaRepository,
        DKN_LATEST_COMPUTE_FILENAME,
    },
    DriaEnv, DKN_LAUNCHER_VERSION,
};

/// Starts the latest compute node version.
///
/// The given directory is checked for the latest version of the compute node.
pub async fn run_compute(exe_dir: &Path, enable_updates: bool) -> Result<ComputeInstance> {
    // get the latest release version from repo
    let latest_release = DriaRelease::from_latest_release(DriaRepository::ComputeNode).await?;
    let latest_version = latest_release.version();

    // read the local latest version from the tracker file
    let local_latest_version = DriaRelease::get_compute_version(exe_dir);

    // download missing latest release if needed, which is when versions differ or file does not exist
    let compute_path = exe_dir.join(DKN_LATEST_COMPUTE_FILENAME);

    if !local_latest_version
        .as_ref()
        .is_some_and(|v| v == latest_version)
        || !compute_path.exists()
    {
        match local_latest_version {
            Some(v) => log::info!("Updating from {} to latest version {}!", v, latest_version),
            None => log::info!("Downloading latest version {}!", latest_version),
        };
        latest_release
            .download_release(exe_dir, DKN_LATEST_COMPUTE_FILENAME)
            .await?;

        // store the version in the tracker file
        DriaRelease::set_compute_version(exe_dir, latest_version)?;
    }

    let dria_env = DriaEnv::new_from_env();
    let workflow_config = dria_env.get_model_config();

    // handle ollama checks
    let ollama_process = if !workflow_config
        .get_models_for_provider(dkn_workflows::ModelProvider::Ollama)
        .is_empty()
    {
        // check if Ollama is running & run it if not
        if check_ollama(&dria_env).await {
            None
        } else {
            Some(spawn_ollama(&dria_env).await?)
        }
    } else {
        // no need for Ollama
        None
    };

    // spawn compute node
    #[cfg(unix)]
    {
        // set file-descriptor limits in Unix, not needed in Windows
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

    let compute_process = Command::new(compute_path)
        .spawn()
        .wrap_err("failed to spawn compute node")?;

    Ok(ComputeInstance {
        compute_dir: exe_dir.into(),
        compute_version: latest_version.into(),
        // launcher version is the version of the binary that started the compute node
        launcher_version: DKN_LAUNCHER_VERSION.into(),
        compute_process,
        ollama_process,
        check_updates: enable_updates,
    })
}
