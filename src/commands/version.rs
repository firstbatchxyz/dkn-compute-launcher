use std::path::PathBuf;

use inquire::Select;

use crate::utils::get_compute_releases;

/// Prompts the user to select a version to download, which is downloaded to `exe_dir` directory.
pub async fn change_version(exe_dir: &PathBuf) -> eyre::Result<Option<PathBuf>> {
    let releases = get_compute_releases().await?;

    let Some(chosen_release) = Select::new("Select a version:", releases)
        .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
        .prompt_skippable()?
    else {
        return Ok(None);
    };

    eprintln!("Downloading version: {}", chosen_release);

    let path = chosen_release
        .download_release(exe_dir, chosen_release.to_filename()?)
        .await?;

    Ok(Some(path))
}

/// Selects the specific version.
///
/// Version must be in `major.minor.patch` format.
pub async fn select_version(exe_dir: &PathBuf, tag: &str) -> eyre::Result<PathBuf> {
    let releases = get_compute_releases().await?;

    let chosen_release = releases
        .iter()
        .find(|release| release.version() == tag)
        .ok_or_else(|| eyre::eyre!("No release found for tag: {}", tag))?;

    eprintln!("Downloading version: {}", chosen_release);

    let path = chosen_release
        .download_release(exe_dir, chosen_release.to_filename()?)
        .await?;

    Ok(path)
}
