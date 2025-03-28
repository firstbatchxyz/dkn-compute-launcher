use inquire::{validator::Validation, Text};

use crate::DriaEnv;

const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/4001";

pub fn edit_port(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    // get existing address
    let addr = &dria_env
        .get(DriaEnv::DKN_P2P_LISTEN_ADDR_KEY)
        .unwrap_or(DEFAULT_LISTEN_ADDR);

    // ensure the address starts with `/ip4/0.0.0.0/tcp/` and ends with a number
    let mut parts = addr.split('/').collect::<Vec<_>>();
    if parts[1] != "ip4" || parts[2] != "0.0.0.0" || parts[3] != "tcp" {
        return Err(eyre::eyre!(
            "The listen address must start with \"/ip4/0.0.0.0/tcp\"."
        ));
    }
    let port = parts[4].parse::<u16>().unwrap();

    // validate the port
    let validator = |port_str: &str| match port_str.parse::<u16>() {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            "Port must be a valid 16-bit unsigned integer.".into(),
        )),
    };

    let existing_port_str = port.to_string();
    let new_port = Text::new("Enter compute node port:")
        .with_validator(validator)
        .with_default(&existing_port_str)
        .with_help_message("Enter 0 to use a random port everytime")
        .prompt()?;

    if new_port != existing_port_str {
        // update the port in the address
        parts[4] = &new_port;
        let new_listen_addr = parts.join("/");
        log::info!("New listen address: {:?}", new_listen_addr);
        dria_env.set(DriaEnv::DKN_P2P_LISTEN_ADDR_KEY, new_listen_addr);
    }

    Ok(())
}
