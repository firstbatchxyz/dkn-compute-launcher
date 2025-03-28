use inquire::{validator::Validation, Text};
use reqwest::Url;

use crate::DriaEnv;

/// Prompts the user to edit the Ollama server settings (host & port).
pub fn edit_ollama(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    let (existing_host, existing_port) = dria_env.get_ollama_config();
    let existing_host = existing_host.to_string();
    let existing_port = existing_port.to_string();

    // change host
    let new_host = Text::new("Enter host:")
        .with_default(&existing_host)
        .with_validator(|host_str: &str| match Url::parse(host_str) {
            Ok(_) => Ok(Validation::Valid),
            Err(err) => Ok(Validation::Invalid(
                format!("Host must be a valid URL: {}", err).into(),
            )),
        })
        .prompt()?;
    if new_host != existing_host {
        dria_env.set(DriaEnv::OLLAMA_HOST_KEY, new_host);
    }

    // change port
    let new_port = Text::new("Enter port:")
        .with_default(&existing_port)
        .with_validator(|port_str: &str| match port_str.parse::<u16>() {
            Ok(_) => Ok(Validation::Valid),
            Err(_) => Ok(Validation::Invalid(
                "Port must be a valid 16-bit unsigned integer.".into(),
            )),
        })
        .prompt()?;
    if new_port != existing_host {
        dria_env.set(DriaEnv::OLLAMA_PORT_KEY, new_port);
    }

    Ok(())
}
