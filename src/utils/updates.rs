use std::path::{Path, PathBuf};

use eyre::Result;

use super::{DriaRelease, DriaRepository};

/// The latest compute node will always be at this file for a chosen directory.
pub const DKN_LATEST_COMPUTE_FILENAME: &str = "dkn-compute-node_latest";

/// Check if there is an update required for the compute node.
pub async fn check_compute_update(exe_dir: &Path) -> Result<()> {
    // get the latest release version from repo
    let latest_release = DriaRelease::from_latest_release(DriaRepository::ComputeNode).await?;
    let latest_version = latest_release.version();

    // read the local latest version from the tracker file
    let local_latest_version = DriaRelease::get_compute_version(exe_dir);

    // download missing latest release if needed, which is when versions differ or file does not exist
    let compute_path = exe_dir.join(DKN_LATEST_COMPUTE_FILENAME);

    if !local_latest_version
        .as_ref()
        .is_some_and(|v| v == latest_version)
        || !compute_path.exists()
    {
        match local_latest_version {
            Some(v) => log::info!("Updating from {} to latest version {}!", v, latest_version),
            None => log::info!("Downloading latest version {}!", latest_version),
        };
        latest_release
            .download_release(exe_dir, DKN_LATEST_COMPUTE_FILENAME)
            .await?;

        // store the version in the tracker file
        DriaRelease::set_compute_version(exe_dir, latest_version)?;

        Ok(())
    } else {
        Ok(())
    }
}

/// Downloads the latest compute node release.
///
/// Returns a path to the downloaded release and the version of the release.
/// If the local version was the latest, returns `None` for the path.
pub async fn download_latest_compute_node(
    exe_dir: &Path,
    local_version: &str,
) -> Result<(Option<PathBuf>, String)> {
    // get latest release & check if we need to update
    let latest_release = DriaRelease::from_latest_release(DriaRepository::ComputeNode).await?;
    let latest_version = latest_release.version();
    if local_version == latest_version {
        Ok((None, latest_version.into()))
    } else {
        // download the latest release to the same path
        let latest_path = latest_release
            .download_release(exe_dir, DKN_LATEST_COMPUTE_FILENAME)
            .await?;

        Ok((Some(latest_path), latest_version.into()))
    }
}

/// Downloads the latest launcher release.
///
/// Returns a path to the downloaded release and the version of the release.
/// If the local version was the latest, returns `None` for the path.
///
/// Note that launcher releases below `0.1.0` are ignored because they
/// belong to the old Go code.
pub async fn download_latest_launcher(
    exe_dir: &Path,
    local_version: &str,
) -> Result<(Option<PathBuf>, String)> {
    const TMP_FILE_NAME: &str = ".tmp.launcher";

    // get latest release & check if we need to update
    let latest_release = DriaRelease::from_latest_release(DriaRepository::Launcher).await?;
    let latest_version = latest_release.version();

    if local_version == latest_version {
        //
        Ok((None, latest_version.into()))
    } else {
        // download the latest release to a temporary path
        let latest_path = latest_release
            .download_release(exe_dir, TMP_FILE_NAME)
            .await?;

        Ok((Some(latest_path), latest_version.into()))
    }
}
