use std::path::PathBuf;

use inquire::Select;

use crate::utils::get_compute_releases;

pub async fn change_version(exe_dir: &PathBuf) -> eyre::Result<Option<PathBuf>> {
    let releases = get_compute_releases().await?;

    let Some(chosen_release) = Select::new("Select a version:", releases)
        .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
        .prompt_skippable()?
    else {
        return Ok(None);
    };

    println!("Downloading version: {}", chosen_release);

    let path = chosen_release.download_release(exe_dir).await?;

    Ok(Some(path))
}
