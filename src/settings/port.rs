use inquire::{validator::Validation, Text};

use crate::DriaEnv;

const LISTEN_ADDR_KEY: &str = "DKN_P2P_LISTEN_ADDR";
const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/4001";

pub fn edit_port(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    // get existing address
    let addr = &dria_env.get(LISTEN_ADDR_KEY).unwrap_or(DEFAULT_LISTEN_ADDR);

    // ensure the address starts with `/ip4/0.0.0.0/tcp/` and ends with a number
    let mut parts = addr.split('/').collect::<Vec<_>>();
    if parts[1] != "ip4" || parts[2] != "0.0.0.0" || parts[3] != "tcp" {
        return Err(eyre::eyre!(
            "The listen address must start with \"/ip4/0.0.0.0/tcp\"."
        ));
    }
    let port = parts[4].parse::<u16>().unwrap();

    let validator = |port_str: &str| match port_str.parse::<u16>() {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            "Port must be a valid 16-bit unsigned integer.".into(),
        )),
    };

    let Some(new_port) = Text::new("Enter port:")
        .with_help_message(&format!("ESC to go back and keep using {}", port))
        .with_validator(validator)
        .prompt_skippable()?
    else {
        return Ok(());
    };

    parts[4] = &new_port;
    let new_listen_addr = parts.join("/");
    log::info!("New listen address: {:?}", new_listen_addr);
    dria_env.set(LISTEN_ADDR_KEY, new_listen_addr);

    Ok(())
}
