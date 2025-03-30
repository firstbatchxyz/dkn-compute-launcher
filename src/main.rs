use clap::Parser;
use std::path::PathBuf;

mod commands;
use commands::Commands;

mod settings;

mod utils;
use utils::*;

// https://docs.rs/clap/latest/clap/_derive/
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"), version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the .env file
    #[arg(short, long, default_value = commands::default_env())]
    env: PathBuf,

    /// Profile name for the environment file
    #[arg(short, long, value_parser = parse_profile)]
    profile: Option<String>,
}

/// Ensures that the profile name contains only alphanumeric characters, '-', or '_'.
fn parse_profile(profile: &str) -> eyre::Result<String> {
    if !profile
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(eyre::eyre!(
            "Profile name must contain only alphanumeric characters, '-', or '_'"
        ));
    }

    Ok(profile.to_string())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // default commands such as version and help exit at this point
    let cli = Cli::parse();

    // env is given by the path, and must be a file
    let mut env_path = cli.env;
    if !env_path.is_file() {
        return Err(eyre::eyre!(
            "env path must be a file: {}",
            env_path.display()
        ));
    }

    // `.<profile>` is appended to the path if given
    if let Some(profile) = cli.profile {
        // we expect this to work because the path is checked to be a file
        let existing_file_name = env_path.file_name().unwrap().to_str().unwrap();
        env_path.set_file_name(format!("{existing_file_name}.{profile}"));
    }

    // read env w.r.t cli argument
    let dotenv_result = dotenvy::from_path(&env_path);

    // init env logger
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .filter(None, log::LevelFilter::Off)
        .filter_module("dkn_compute_launcher", log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // log about env usage after env logger init is executed
    match dotenv_result {
        Ok(_) => log::info!("Loaded env file at: {}", env_path.display()),
        Err(_) => {
            log::warn!(
                "No env file found at {}, creating a new one",
                env_path.display()
            );
            DriaEnv::new_default_file(&env_path)?;
        }
    }

    // get the directory w.r.t env file, which will be used for the executable's directory
    // when a given path is relative, the parent may be empty; this is handled by checking
    // if the underlying `OsStr` is empty or not, in which case the fallback is given by
    // the `std::env::current_dir` function.
    let exe_dir = env_path
        .parent()
        .map(|dir| dir.to_owned())
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| std::env::current_dir().expect("could not get current directory"));

    match &cli.command {
        Commands::Settings => commands::change_settings(&env_path).await?,
        Commands::Setup => commands::setup_environment(&env_path)?,
        Commands::Points => commands::show_points().await?,
        Commands::EnvEditor => commands::edit_environment_file(&env_path)?,
        Commands::Uninstall => commands::uninstall_launcher(&exe_dir, &env_path).await?,
        Commands::Info => commands::show_info(),
        Commands::Update => commands::update(&exe_dir).await,
        Commands::Specific { run, tag } => {
            // downloads the specific version under the `exedir`, with the filename including the version tag
            // e.g. `./my/dir/dkn-compute-node_v0.3.6`
            let exe_path = commands::download_specific_release(&exe_dir, tag.as_ref()).await?;

            // if `run` is true, the binary is executed immediately
            if *run {
                commands::run_compute_node(&exe_path, &env_path, false)
                    .await?
                    .monitor_process()
                    .await;
            } else {
                log::info!("Executable is ready at {}", exe_path.display());
            }
        }
        Commands::Start => {
            // downloads the latest version under the `exedir`, with the filename including "latest"
            // e.g. `./my/dir/dkn-compute-node_latest`
            let exe_path = exe_dir.join(DKN_LATEST_COMPUTE_FILE);

            commands::run_compute_node(&exe_path, &env_path, true)
                .await?
                .monitor_process()
                .await;
        }
        Commands::Referrals => commands::handle_referrals().await?,
    };

    Ok(())
}
