use dkn_workflows::{Model, ModelProvider};
use inquire::{MultiSelect, Select};

use crate::DriaEnv;

const MODELS_KEY: &str = "DKN_MODELS";

pub fn edit_models(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    let mut is_changed = false;

    // TODO: can remove models_config perhaps?
    let models_config = dria_env.get_model_config();

    let mut chosen_models = models_config
        .models
        .iter()
        .map(|(_, m)| m.clone())
        .collect::<Vec<_>>();

    // choose a provider
    loop {
        let Some(provider) =
            Select::new("Select a model provider:", ModelProvider::all().collect())
                .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
                .prompt_skippable()?
        else {
            break;
        };

        // then choose a model of that provider
        let my_prov_models = chosen_models
            .iter()
            .cloned()
            .filter(|m| ModelProvider::from(m) == provider)
            .collect::<Vec<_>>();
        let all_prov_models = Model::all_with_provider(provider).collect::<Vec<_>>();
        let default_selected_idxs = all_prov_models
            .iter()
            .enumerate()
            .filter_map(|(idx, model)| {
                if my_prov_models.contains(model) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let Some(mut selected_prov_models) = MultiSelect::new(
            "Choose the models that you would like to serve:",
            all_prov_models,
        )
        .with_default(&default_selected_idxs)
        .with_help_message(
            "↑↓ to move, space to select one, → to all, ← to none, type to filter, ESC to go back",
        )
        .prompt_skippable()?
        else {
            continue;
        };

        is_changed = true;

        // update the chosen models
        // those that exist in chosen_models but not in selected_prov_models are removed (via retain)
        chosen_models.retain(|m| !selected_prov_models.contains(m));

        // those that exist in selected_prov_models but not in chosen_models are added
        selected_prov_models.retain(|m| !chosen_models.contains(m));
        chosen_models.extend(selected_prov_models);
    }

    if is_changed {
        // save models
        let mut new_models = chosen_models
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>();

        // sort by model name so that they are easier to choose
        new_models.sort();

        eprintln!("Chosen models:\n - {}", new_models.join("\n - "));
        dria_env.set(MODELS_KEY, new_models.join(","));
    } else {
        eprintln!("No changes made.");
    }

    Ok(())
}
