mod start;
pub use start::run_compute_node;

mod editor;
pub use editor::edit_environment_file;

mod settings;
pub use settings::change_settings;

mod specific;
pub use specific::download_specific_release;

mod update;
pub use update::update;

mod setup;
pub use setup::setup_environment;

mod info;
pub use info::show_info;

mod referrals;
pub use referrals::handle_referrals;

mod uninstall;
pub use uninstall::uninstall_launcher;

mod points;
pub use points::show_points;

/// Launcher commands.
#[derive(clap::Subcommand)]
pub enum Commands {
    /// Change node settings: models, api keys, network settings.
    Settings,
    /// Setup the environment file from scratch (will overwrite existing values).
    Setup,
    /// Start the latest compute node
    Start,
    /// Generate or enter a referral code.
    Referrals,
    /// Show your $DRIA points.
    Points,
    /// Uninstall the launcher & its files.
    Uninstall,
    /// Show information about the current environment.
    Info,
    /// Manually update the compute node & launcher.
    Update,
    /// Run a specific compute node version.
    Specific {
        /// Run the chosen executable immediately.
        #[arg(long, default_value_t = false)]
        run: bool,
        /// Tag of the version to download, bypasses the prompt if provided.
        #[arg(long)]
        tag: Option<String>,
    },
    /// Open a command-line text editor for your environment file (advanced).
    EnvEditor,
}

/// Returns the default targeted environment file.
///
/// In **release mode**:
/// - On Unix systems, this is `$HOME/.dria/dkn-compute-launcher/.env`.
/// - On Windows systems, this is `%USERPROFILE%\.dria\compute\.env`.
///
/// In **debug mode**, this is just `.env` in the current directory.
///
/// If there is an error, it will also just return `.env`.
///
/// Its important to name the file `.env` all the time due to how compute node reads it.
#[inline]
pub fn default_env() -> String {
    let env_filename = ".env".to_string();

    if cfg!(debug_assertions) {
        env_filename
    } else {
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
}
