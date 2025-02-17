use std::path::Path;

use eyre::Result;
use self_update::self_replace;

use crate::{
    utils::{download_latest_compute_node, download_latest_launcher, DriaRelease},
    DKN_LAUNCHER_VERSION,
};

#[inline]
pub async fn update(exe_dir: &Path) -> Result<()> {
    log::info!("Updating compute node...");
    update_compute(exe_dir).await?;

    log::info!("Updating launcher");
    update_launcher(exe_dir).await?;

    Ok(())
}

async fn update_launcher(exe_dir: &Path) -> Result<()> {
    let (latest_path, latest_version) =
        download_latest_launcher(exe_dir, DKN_LAUNCHER_VERSION).await?;

    if let Some(latest_path) = latest_path {
        log::info!("Updated launcher to version: {}", latest_version);
        self_replace::self_replace(&latest_path)?;

        // remove the temporary file
        std::fs::remove_file(&latest_path)?;
    } else {
        log::info!("Launcher already at latest version: {}", latest_version);
    }

    Ok(())
}

async fn update_compute(exe_dir: &Path) -> Result<()> {
    let local_version = DriaRelease::get_compute_version(exe_dir).unwrap_or_default();
    let (latest_path, latest_version) =
        download_latest_compute_node(exe_dir, &local_version).await?;

    if latest_path.is_some() {
        log::info!("Updated compute node to version: {}", latest_version);

        // store the version as well
        DriaRelease::set_compute_version(exe_dir, &latest_version)?;
    } else {
        log::info!("Compute node already at latest version: {}", latest_version);
    }

    Ok(())
}
