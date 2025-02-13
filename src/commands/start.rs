use eyre::Result;
use std::path::PathBuf;
use tokio::process::Command;

use crate::{
    utils::{check_ollama, spawn_ollama, ComputeInstance, DriaRelease},
    DriaEnv,
};

/// The filename for the version tracker file, simply stores the string for the version.
const DKN_VERSION_TRACKER_FILENAME: &str = ".dkn.version";
/// The filename for the latest compute node binary.
const DKN_LATEST_VERSION_FILENAME: &str = "dkn-compute-node_latest";

/// Starts the latest compute node version.
///
/// The given directory is checked for the latest version of the compute node.
pub async fn run_compute(exe_dir: &PathBuf) -> Result<ComputeInstance> {
    // get the latest release version from repo
    let latest_release = DriaRelease::get_latest_compute_release().await?;
    let latest_version = latest_release.version();

    // read the local latest version from the tracker file
    let local_version_tracker_path = exe_dir.join(DKN_VERSION_TRACKER_FILENAME);
    let local_latest_version =
        std::fs::read_to_string(&local_version_tracker_path).unwrap_or_default();

    // download missing latest release if needed, which is when versions differ or file does not exist
    let compute_path = exe_dir.join(DKN_LATEST_VERSION_FILENAME);
    if latest_version != local_latest_version || !compute_path.exists() {
        if local_latest_version.is_empty() {
            eprintln!(
                "Upgrading from {} to latest version {}!",
                local_latest_version, latest_version
            );
        } else {
            eprintln!("Downloading latest version {}!", latest_version);
        }
        latest_release
            .download_release(exe_dir, DKN_LATEST_VERSION_FILENAME)
            .await?;

        // store the version in the tracker file
        std::fs::write(&local_version_tracker_path, latest_version)?;
    }

    let dria_env = DriaEnv::new();
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
        compute_name: DKN_LATEST_VERSION_FILENAME.into(),
        compute_version: latest_version.into(),
        compute_process,
        workflow_config,
        ollama_process,
    })
}
