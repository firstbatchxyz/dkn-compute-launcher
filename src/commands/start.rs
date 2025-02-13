use dkn_workflows::DriaWorkflowsConfig;
use eyre::Result;
use std::path::PathBuf;
use tokio::process::{Child, Command};

use crate::{
    utils::{check_ollama, spawn_ollama, DriaRelease},
    DriaEnv,
};

/// The filename for the version tracker file, simply stores the string for the version.
const DKN_VERSION_TRACKER_FILENAME: &str = ".dkn.version";
/// The filename for the latest compute node binary.
const DKN_LATEST_VERSION_FILENAME: &str = "dkn-compute-node_latest";

/// A launched compute node.
pub struct ComputeInstance {
    /// Executed path.
    compute_path: PathBuf,
    /// Workflow configurations, e.g. models.
    workflow_config: DriaWorkflowsConfig,
    /// The compute process handle.
    compute_process: Child,
    /// Optionally launched Ollama process.
    ///
    /// This is only used when the compute node is started with Ollama models
    /// and an Ollama instance is NOT running at that time.
    ollama_process: Option<Child>,
}

/// Starts the latest compute node version.
///
/// The given directory is checked for the latest version of the compute node.
pub async fn run_compute(exe_dir: &PathBuf) -> Result<ComputeInstance> {
    // get the latest release version from repo
    let latest_release = DriaRelease::get_latest_release().await?;

    // read the local latest version from the tracker file
    let local_version_tracker_path = exe_dir.join(DKN_VERSION_TRACKER_FILENAME);
    let local_latest_version =
        std::fs::read_to_string(&local_version_tracker_path).unwrap_or_default();

    // download missing latest release if needed, which is when versions differ or file does not exist
    let compute_path = exe_dir.join(DKN_LATEST_VERSION_FILENAME);
    if latest_release.version() != local_latest_version || !compute_path.exists() {
        // TODO: print local version here as well
        eprintln!("Downloading latest version ({})!", latest_release.version());
        latest_release
            .download_release(exe_dir, DKN_LATEST_VERSION_FILENAME)
            .await?;

        // store the version in the tracker file
        std::fs::write(&local_version_tracker_path, latest_release.version())?;
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

    // launch compute node
    let mut compute_process = Command::new(&compute_path).spawn()?;

    // wait a few seconds
    std::thread::sleep(std::time::Duration::from_secs(10));

    // kill the process
    if let Err(e) = compute_process.kill().await {
        log::error!("Failed to kill Compute process: {}", e);
    } else {
        log::info!("Process killed.");
    }

    Ok(ComputeInstance {
        compute_path,
        compute_process,
        workflow_config,
        ollama_process,
    })
}
