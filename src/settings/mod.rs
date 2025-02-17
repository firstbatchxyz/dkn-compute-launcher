mod models;
pub use models::edit_models;

mod apikey;
pub use apikey::{edit_api_keys, DriaApiKeyKind};

mod wallet;
pub use wallet::edit_wallet;

mod port;
pub use port::edit_port;

mod ollama;
pub use ollama::edit_ollama;

mod loglevel;
pub use loglevel::edit_log_level;

/// Compute node setting commands.
#[derive(Debug, Clone, enum_iterator::Sequence)]
pub enum Settings {
    /// Configure your wallet (secret key).
    Wallet,
    /// Configure the selected port.
    Port,
    /// Configure the selected models.
    Models,
    /// Configure Ollama settings.
    Ollama,
    /// Configure your API Keys.
    ApiKeys,
    /// Configure log-levels.
    LogLevels,
    /// Quit settings menu.
    SaveExit,
}

impl Settings {
    #[inline]
    pub fn all() -> Vec<Self> {
        enum_iterator::all::<Self>().collect()
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wallet => write!(f, "Wallet"),
            Self::Port => write!(f, "Port"),
            Self::Models => write!(f, "Models"),
            Self::Ollama => write!(f, "Ollama"),
            Self::ApiKeys => write!(f, "API Keys"),
            Self::LogLevels => write!(f, "Log Levels"),
            Self::SaveExit => write!(f, "Save & Exit"),
        }
    }
}
