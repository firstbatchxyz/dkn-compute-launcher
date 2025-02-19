use clap::Subcommand;
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
        #[arg(short, long)]
        tag: Option<String>,
    },
}

/// Returns the default targeted environment file.
///
/// - On Unix systems, this is `~/.dria/compute/.env`.
/// - On Windows systems, this is `%USERPROFILE%\.dria\compute\.env`.
///
/// If there is an error, it will return just `.env`.
#[inline]
pub fn default_env() -> String {
    ".env".to_string()

    // TODO: do the thing below for profile management
    // let fallback_path = ".env.default".to_string();
    // match homedir::my_home() {
    //     Ok(Some(home)) => home
    //         .join(".dria")
    //         .join("compute")
    //         .join(".env.default")
    //         .into_os_string()
    //         .into_string()
    //         .unwrap_or(fallback_path),
    //     Ok(None) | Err(_) => fallback_path,
    // }
}

/// Returns the default executables directory.
#[inline]
pub fn default_exedir() -> &'static str {
    "."
}
