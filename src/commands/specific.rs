use eyre::{eyre, Result};
use inquire::Select;
use std::path::{Path, PathBuf};

use crate::{
    get_releases,
    utils::{DriaRelease, DriaRepository},
};

/// Prompts the user to select a version to download, which is downloaded to `exe_dir` directory.
///
/// ### Arguments
/// - `exe_dir`: directory where the binary is located
/// - `tag`: optional tag to download directly
///
/// ### Returns
/// Path to the downloaded binary.
///
/// ### Errors
/// - If the `exe_dir` is not a directory
/// - If the release could not be downloaded
/// - If the release could not be found for the given tag
/// - If the user cancels the prompt
pub async fn download_specific_release(exe_dir: &Path, tag: Option<&String>) -> Result<PathBuf> {
    if !exe_dir.is_dir() {
        return Err(eyre!("{} must be a directory", exe_dir.display()));
    }

    let releases = get_releases(DriaRepository::ComputeNode).await?;

    // filter out non-well formed releases, all release should be like `vX.Y.Z`,
    // this is done so that launcher doesnt clutter the prompt with non-release versions
    let releases = releases.into_iter().collect::<Vec<_>>();

    let chosen_release = match tag {
        // choose the tag directly
        Some(tag) => releases
            .into_iter()
            .find(|release| release.version() == tag)
            .ok_or_else(|| eyre::eyre!("No release found for tag: {}", tag))?,
        // prompt the user for selection
        None => Select::new(
            "Choose a version and press ENTER:",
            releases
                .into_iter()
                .filter(|release: &DriaRelease| {
                    // we only want releases that are well formed
                    let parts = release.version().split('.').collect::<Vec<_>>();

                    parts.len() == 3
                        && parts[0].parse::<u32>().is_ok()
                        && parts[1].parse::<u32>().is_ok()
                        && parts[2].parse::<u32>().is_ok()
                })
                .collect::<Vec<_>>(),
        )
        .with_help_message("↑↓ to move, type to filter by name, ENTER to select")
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
