use crate::utils::DriaEnv;

/// Show information about the current environment.
pub fn show_info() {
    let dria_env = DriaEnv::new_from_env();
    log::info!(
        "Wallet: {}",
        dria_env.get("DKN_WALLET_SECRET_KEY").unwrap_or("<none>")
    );
    // TODO: add public key and address here
    log::info!(
        "Log Levels: {}",
        dria_env.get("RUST_LOG").unwrap_or("<none>")
    );

    log::info!(
        "Models: {}",
        dria_env
            .get_model_config()
            .models
            .iter()
            .map(|(_, m)| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
