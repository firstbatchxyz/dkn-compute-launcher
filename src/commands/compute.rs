use dkn_workflows::DriaWorkflowsConfig;
use eyre::{eyre, Result};
use std::path::PathBuf;
use tokio::process::{Child, Command};

use crate::{
    utils::{check_ollama, spawn_ollama},
    DriaEnv,
};

/// A launched compute node.
pub struct ComputeInstance {
    compute_path: PathBuf,
    compute_process: Child,
    workflow_config: DriaWorkflowsConfig,
    /// Optionally launched Ollama process.
    ///
    /// This is only used when the compute node is started with Ollama models
    /// and an Ollama instance is NOT running at that time.
    ollama_process: Option<Child>,
}

/// Starts a compute node at the given path.
///
/// The `Command` inherits the current program's environment variables,
/// so by using the launcher we are able to start the compute node with the same environment variables as the launcher.
pub async fn run_compute(compute_path: &PathBuf) -> Result<()> {
    let dria_env = DriaEnv::new();
    if !compute_path.exists() {
        return Err(eyre!("Compute binary not found at {:?}", compute_path));
    }

    // TODO: can we detect if the compute node is already running at the same address and kill it?

    // check if Ollama is running & run it if not
    // TODO: if ollama is required w.r.t chosen models
    // let ollama_process = if check_ollama(&dria_env).await {
    //     None
    // } else {
    //     Some(spawn_ollama(&dria_env).await?)
    // };

    // launch compute node
    let mut compute_process = Command::new(compute_path).spawn()?;

    // wait a few seonds
    std::thread::sleep(std::time::Duration::from_secs(10));

    // kill the process
    if let Err(e) = compute_process.kill().await {
        log::error!("Failed to kill Compute process: {}", e);
    } else {
        log::info!("Process killed.");
    }

    Ok(())
}
