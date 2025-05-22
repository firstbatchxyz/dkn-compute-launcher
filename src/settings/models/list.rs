use crate::DriaEnv;

/// List all chosen models.
pub fn list_models(dria_env: &DriaEnv) {
    let models = dria_env.get_models();
    if models.is_empty() {
        eprintln!("No models configured.");
    } else {
        eprintln!(
            "Configured models:\n - {}",
            models
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("\n - ")
        );
    }
}
