use inquire::Select;

use crate::DriaEnv;

pub fn edit_api_keys(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    loop {
        // choose an API key name
        let Some(chosen_api_key) =
            Select::new("Select an API key to change:", DriaApiKeyKind::all())
                .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
                .prompt_skippable()?
        else {
            break;
        };

        // edit the API key
        let Some(new_value) = inquire::Text::new("Enter the new value:")
            .with_default(dria_env.get(chosen_api_key.name()).unwrap_or_default())
            .with_help_message("ESC to go back")
            .prompt_skippable()?
        else {
            continue;
        };

        eprintln!("Setting {} to {}", chosen_api_key, new_value);
        dria_env.set(chosen_api_key.name(), new_value);
    }

    Ok(())
}

#[derive(Debug, Clone, enum_iterator::Sequence)]
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
}

impl std::fmt::Display for DriaApiKeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
