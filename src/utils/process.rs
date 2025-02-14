use eyre::Result;
use std::path::PathBuf;
use tokio::process::{Child, Command};
use tokio::time;

use super::{DriaRelease, DriaRepo};

/// A launched compute node.
pub struct ComputeInstance {
    /// Executed compute node's directory.
    pub compute_dir: PathBuf,
    /// Executed compute node's file name.
    pub compute_name: PathBuf,
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
                  if let Err(e) = self.handle_compute_update().await {
                    log::error!("Failed to update compute node: {}", e);
                  }
              },
              // launcher self-update checks
               _ = launcher_update_interval.tick() => {
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
        let latest_release = DriaRelease::from_latest_release(DriaRepo::ComputeNode).await?;
        let latest_version = latest_release.version();

        // check if we need to update
        if self.compute_version == latest_version {
            return Ok(());
        }

        // download the latest release to the same path
        latest_release
            .download_release(&self.compute_dir, &self.compute_name)
            .await?;

        // update version field
        self.compute_version = latest_version.into();

        // its safe to do this here even though `monitor_process` waits for a kill
        // signal, because that thread is used within this function at this moment
        self.compute_process.kill().await?;

        // restart the compute node
        self.compute_process = Command::new(self.compute_dir.join(&self.compute_name)).spawn()?;

        Ok(())
    }

    pub async fn handle_launcher_update(&mut self) -> Result<()> {
        let latest_release = DriaRelease::from_latest_release(DriaRepo::Launcher).await?;
        let latest_version = latest_release.version();

        // check if we need to update
        if self.launcher_version == latest_version {
            return Ok(());
        }

        // download the latest release to a temporary path
        latest_release
            .download_release(&self.compute_dir, "launcher-tmp")
            .await?;

        // TODO: !!! download to tempfile

        // TODO: self-replace

        Ok(())
    }
}
