use crate::utils::DriaEnv;

/// Show information about the current environment.
pub fn show_info() {
    let dria_env = DriaEnv::new_from_env();

    // wallet
    if let Ok((_, _, addr)) = dria_env.get_account() {
        eprintln!("Address: {}", addr);
    } else {
        eprintln!("Address: no wallet configured!");
    }

    // log levels
    eprintln!(
        "Log Levels: {}",
        dria_env.get(DriaEnv::LOG_LEVEL_KEY).unwrap_or("none")
    );

    // models
    let model_names = dria_env.get_model_config().get_model_names();
    if model_names.is_empty() {
        eprintln!("Models: no models configured!");
    } else {
        eprintln!("Models:\n - {}", model_names.join("\n - "));
    }

    eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
}
