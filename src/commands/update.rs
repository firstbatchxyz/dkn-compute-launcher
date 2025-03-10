use eyre::Result;
use self_update::self_replace;
use std::path::Path;

use crate::utils::{
    check_for_compute_node_update, check_for_launcher_update, DriaRelease, DKN_LATEST_COMPUTE_FILE,
    DKN_LAUNCHER_VERSION,
};

/// Updates the compute node and launcher to the latest version.
///
/// See [`update_compute`] and [`update_launcher`] for more details.
///
/// ### Arguments
/// - `exe_dir`: directory where the binary is located
#[inline]
pub async fn update(exe_dir: &Path) {
    log::debug!("Checking compute node version.");
    if let Err(e) = update_compute(exe_dir).await {
        log::error!("Error updating compute node: {}", e);
    }

    // update the launcher only in release mode, otherwise this will try to update
    // when you are running with `cargo run` etc.
    if cfg!(debug_assertions) {
        log::debug!("Checking launcher version.");
        if let Err(e) = update_launcher(exe_dir).await {
            log::error!("Error updating launcher: {}", e);
        }
    }
}

/// Updates the launcher node, replacing the current binary with the latest one via `self_replace`.
///
/// ### Arguments
/// - `exe_dir`: directory where the binary is located
///
/// ### Errors
/// - If latest release could not be downloaded
/// - If self-replace fails
/// - If the temporary file fails to be removed.
async fn update_launcher(exe_dir: &Path) -> Result<()> {
    // the local version is read from the constant value in the binary
    let (latest_release, requires_update) = check_for_launcher_update(DKN_LAUNCHER_VERSION).await?;

    if requires_update {
        log::info!("Updating launcher to version: {}", latest_release.version());

        let latest_path = latest_release
            .download_release(exe_dir, ".tmp_launcher", true)
            .await?;

        // replace its own binary with the latest version
        self_replace::self_replace(&latest_path)?;

        // remove the temporary file
        std::fs::remove_file(&latest_path)?;
    } else {
        log::info!(
            "Launcher already at latest version: {}",
            latest_release.version()
        );
    }

    Ok(())
}

/// Updates the compute node, replacing the `latest` binary at the given directory with the new version.
///
/// ### Arguments
/// - `exe_dir`: directory where the binary is located
///
/// ### Errors
/// - If latest release could not be downloaded
/// - If local version tracker update does not complete
async fn update_compute(exe_dir: &Path) -> Result<()> {
    let (latest_release, requires_update) = check_for_compute_node_update(exe_dir).await?;
    if requires_update {
        log::info!(
            "Updating compute node to version: {}",
            latest_release.version()
        );

        latest_release
            .download_release(exe_dir, DKN_LATEST_COMPUTE_FILE, true)
            .await?;

        // store the version as well
        DriaRelease::set_compute_version(exe_dir, latest_release.version())?;
    } else {
        log::info!(
            "Compute node already at latest version: {}",
            latest_release.version()
        );
    }

    Ok(())
}
