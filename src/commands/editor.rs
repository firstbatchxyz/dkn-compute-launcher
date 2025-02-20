use eyre::{eyre, Result};
use inquire::Editor;
use std::fs;
use std::path::PathBuf;

/// Edit the environment file at the given path.
///
/// ### Arguments
/// - `env_path`: path to the environment file
///
/// ### Errors
/// - If the environment file does not exist
/// - If the file could not be read
pub fn edit_environment_file(env_path: &PathBuf) -> Result<()> {
    if !env_path.exists() {
        return Err(eyre!(
            "Environment file does not exist: {}",
            env_path.display()
        ));
    }

    let Ok(existing_env_content) = fs::read_to_string(env_path) else {
        return Err(eyre::eyre!("Could not read {}", env_path.display()));
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
