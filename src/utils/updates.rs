use std::path::Path;

use eyre::Result;

use super::{DriaRelease, DriaRepository, DKN_LATEST_COMPUTE_FILE};

/// Check if there is an update required for the compute node.
///
/// Returns the latest release, along with a boolean indicating whether an update is required.
pub async fn check_for_compute_node_update(exe_dir: &Path) -> Result<(DriaRelease, bool)> {
    // read the local latest version from the tracker file
    // if file does not exist it returns `None`, which indicates an update is required
    let current_version = DriaRelease::get_compute_version(exe_dir);

    // get the latest release version from repo
    let latest_release = DriaRelease::from_latest_release(DriaRepository::ComputeNode).await?;
    let latest_version = latest_release.version();

    // checks if compute path exists
    let compute_exists = exe_dir.join(DKN_LATEST_COMPUTE_FILE).exists();

    // update is required only if the local version is not the latest or the compute file does not exist
    let requires_update = !current_version
        .as_ref()
        .is_some_and(|v| v == latest_version)
        || !compute_exists;

    Ok((latest_release, requires_update))
}

/// Check if there is an update required for the launcher.
///
/// Returns the latest release, along with a boolean indicating whether update is required.
pub async fn check_for_launcher_update(current_version: &str) -> Result<(DriaRelease, bool)> {
    // get the latest release version from repo
    let latest_release = DriaRelease::from_latest_release(DriaRepository::Launcher).await?;
    let latest_version = latest_release.version();

    // update is required only if the local version is not the latest
    let requires_update = current_version != latest_version;

    Ok((latest_release, requires_update))
}
