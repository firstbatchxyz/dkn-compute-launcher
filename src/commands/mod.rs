use clap::Subcommand;
use colored::Colorize;
use std::path::PathBuf;

mod start;
pub use start::run_compute;

mod editor;
pub use editor::edit_environment_file;

mod settings;
pub use settings::change_settings;

mod specific;
pub use specific::download_specific_release;

mod update;
pub use update::update;

mod measure;
pub use measure::measure_tps;

mod setup;
pub use setup::setup_environment;

/// Launcher commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Change node settings: models, api keys, network settings.
    Settings,
    /// Setup the environment file from scratch (will overwrite existing values).
    Setup,
    /// Open a command-line text editor for your environment file (advanced).
    EnvEditor,
    /// Measure performance (TPS) of Ollama models on your machine.
    Measure,
    /// Start the latest compute node
    Start {
        /// Directory where the executables are stored.
        #[arg(long, default_value = default_exedir())]
        exedir: PathBuf,
    },
    /// Manually update the compute node & launcher.
    Update {
        /// Directory where the executables are stored.
        #[arg(long, default_value = default_exedir())]
        exedir: PathBuf,
    },
    /// Run a specific compute node version.
    Specific {
        /// Directory where the executables are stored.
        #[arg(long, default_value = default_exedir())]
        exedir: PathBuf,
        /// Run the chosen executable immediately.
        #[arg(short, long, default_value_t = false)]
        run: bool,
        /// Tag of the version to download, bypasses the prompt if provided.
        #[arg(short, long, value_parser = parse_version_tag)]
        tag: Option<String>,
    },
}

/// Parses a version tag in the format `major.minor.patch`.
fn parse_version_tag(s: &str) -> Result<String, String> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return Err("Version must be in format 'major.minor.patch'".to_string());
    }

    for (idx, part) in parts.iter().enumerate() {
        if part.parse::<u32>().is_err() {
            return Err(format!(
                "{} version must be a non-negative integer",
                match idx {
                    0 => "Major".bold(),
                    1 => "Minor".bold(),
                    2 => "Patch".bold(),
                    _ => unreachable!(),
                }
            ));
        }
    }

    Ok(s.to_string())
}

/// Returns the default targeted environment file.
///
/// - On Unix systems, this is `~/.dria/dkn-compute-launcher/.env`.
/// - On Windows systems, this is `%USERPROFILE%\.dria\compute\.env`.
///
/// If there is an error, it will return just `.env`.
#[inline]
pub fn default_env() -> String {
    let env_filename = ".env.default".to_string();

    match homedir::my_home() {
        Ok(Some(home)) => home
            .join(".dria")
            .join("dkn-compute-launcher")
            .join(&env_filename)
            .into_os_string()
            .into_string()
            .unwrap_or(env_filename),
        Ok(None) | Err(_) => env_filename,
    }
}

/// Returns the default executables directory.
#[inline]
pub fn default_exedir() -> String {
    match homedir::my_home() {
        Ok(Some(home)) => home
            .join(".dria")
            .join("dkn-compute-launcher")
            .into_os_string()
            .into_string()
            .unwrap_or(".".to_string()),
        Ok(None) | Err(_) => ".".to_string(),
    }
}
