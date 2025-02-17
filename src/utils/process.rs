use eyre::{Context, Result};
use self_update::self_replace;
use std::path::PathBuf;
use tokio::process::{Child, Command};
use tokio::time;

use super::{download_latest_compute_node, download_latest_launcher};

/// A launched compute node.
pub struct ComputeInstance {
    /// Executed compute node's directory.
    pub compute_dir: PathBuf,
    /// Executed compute node's version.
    pub compute_version: String,
    /// The compute process handle.
    pub compute_process: Child,
    /// Executed launcher version.
    pub launcher_version: String,
    /// Optionally launched Ollama process.
    ///
    /// This is only used when the compute node is started with Ollama models
    /// and an Ollama instance is NOT running at that time.
    pub ollama_process: Option<Child>,
    /// Whether to check for updates or not.
    pub check_updates: bool,
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
        const COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS: u64 = 2 * 60;
        /// Number of seconds between refreshing for launcher updates.
        const LAUNCHER_UPDATE_CHECK_INTERVAL_SECS: u64 = 2 * 60;

        let mut compute_node_update_interval = time::interval(time::Duration::from_secs(
            COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS,
        ));
        compute_node_update_interval.tick().await; // move one tick

        let mut launcher_update_interval = time::interval(time::Duration::from_secs(
            LAUNCHER_UPDATE_CHECK_INTERVAL_SECS,
        ));
        launcher_update_interval.tick().await; // move one tick

        loop {
            tokio::select! {
              // wait for compute node to exit; the node will handle signals on its own so we dont
              // have to check for signal and call kill() explicitly here
              _ = self.compute_process.wait() => {
                  // now that compute is closed, we should kill Ollama if it was launched by us
                  if let Some(ollama_process) = &mut self.ollama_process {
                      if let Err(e) = ollama_process.kill().await {
                          log::warn!("Failed to kill Ollama process: {}", e);
                      }
                  }
                  break;
              },
              // compute node update checks
               _ = compute_node_update_interval.tick() => {
                  if !self.check_updates { continue; }
                  if let Err(e) = self.handle_compute_update().await {
                    log::error!("Failed to update compute node: {}", e);
                  }
              },
              // launcher self-update checks
               _ = launcher_update_interval.tick() => {
                  if !self.check_updates { continue; }
                  if let Err(e) = self.handle_launcher_update().await {
                    log::error!("Failed to update launcher: {}", e);
                  }
              },
            }
        }

        log::warn!("Quitting launcher!");
    }

    /// Checks for the latest compute node release and updates if needed.
    ///
    /// This replaces the existing process on-the-run.
    pub async fn handle_compute_update(&mut self) -> Result<()> {
        let (latest_path, latest_version) =
            download_latest_compute_node(&self.compute_dir, &self.launcher_version).await?;

        if let Some(latest_path) = latest_path {
            self.compute_version = latest_version;

            // its safe to do this here even though `monitor_process` waits for a kill
            // signal, because that thread is used within this function at this moment
            self.compute_process.kill().await?;

            // restart the compute node
            self.compute_process = Command::new(latest_path).spawn()?;
        }

        Ok(())
    }

    /// Checks for the latest launcher release and updates if needed.
    ///
    /// This replaces the existing launcher binary.
    pub async fn handle_launcher_update(&mut self) -> Result<()> {
        let (latest_path, latest_version) =
            download_latest_launcher(&self.compute_dir, &self.launcher_version).await?;

        if let Some(latest_path) = latest_path {
            self.launcher_version = latest_version;

            self_replace::self_replace(&latest_path).wrap_err("could not update launcher")?;

            // remove the temporary file
            std::fs::remove_file(&latest_path)
                .wrap_err("could not remove temporary launcher file")?;
        }

        Ok(())
    }
}
