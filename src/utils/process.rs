use dkn_workflows::DriaWorkflowsConfig;
use std::{path::PathBuf, time::Duration};
use tokio::process::Child;

/// A launched compute node.
pub struct ComputeInstance {
    /// Executed compute node's path.
    pub compute_path: PathBuf,
    /// Executed compute node's version.
    pub compute_version: String,
    /// Workflow configurations, e.g. models.
    pub workflow_config: DriaWorkflowsConfig,
    /// The compute process handle.
    pub compute_process: Child,
    /// Optionally launched Ollama process.
    ///
    /// This is only used when the compute node is started with Ollama models
    /// and an Ollama instance is NOT running at that time.
    pub ollama_process: Option<Child>,
}

impl ComputeInstance {
    /// The main loop of compute process. It handles the following:
    ///
    /// - Monitors compute node process, exits on error.
    /// - Keeps a handle on Ollama process as well if needed, to shut it down when compute node is stopped.
    /// - Handles signals to gracefully shut down the compute node.
    /// - Every now and then checks for the latest compute node release, and restarts it if there is an update.
    /// - EVery now and then checks for the latest launcher release, and replaces the binary "in-place" if there is an update.
    pub async fn monitor_process(&mut self) {
        /// Number of seconds between refreshing for compute node updates.
        const COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS: u64 = 25 * 60;
        /// Number of seconds between refreshing for launcher updates.
        const LAUNCHER_UPDATE_CHECK_INTERVAL_SECS: u64 = 25 * 60;

        let mut compute_node_update_interval =
            tokio::time::interval(Duration::from_secs(COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS));
        compute_node_update_interval.tick().await; // move one tick

        let mut launcher_update_interval =
            tokio::time::interval(Duration::from_secs(LAUNCHER_UPDATE_CHECK_INTERVAL_SECS));
        launcher_update_interval.tick().await; // move one tick

        loop {
            tokio::select! {
              // wait for compute node to exit; the node will handle signals on its own so we dont
              // have to check for signal and call kill() explicitly here
              // FIXME: what happens when we kill() due to compute update?
              _ = self.compute_process.wait() => {
                  // now that compute is closed, we should kill Ollama if it was launched by us
                  if let Some(ollama_process) = &mut self.ollama_process {
                      if let Err(e) = ollama_process.kill().await {
                          eprintln!("Failed to kill Ollama process: {}", e);
                      }
                  }
                  break;
              },
               _ = compute_node_update_interval.tick() => self.handle_compute_update().await,
               _ = launcher_update_interval.tick() => self.handle_launcher_update().await,
            }
        }

        eprintln!("Quitting launcher!");
    }

    pub async fn handle_compute_update(&mut self) {}

    pub async fn handle_launcher_update(&mut self) {}
}
