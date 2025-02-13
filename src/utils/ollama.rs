use eyre::{Context, Result};
use std::env;
use std::process::Stdio;
use tokio::process::{Child, Command};
use which::which;

use crate::DriaEnv;

const OLLAMA_RETRY_COUNT: usize = 10;
const OLLAMA_RETRY_INTERVAL_MILLIS: u64 = 500;

/// Spawns a local Ollama server process at the given host and port.
///
/// ## Arguments
/// - `dria_env`: The environment variables to use for the Ollama process.
///
/// ## Returns
/// A `Child` process handle to the spawned Ollama process.
///
/// ## Errors
/// - If the Ollama executable is not found in the system.
pub async fn spawn_ollama(dria_env: &DriaEnv) -> Result<Child> {
    let (host, port) = dria_env.get_ollama_values();

    // find the path to binary
    let exe_path = which("ollama").wrap_err("could not find Ollama executable")?;

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
    eprintln!("Waiting for Ollama to start");
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
    let (host, port) = dria_env.get_ollama_values();

    match reqwest::get(&format!("{}:{}", host, port)).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "require Ollama"]
    async fn test_ollama_spawn_and_check() {
        let mut dria_env = DriaEnv::new();
        dria_env.set("OLLAMA_HOST", "http://127.0.0.1");
        dria_env.set("OLLAMA_PORT", "11438"); // not default!
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
