use eyre::Result;
use std::path::PathBuf;
use tokio::process::Command;

use crate::{
    utils::{
        check_ollama, spawn_ollama, ComputeInstance, DriaRelease, DriaRepository,
        DKN_LATEST_COMPUTE_FILENAME,
    },
    DriaEnv, CRATE_VERSION,
};

/// Starts the latest compute node version.
///
/// The given directory is checked for the latest version of the compute node.
pub async fn run_compute(exe_dir: &PathBuf, enable_updates: bool) -> Result<ComputeInstance> {
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
    let compute_process = Command::new(&compute_path).spawn()?;

    Ok(ComputeInstance {
        compute_dir: exe_dir.into(),
        compute_version: latest_version.into(),
        // launcher version is the version of the binary that started the compute node
        launcher_version: CRATE_VERSION.into(),
        compute_process,
        ollama_process,
        check_updates: enable_updates,
    })
}
