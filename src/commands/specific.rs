use std::path::{Path, PathBuf};

use inquire::Select;

use crate::{get_releases, utils::DriaRepository};

/// Prompts the user to select a version to download, which is downloaded to `exe_dir` directory.
pub async fn download_specific_release(
    exe_dir: &Path,
    tag: Option<&String>,
) -> eyre::Result<PathBuf> {
    let releases = get_releases(DriaRepository::ComputeNode).await?;

    let chosen_release = match tag {
        // choose the tag directly
        Some(tag) => releases
            .into_iter()
            .find(|release| release.version() == tag)
            .ok_or_else(|| eyre::eyre!("No release found for tag: {}", tag))?,
        // prompt the user for selection
        None => Select::new("Select a version:", releases)
            .with_help_message("↑↓ to move, enter to select, type to filter, ESC to abort")
            .prompt()?,
    };

    let filename = chosen_release.to_filename()?;
    let dest_path = exe_dir.join(&filename);
    if !dest_path.exists() {
        log::info!("Downloading version: {}", chosen_release);
        chosen_release
            .download_release(exe_dir, filename, true)
            .await
    } else {
        log::info!("Using existing version: {}", chosen_release);
        Ok(dest_path)
    }
}
