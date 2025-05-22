use dkn_executor::ModelProvider;
use inquire::{error::InquireResult, Select};
use std::collections::HashSet;

use crate::{utils::Selectable, DriaEnv};

pub fn edit_api_keys(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    loop {
        // choose an API key name
        let Selectable::Some(chosen_api_key) = Select::new(
            "Select an API key to change:",
            Selectable::new(DriaApiKeyKind::all()),
        )
        .with_help_message("↑↓ to move, ENTER to select, type to filter")
        .prompt()?
        else {
            break;
        };

        // edit the API key
        let new_value = chosen_api_key.prompt_api(dria_env)?;

        // empty value is ignored immediately
        if new_value.is_empty() {
            continue;
        };

        // delete the API key
        if new_value.eq_ignore_ascii_case("delete") {
            dria_env.set(chosen_api_key.name(), "");
            continue;
        }

        dria_env.set(chosen_api_key.name(), new_value);
    }

    Ok(())
}

#[derive(Debug, Clone, enum_iterator::Sequence, Hash, Eq, PartialEq)]
pub enum DriaApiKeyKind {
    OpenAI,
    Gemini,
    OpenRouter,
}

impl DriaApiKeyKind {
    #[inline]
    pub fn all() -> Vec<DriaApiKeyKind> {
        enum_iterator::all::<DriaApiKeyKind>().collect()
    }

    /// Returns the name of the environment variable that stores the API key.
    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenAI => DriaEnv::OPENAI_APIKEY_KEY,
            Self::Gemini => DriaEnv::GEMINI_APIKEY_KEY,
            Self::OpenRouter => DriaEnv::OPENROUTER_APIKEY_KEY,
        }
    }

    /// Returns a help message for the API key, e.g. where to get it from.
    pub fn help_message(&self) -> &'static str {
        match self {
            Self::OpenAI => "Get yours at https://platform.openai.com/api-keys",
            Self::Gemini => "Get yours at https://aistudio.google.com/app/apikey",
            Self::OpenRouter => "Get yours at https://openrouter.ai/keys",
        }
    }

    /// Given a list of providers (can contain duplicates) returns the unique set of API key kinds.
    pub fn from_providers(
        providers: impl Iterator<Item = ModelProvider>,
    ) -> impl Iterator<Item = Self> {
        let set: HashSet<Self> =
            HashSet::from_iter(providers.filter_map(|provider| match provider {
                ModelProvider::OpenAI => Some(Self::OpenAI),
                ModelProvider::Gemini => Some(Self::Gemini),
                ModelProvider::OpenRouter => Some(Self::OpenRouter),
                _ => None,
            }));

        set.into_iter()
    }

    /// A wrapper for `inquire::Text` for prompting the user to enter the API key.
    #[inline]
    pub fn prompt_api(&self, dria_env: &DriaEnv) -> InquireResult<String> {
        inquire::Text::new(&format!("Enter your {}:", self.name()))
            .with_default(dria_env.get(self.name()).unwrap_or_default())
            .with_help_message(&format!(
                "{} | type 'delete' to remove the API key",
                self.help_message()
            ))
            .prompt()
    }
}

impl std::fmt::Display for DriaApiKeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
