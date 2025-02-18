use clap::Parser;
use std::path::PathBuf;

mod commands;
use commands::Commands;

mod settings;

mod utils;
use utils::*;

pub const DKN_LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    // read env w.r.t cli argument, defaults to `.env`
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
            }
        }
    }

    match &cli.command {
        Commands::Settings => commands::change_settings(&cli.env)?,
        Commands::Setup => commands::setup_environment(&cli.env)?,
        Commands::EnvEditor => commands::edit_environment_file(&cli.env)?,
        Commands::Bench => commands::run_benchmarks().await?,
        Commands::Version { exedir, run, tag } => {
            // get the executable path
            let exe = if let Some(tag) = tag {
                Some(commands::select_version(exedir, tag).await?)
            } else {
                commands::change_version(exedir).await?
            };

            // run the downloaded executable optionally
            if let (Some(exe), true) = (exe, *run) {
                commands::run_compute(&exe, false)
                    .await?
                    .monitor_process()
                    .await;
            }
        }
        Commands::Start { exedir } => {
            commands::run_compute(exedir, true)
                .await?
                .monitor_process()
                .await;
        }
        Commands::Update { exedir } => {
            commands::update(exedir).await?;
        }
    };

    Ok(())
}
