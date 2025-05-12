use dkn_workflows::{Model, ModelProvider};
use inquire::{MultiSelect, Select};

use crate::{utils::Selectable, DriaEnv};

/// Edit the chosen models.
pub fn edit_models(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    let mut is_changed = false;

    // choose a provider
    let mut chosen_models = dria_env.get_model_config().models.to_vec();
    loop {
        let Selectable::Some(provider) = Select::new(
            "Select a model provider:",
            Selectable::new(ModelProvider::all().collect()),
        )
        .with_help_message("↑↓ to move, type to filter provider, ENTER to select")
        .prompt()?
        else {
            if chosen_models.is_empty() {
                log::error!("You must choose at least 1 model!");
                continue;
            } else {
                break;
            }
        };

        // then choose models that belong to this provider
        let my_prov_models = chosen_models
            .iter()
            .filter(|m| ModelProvider::from(*m) == provider)
            .cloned()
            .collect::<Vec<_>>();
        let all_prov_models = Model::all_with_provider(&provider).collect::<Vec<_>>();
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

        let  selected_prov_models = MultiSelect::new(
          "Choose your models with SPACE, then press ENTER:",
          all_prov_models.clone(),
      )
      .with_default(&default_selected_idxs)
      .with_help_message(
          "↑↓ to move, SPACE to select one, ←/→ to select all/none, type to filter models, ENTER to confirm"
      )
      .prompt()?;

        is_changed = true;

        // remove all provider models from the chosen models
        chosen_models.retain(|m| ModelProvider::from(m) != provider);

        // and then extend the chosen models with the selected models
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

        log::info!("Chosen models:\n - {}", new_models.join("\n - "));
        dria_env.set(DriaEnv::DKN_MODELS_KEY, new_models.join(","));
    } else {
        log::info!("No changes made.");
    }

    Ok(())
}
