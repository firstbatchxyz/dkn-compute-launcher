use std::path::PathBuf;

use dkn_workflows::DriaWorkflowsConfig;
use tokio::process::{Child, Command};

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
pub async fn run_compute(compute_path: &PathBuf) -> std::io::Result<Child> {
    // TODO: check Ollama models and other requirements

    // launch compute node
    Command::new(compute_path).spawn()

    // // wait a few seonds
    // std::thread::sleep(std::time::Duration::from_secs(5));

    // // kill the process
    // if let Err(e) = child.kill().await {
    //     log::error!("Failed to kill Compute process: {}", e);
    // } else {
    //     log::info!("Ollama process killed.");
    // }
    // Ok(())
}
