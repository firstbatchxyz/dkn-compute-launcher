use eyre::{Context, Result};
use std::env;
use std::process::Stdio;
use tokio::process::{Child, Command};
use which::which;

use crate::DriaEnv;

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
    let (host, port) = dria_env.ollama_values();

    // find the path to binary
    let exe_path = which("ollama").wrap_err("could not find Ollama executable")?;

    log::debug!("Using Ollama executable at {:?}", exe_path);

    // ollama requires the OLLAMA_HOST environment variable to be set before launching
    let old_var = env::var("OLLAMA_HOST").ok();
    env::set_var("OLLAMA_HOST", format!("{}:{}", host, port));
    let command = Command::new(exe_path)
        .arg("serve")
        .stdout(Stdio::null()) // ignore the output for simplicity
        .spawn()
        .wrap_err("could not spawn Ollama")?;

    // restore old variable
    if let Some(val) = old_var {
        env::set_var("OLLAMA_HOST", val);
    } else {
        env::remove_var("OLLAMA_HOST");
    }

    // TODO: wait for server to start

    Ok(command)
}

/// Checks if ollama is running at the configured host & port, returns `true` if it is.
///
/// Ollama responds to a GET request at its root with "Ollama is running".
pub async fn check_ollama(dria_env: &DriaEnv) -> bool {
    let (host, port) = dria_env.ollama_values();

    match reqwest::get(&format!("{}:{}", host, port)).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_run() {
        let mut dria_env = DriaEnv::new();
        dria_env.set("OLLAMA_HOST", "http://127.0.0.1");
        dria_env.set("OLLAMA_PORT", "11438"); // not default!
        let mut child = spawn_ollama(&dria_env).await.unwrap();

        // wait for 10 seconds
        println!("Waiting for 10 seconds...");
        sleep(Duration::from_secs(10)).await;

        // kill the process
        if let Err(e) = child.kill().await {
            log::error!("Failed to kill Ollama process: {}", e);
        } else {
            log::info!("Ollama process killed.");
        }
    }
}
