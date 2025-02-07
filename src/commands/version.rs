use inquire::Select;

use crate::utils::get_compute_releases;

pub async fn change_version() -> eyre::Result<()> {
    let releases = get_compute_releases().await?;

    let Some(chosen_release) = Select::new("Select a version:", releases)
        .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
        .prompt_skippable()?
    else {
        return Ok(());
    };

    println!("Chosen version: {}", chosen_release);

    chosen_release.download_release().await?;

    Ok(())
}
