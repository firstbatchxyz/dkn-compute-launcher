use crate::utils::DriaEnv;

/// Show information about the current environment.
pub fn show_info() {
    let dria_env = DriaEnv::new_from_env();

    if let Ok((_, _, addr)) = dria_env.get_account() {
        eprintln!("Address: {}", addr);
    }

    eprintln!(
        "Log Levels: {}",
        dria_env.get("RUST_LOG").unwrap_or("<none>")
    );

    eprintln!(
        "Models: {}",
        dria_env
            .get_model_config()
            .models
            .iter()
            .map(|(_, m)| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
}
