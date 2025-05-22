use inquire::Select;

use crate::{utils::Selectable, DriaEnv};

mod edit;
pub use edit::edit_models; // also used by `setup` command

mod list;
use list::list_models;

mod measure;
use measure::measure_tps;

mod remove;
use remove::remove_local_models;

#[derive(Debug, Clone, enum_iterator::Sequence)]
enum ModelSettings {
    Edit,
    List,
    Remove,
    /// Measure performance (TPS) of Ollama models on your machine.
    Measure,
}

impl ModelSettings {
    #[inline]
    pub fn all() -> Vec<Self> {
        enum_iterator::all::<Self>().collect()
    }
}

impl std::fmt::Display for ModelSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Edit => write!(f, "Edit model selection"),
            Self::List => write!(f, "List chosen models"),
            Self::Remove => write!(f, "Remove local models"),
            Self::Measure => write!(f, "Measure local models"),
        }
    }
}

pub async fn show_model_settings_menu(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    loop {
        let Selectable::Some(choice) = Select::new(
            "Choose model settings:",
            Selectable::new(ModelSettings::all()),
        )
        .with_help_message("↑↓ to move, ENTER to select")
        .prompt()?
        else {
            return Ok(());
        };

        match choice {
            ModelSettings::Edit => {
                edit_models(dria_env)?;
            }
            ModelSettings::List => {
                list_models(dria_env);
            }
            ModelSettings::Remove => {
                remove_local_models(dria_env).await?;
            }
            ModelSettings::Measure => {
                measure_tps(dria_env).await?;
            }
        }
    }
}
