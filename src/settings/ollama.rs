use inquire::{validator::Validation, Text};
use reqwest::Url;

use crate::DriaEnv;

const OLLAMA_HOST_KEY: &str = "OLLAMA_HOST";
const OLLAMA_PORT_KEY: &str = "OLLAMA_PORT";
const DEFAULT_OLLAMA_HOST: &str = "http://localhost";
const DEFAULT_OLLAMA_PORT: &str = "11434";

pub fn edit_ollama(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    // asks for a change on both host and port
    if let Some(new_host) = Text::new("Enter host:")
        .with_help_message("ESC to change port instead")
        .with_default(dria_env.get(OLLAMA_HOST_KEY).unwrap_or(DEFAULT_OLLAMA_HOST))
        .with_validator(|host_str: &str| match Url::parse(host_str) {
            Ok(_) => Ok(Validation::Valid),
            Err(err) => Ok(Validation::Invalid(
                format!("Host must be a valid URL: {}", err).into(),
            )),
        })
        .prompt_skippable()?
    {
        dria_env.set(OLLAMA_HOST_KEY, new_host);
    }

    if let Some(new_port) = Text::new("Enter port:")
        .with_help_message("ESC to go back")
        .with_default(dria_env.get(OLLAMA_PORT_KEY).unwrap_or(DEFAULT_OLLAMA_PORT))
        .with_validator(|port_str: &str| match port_str.parse::<u16>() {
            Ok(_) => Ok(Validation::Valid),
            Err(_) => Ok(Validation::Invalid(
                "Port must be a valid 16-bit unsigned integer.".into(),
            )),
        })
        .prompt_skippable()?
    {
        dria_env.set(OLLAMA_PORT_KEY, new_port);
    };

    Ok(())
}
