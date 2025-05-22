use inquire::Confirm;
use std::path::Path;

use crate::utils::DKN_VERSION_TRACKER_FILE;

/// Uninstalls the launcher and its environment file, along with the compute node binaries & its version tracker.
///
/// ### Arguments
/// - `env_dir`: directory where the compute node binaries are located
/// - `env_path`: path to the environment file
/// - `backup_path`: optional path to the backup the env file
///
/// We normally expect `env_path` to be a continuation of `env_dir`, but it is passed separately because we may not know
/// which particular environment file is used within that directory.
///
/// ### Errors
/// - If the environment file could not be removed
/// - If the compute node binaries could not be removed
/// - If the version tracker exists but could not be removed
/// - If the launcher itself could not be removed
///
/// ### Notes
/// - The user is asked for confirmation before uninstalling.
pub async fn uninstall_launcher(
    env_dir: &Path,
    env_path: &Path,
    backup_path: Option<&Path>,
) -> eyre::Result<()> {
    let launcher_path = std::env::current_exe()?;

    // provide a help message to prompt the user to backup their env file
    // if the backup path is not given
    let help_message = if let Some(backup_path) = backup_path {
        format!(
            "{} will be saved to {}",
            env_path.display(),
            backup_path.display()
        )
    } else {
        "Make sure you have backed up your secret key within the environment file!".to_string()
    };

    // ask for confirmation
    let answer =
        Confirm::new(&format!(
          "Are you sure you want to uninstall the launcher \"{}\", env \"{}\" and all related files within \"{}\"? (y/n)",
          launcher_path.display(),
          env_path.display(),
          env_dir.display(),
        ))
            .with_help_message(help_message.as_str())
            .prompt()?;

    if !answer {
        log::info!("Aborting, you can still use the launcher :)");
        return Ok(());
    } else {
        log::info!("Uninstalling the launcher");
    }

    // remove the compute node binaries within the directory
    log::info!(
        "Removing compute node binaries within: {}",
        env_dir.display()
    );
    for path in std::fs::read_dir(env_dir)?.flatten().map(|e| e.path()) {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("dkn-compute-node") {
                log::info!("Removing: {}", path.display());
                std::fs::remove_file(&path)?;
            }
        }
    }

    // remove version tracker
    let version_tracker = env_dir.join(DKN_VERSION_TRACKER_FILE);
    if version_tracker.exists() {
        log::info!("Removing version tracker: {}", version_tracker.display());
        std::fs::remove_file(&version_tracker)?;
    }

    // remove the executable with `self_replace`
    log::info!("Removing the launcher itself: {}", launcher_path.display());
    self_update::self_replace::self_delete()?;

    // remove .env file within the directory
    if env_path.exists() {
        // if there is a backup path, copy the env file to it
        if let Some(backup_path) = backup_path {
            log::info!(
                "Backing up the environment file to: {}",
                backup_path.display()
            );
            std::fs::copy(env_path, backup_path)?;
        }
        log::info!("Removing environment file: {}", env_path.display());
        std::fs::remove_file(env_path)?;
    }

    Ok(())
}
