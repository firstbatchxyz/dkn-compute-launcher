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

    /// Path to the .env file.
    #[arg(short, long, default_value = commands::default_env())]
    pub env: PathBuf,

    /// Enable debug-level logs
    #[arg(short, long)]
    pub debug: bool,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // default commands such as version and help exit at this point
    let cli = Cli::parse();

    // read env w.r.t cli argument
    let dotenv_result = dotenvy::from_path(&cli.env);

    // init env logger
    let log_level = match cli.debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Seconds))
        .filter(None, log::LevelFilter::Off)
        .filter_module("dkn_compute_launcher", log_level)
        .parse_default_env()
        .init();

    // log about env usage after env logger init is executed
    match dotenv_result {
        Ok(_) => log::info!("Loaded env file at: {}", cli.env.display()),
        Err(_) => {
            log::warn!("No env file found at {}", cli.env.display());
            log::info!(
                "Creating a new environment to be saved at {}",
                cli.env.display()
            );
            commands::setup_environment(&cli.env)?;

            // early-exit if the user wanted to setup anyways
            if let Commands::Setup = cli.command {
                return Ok(());
            } else {
                // override the env file with the new one
                dotenvy::from_path_override(&cli.env)?;
            }
        }
    }

    // get the directory w.r.t env file, which will be used for the executable's directory
    // when a given path is relative, the parent may be empty; this is handled by checking
    // if the underlying `OsStr` is empty or not, in which case the fallback is given by
    // the `std::env::current_dir` function.
    let exe_dir = cli
        .env
        .parent()
        .map(|dir| dir.to_owned())
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| std::env::current_dir().expect("could not get current directory"));

    match &cli.command {
        Commands::Settings => commands::change_settings(&cli.env).await?,
        Commands::Setup => commands::setup_environment(&cli.env)?,
        Commands::Points => commands::show_points().await?,
        Commands::EnvEditor => commands::edit_environment_file(&cli.env)?,
        Commands::Uninstall => commands::uninstall_launcher(&exe_dir, &cli.env).await?,
        Commands::Info => commands::show_info(),
        Commands::Update => commands::update(&exe_dir).await,
        Commands::Specific { run, tag } => {
            // downloads the specific version under the `exedir`, with the filename including the version tag
            // e.g. `./my/dir/dkn-compute-node_v0.3.6`
            let exe_path = commands::download_specific_release(&exe_dir, tag.as_ref()).await?;

            // if `run` is true, the binary is executed immediately
            if *run {
                commands::run_compute(&exe_path, false)
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
            let exe = exe_dir.join(DKN_LATEST_COMPUTE_FILE);

            commands::run_compute(&exe, true)
                .await?
                .monitor_process()
                .await;
        }
        Commands::Referrals => commands::handle_referrals().await?,
        Commands::Check => commands::check_command().await?,
    };

    Ok(())
}
