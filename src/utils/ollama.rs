use eyre::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use ollama_rs::{error::OllamaError, Ollama};
use std::env;
use std::process::Stdio;
use tokio::process::{Child, Command};
use which::which;

use crate::DriaEnv;

use super::{PROGRESS_BAR_CHARS, PROGRESS_BAR_TEMPLATE};

const OLLAMA_RETRY_COUNT: usize = 10;
const OLLAMA_RETRY_INTERVAL_MILLIS: u64 = 500;

/// Spawns a local Ollama server process at the given host and port.
///
/// ### Arguments
/// - `dria_env`: The environment variables to use for the Ollama process.
///
/// ### Returns
/// A `Child` process handle to the spawned Ollama process.
///
/// ### Errors
/// - If the Ollama executable is not found in the system.
pub async fn spawn_ollama(dria_env: &DriaEnv) -> Result<Child> {
    let (host, port) = dria_env.get_ollama_config();

    // find the path to binary
    let exe_path = which("ollama").wrap_err(
        "could not find Ollama executable, please install it from https://ollama.com/download",
    )?;

    log::debug!("Using Ollama executable at {:?}", exe_path);

    // ollama requires the OLLAMA_HOST environment variable to be set before launching
    let old_var = env::var("OLLAMA_HOST").ok();
    env::set_var("OLLAMA_HOST", format!("{}:{}", host, port));
    let command = Command::new(exe_path)
        .arg("serve")
        .stdout(Stdio::null()) // ignored
        .stderr(Stdio::null()) // ignored
        .spawn()
        .wrap_err("could not spawn Ollama")?;

    // restore old variable
    if let Some(val) = old_var {
        env::set_var("OLLAMA_HOST", val);
    } else {
        env::remove_var("OLLAMA_HOST");
    }

    // check ollama to see if its running
    log::info!("Waiting for Ollama to start");
    for _ in 0..OLLAMA_RETRY_COUNT {
        if check_ollama(dria_env).await {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(
            OLLAMA_RETRY_INTERVAL_MILLIS,
        ))
        .await;
    }
    if !check_ollama(dria_env).await {
        return Err(eyre::eyre!(
            "Ollama failed to start after {} retries",
            OLLAMA_RETRY_COUNT
        ));
    }

    Ok(command)
}

/// Checks if ollama is running at the configured host & port, returns `true` if it is.
///
/// Ollama responds to a GET request at its root with "Ollama is running".
pub async fn check_ollama(dria_env: &DriaEnv) -> bool {
    let (host, port) = dria_env.get_ollama_config();

    match reqwest::get(&format!("{}:{}", host, port)).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

pub async fn pull_model_with_progress(ollama: &Ollama, model_name: String) -> Result<()> {
    let mut pull_stream = ollama.pull_model_stream(model_name.clone(), false).await?;
    let mut pull_error: Option<OllamaError> = None;
    let mut pull_bar: Option<ProgressBar> = None;
    while let Some(status) = pull_stream.next().await {
        match status {
            Ok(status) => {
                // if there is a bar & status, log it
                if let Some(ref pb) = pull_bar {
                    if let Some(completed) = status.completed {
                        pb.set_position(completed);
                    }
                } else
                // otherwise try to create bar
                if let Some(total) = status.total {
                    pull_bar = Some(
                        ProgressBar::new(total)
                            .with_message(format!("Pulling {}", model_name))
                            .with_style(
                                ProgressStyle::default_bar()
                                    .template(PROGRESS_BAR_TEMPLATE)?
                                    .progress_chars(PROGRESS_BAR_CHARS),
                            ),
                    );
                }
            }
            Err(err) => {
                pull_error = Some(err);
                break;
            }
        }
    }

    if let Some(err) = pull_error {
        log::error!("Failed to pull model {}: {:?}", model_name, err);
        // no need to care about `pull_bar` here, it will be dropped
    } else if let Some(pb) = pull_bar {
        pb.finish_with_message(format!("{} pull complete.", model_name));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Ollama"]
    async fn test_ollama_spawn_and_check() {
        let mut dria_env = DriaEnv::new_from_env();
        dria_env.set("OLLAMA_HOST", "http://127.0.0.1");
        dria_env.set("OLLAMA_PORT", "11438"); // not the default port!
        let mut child = spawn_ollama(&dria_env).await.unwrap();

        // check for healthiness
        assert!(check_ollama(&dria_env).await, "ollama is not healthy");

        // kill the process
        if let Err(e) = child.kill().await {
            log::error!("Failed to kill Ollama process: {}", e);
        } else {
            log::info!("Ollama process killed.");
        }
    }
}
