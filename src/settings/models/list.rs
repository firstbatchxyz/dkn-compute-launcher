use crate::DriaEnv;

/// List all chosen models.
pub fn list_models(dria_env: &DriaEnv) {
    let models = dria_env.get_model_config().get_model_names();
    if models.is_empty() {
        eprintln!("No models configured.");
    } else {
        eprintln!("Configured models:\n - {}", models.join("\n - "));
    }
}
