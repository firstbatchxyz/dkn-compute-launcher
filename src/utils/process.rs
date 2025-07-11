use eyre::{Context, Result};
use self_update::self_replace;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::utils::{DriaRelease, DKN_LATEST_COMPUTE_FILE};

use super::{check_for_compute_node_update, check_for_launcher_update};

/// Number of seconds between refreshing for compute node updates.
const COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS: Duration = Duration::from_secs(60 * 60); // every few hours
/// Number of seconds between refreshing for launcher updates.
const LAUNCHER_UPDATE_CHECK_INTERVAL_SECS: Duration = Duration::from_secs(3 * 60 * 60); // every few hours

/// A launched compute node.
pub struct ComputeInstance {
    /// Executed compute node's directory.
    pub compute_dir: PathBuf,
    /// The compute process handle.
    pub compute_process: Child,
    /// Executed launcher version.
    pub launcher_version: String,
    /// Optionally launched Ollama process [`Child`].
    ///
    /// This is only used when the compute node is started with Ollama models
    /// and an Ollama instance is NOT running at that time.
    pub ollama_process: Option<Child>,
    /// Whether to check for updates or not.
    ///
    /// This is `true` unless you are running a specific version for a particular reason.
    pub check_updates: bool,
    /// [`CancellationToken`] for the main loop.
    pub cancellation: CancellationToken,
}

impl ComputeInstance {
    /// The main loop of compute process. It handles the following:
    ///
    /// - Monitors compute node process, exits on error.
    /// - Keeps a handle on Ollama process as well if needed, to shut it down when compute node is stopped.
    /// - Handles signals to gracefully shut down the compute node.
    /// - Every [`COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS`] checks for the latest compute node release, and restarts it if there is an update.
    /// - EVery [`LAUNCHER_UPDATE_CHECK_INTERVAL_SECS`] checks for the latest launcher release, and replaces the binary "in-place" if there is an update.
    pub async fn monitor_process(&mut self) {
        let mut compute_node_update_interval = interval(COMPUTE_NODE_UPDATE_CHECK_INTERVAL_SECS);
        let mut launcher_update_interval = interval(LAUNCHER_UPDATE_CHECK_INTERVAL_SECS);

        // move one tick
        launcher_update_interval.tick().await;
        compute_node_update_interval.tick().await;

        loop {
            tokio::select! {
              // additional check in case the process is closed unexpectedly
              _ = self.compute_process.wait() => {
                log::info!("Compute node was closed, terminating.");

                  // now that compute is closed, we should kill Ollama if it was launched by us
                  self.close_ollama().await.unwrap_or_else(|e| log::warn!("Failed to close Ollama: {}", e));
                  break;
              },
              // cancellation signal, indicates that a signal has been received to shut down
              _ = self.cancellation.cancelled() => {
                  log::info!("Received cancellation signal, shutting down launcher.");

                  // close ollama if it was launched by us
                  self.close_ollama().await.unwrap_or_else(|e| log::warn!("Failed to close Ollama: {}", e));

                  // kill the compute process, note that the compute process may handle the signal as well on its own,
                  // but we need to make sure that it is killed in case it doesn't (TODO: may be OS related?)
                  if let Err(e) = self.compute_process.kill().await {
                    log::warn!("Failed to kill compute process: {}", e);
                  }

                  break;
              }
              // compute node update checks
               _ = compute_node_update_interval.tick() => {
                  if !self.check_updates { continue; }

                  if let Err(err) = self.handle_compute_update().await {
                    log::error!("Error updating compute node: {err}");
                  }
              },
              // launcher self-update checks
               _ = launcher_update_interval.tick() => {
                  if !self.check_updates { continue; }

                  if let Err(err) = self.handle_launcher_update().await {
                    log::error!("Error updating launcher: {err}");
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
        // check version
        let (latest_release, requires_update) =
            check_for_compute_node_update(&self.compute_dir).await?;

        if requires_update {
            // kill existing compute node
            //
            // its safe to do this here even though `monitor_process` waits for a kill
            // signal, because that thread is used within this function at this moment
            self.compute_process.kill().await?;

            log::info!(
                "Updating compute node to version from to {}",
                latest_release.version()
            );

            let latest_path = latest_release
                .download_release(&self.compute_dir, DKN_LATEST_COMPUTE_FILE, true)
                .await?;

            // restart the compute node
            //
            // we dont set file-descriptors here again, because the process already
            // has that setting on the first launch
            self.compute_process = Command::new(latest_path).spawn()?;

            // update version tracker
            DriaRelease::set_compute_version(&self.compute_dir, latest_release.version())?;
        }

        Ok(())
    }

    async fn close_ollama(&mut self) -> Result<()> {
        if let Some(ollama_process) = &mut self.ollama_process {
            if let Err(e) = ollama_process.kill().await {
                log::warn!("Failed to kill Ollama process: {}", e);
            }
        }

        Ok(())
    }

    /// Checks for the latest launcher release and updates if needed.
    ///
    /// This replaces the existing launcher binary.
    pub async fn handle_launcher_update(&mut self) -> Result<()> {
        // check version
        let (latest_release, requires_update) =
            check_for_launcher_update(&self.launcher_version).await?;

        if requires_update {
            log::info!(
                "Updating launcher version from {} to {}",
                self.launcher_version,
                latest_release.version()
            );

            // we don't log the progress here
            let latest_path = latest_release
                .download_release(&self.compute_dir, ".tmp_launcher", false)
                .await?;

            // replace its own binary with the latest version
            self_replace::self_replace(&latest_path)?;

            // remove the temporary file
            std::fs::remove_file(&latest_path)
                .wrap_err("could not remove temporary launcher file")?;
        }

        Ok(())
    }
}
