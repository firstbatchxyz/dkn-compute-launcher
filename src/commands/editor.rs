use inquire::Editor;
use std::fs;
use std::path::PathBuf;

use crate::utils::DriaEnv;

/// Edit the environment file at the given path.
pub fn edit_environment_file(env_path: &PathBuf) -> eyre::Result<()> {
    let existing_env_content = if env_path.exists() {
        fs::read_to_string(env_path)?
    } else {
        log::warn!(
            "Environment file not found at: {}, will create a new one on save!",
            env_path.display()
        );

        DriaEnv::EXAMPLE_ENV.to_string()
    };

    let prompt = format!("Edit {} file:", env_path.display());
    let Some(new_env_content) = Editor::new(&prompt)
        .with_predefined_text(&existing_env_content)
        .with_help_message("ESC to go back")
        .prompt_skippable()?
    else {
        return Ok(());
    };

    if existing_env_content != new_env_content {
        fs::write(env_path, new_env_content)?;
        log::info!("Environment file updated successfully.");
    } else {
        log::info!("No changes made to the file.");
    }

    Ok(())
}
