use std::collections::HashSet;

use dkn_workflows::ModelProvider;
use inquire::{error::InquireResult, Select};

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

        dria_env.set(chosen_api_key.name(), new_value);
    }

    Ok(())
}

#[derive(Debug, Clone, enum_iterator::Sequence, Hash, Eq, PartialEq)]
pub enum DriaApiKeyKind {
    OpenAI,
    Gemini,
    OpenRouter,
    Serper,
    Jina,
}

impl DriaApiKeyKind {
    #[inline]
    pub fn all() -> Vec<DriaApiKeyKind> {
        enum_iterator::all::<DriaApiKeyKind>().collect()
    }

    /// Returns the name of the environment variable that stores the API key.
    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Gemini => "GEMINI_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
            Self::Serper => "SERPER_API_KEY",
            Self::Jina => "JINA_API_KEY",
        }
    }

    /// Given a list of providers (can contain duplicates) returns the unique set of API key kinds.
    pub fn from_providers(providers: &[ModelProvider]) -> Vec<DriaApiKeyKind> {
        let set: HashSet<DriaApiKeyKind> =
            HashSet::from_iter(providers.iter().filter_map(|provider| match provider {
                ModelProvider::OpenAI => Some(DriaApiKeyKind::OpenAI),
                ModelProvider::Gemini => Some(DriaApiKeyKind::Gemini),
                ModelProvider::OpenRouter => Some(DriaApiKeyKind::OpenRouter),
                ModelProvider::Ollama => None,
                ModelProvider::VLLM => None,
            }));

        set.into_iter().collect()
    }

    /// A wrapper for `inquire::Text` for prompting the user to enter the API key.
    #[inline]
    pub fn prompt_api(&self, dria_env: &DriaEnv) -> InquireResult<String> {
        inquire::Text::new(&format!("Enter your {}:", self.name()))
            .with_default(dria_env.get(self.name()).unwrap_or_default())
            .with_help_message("ENTER without typing anything to keep using the existing value")
            .prompt()
    }
}

impl std::fmt::Display for DriaApiKeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
