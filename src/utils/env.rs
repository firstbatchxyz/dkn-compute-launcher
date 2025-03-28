use std::{collections::HashMap, io, path::Path};

use dkn_workflows::DriaWorkflowsConfig;
use eyre::OptionExt;

use crate::settings;

use super::crypto::secret_key_to_account;

#[derive(Debug, Clone)]
pub struct DriaEnv {
    kv: HashMap<&'static str, String>,
    is_changed: bool,
}

impl DriaEnv {
    // log-level key
    pub const LOG_LEVEL_KEY: &'static str = "RUST_LOG";

    // dkn stuff
    pub const DKN_WALLET_KEY: &'static str = "DKN_WALLET_SECRET_KEY";
    pub const DKN_MODELS_KEY: &'static str = "DKN_MODELS";
    pub const DKN_P2P_LISTEN_ADDR_KEY: &'static str = "DKN_P2P_LISTEN_ADDR";
    pub const DKN_BATCH_SIZE_KEY: &'static str = "DKN_BATCH_SIZE";

    // ollama stuff
    pub const OLLAMA_HOST_KEY: &str = "OLLAMA_HOST";
    pub const OLLAMA_PORT_KEY: &str = "OLLAMA_PORT";
    pub const OLLAMA_AUTO_PULL_KEY: &str = "OLLAMA_AUTO_PULL";

    // api keys
    pub const OPENAI_APIKEY_KEY: &'static str = "OPENAI_API_KEY";
    pub const GEMINI_APIKEY_KEY: &'static str = "GEMINI_API_KEY";
    pub const OPENROUTER_APIKEY_KEY: &'static str = "OPENROUTER_API_KEY";
    pub const SERPER_APIKEY_KEY: &'static str = "SERPER_API_KEY";
    pub const JINA_APIKEY_KEY: &'static str = "JINA_API_KEY";

    /// All environment keys that we are interested in.
    pub const KEY_NAMES: [&str; 13] = [
        // log level
        Self::LOG_LEVEL_KEY,
        // DKN
        Self::DKN_WALLET_KEY,
        Self::DKN_MODELS_KEY,
        Self::DKN_P2P_LISTEN_ADDR_KEY,
        Self::DKN_BATCH_SIZE_KEY,
        // API keys
        Self::OPENAI_APIKEY_KEY,
        Self::GEMINI_APIKEY_KEY,
        Self::OPENROUTER_APIKEY_KEY,
        Self::SERPER_APIKEY_KEY,
        Self::JINA_APIKEY_KEY,
        // Ollama
        Self::OLLAMA_HOST_KEY,
        Self::OLLAMA_PORT_KEY,
        Self::OLLAMA_AUTO_PULL_KEY,
    ];

    /// Check if the environment has been changed.
    #[inline]
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    /// Get the value of a key.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.kv.get(key).map(|s| s.as_str())
    }

    /// Set the value of a key, and mark the environment as changed.
    #[inline]
    pub fn set(&mut self, key: &'static str, value: impl ToString) {
        self.kv.insert(key, value.to_string());

        // we should not set this to `false` anywhere
        self.is_changed = true;
    }

    /// Create a new instance of `DriaEnv` with the environment variables specified in `KEY_NAMES`.
    ///
    /// - Non-existent variables are ignored.
    /// - Empty variables are ignored.
    pub fn new_from_env() -> Self {
        Self {
            kv: HashMap::from_iter(
                Self::KEY_NAMES
                    .into_iter()
                    // read env vars & keep existing ones
                    .filter_map(|k| std::env::var(k).map(|v| (k, v)).ok())
                    // remove empty values
                    .filter(|(_, v)| !v.is_empty()),
            ),
            is_changed: false,
        }
    }

    /// Expects a content string (from an env file) and saves the keys to this content.
    ///
    /// - If a key exists in the content, it will be replaced with the value from the env.
    /// - If multiple keys exists for the same key name, only the last & uncommented one will be used.
    /// - If a key does not exist in the content, it will be appended to the end of the content.
    pub fn save_to_content(&self, content: &str) -> String {
        let mut ans_lines = Vec::<String>::new();
        let mut kv_to_add = self.kv.clone();

        for lines in content.lines() {
            // get keys via `iter_mut` because we cant remove them otherwise
            if let Some(matched_key) = kv_to_add
                .iter_mut()
                .map(|(k, _)| *k)
                .find(|k| lines.starts_with(&format!("{}=", k)))
            {
                // replace the line with the new value
                ans_lines.push(format!(
                    "{}={}",
                    matched_key,
                    kv_to_add.remove(matched_key).unwrap()
                ));
            } else {
                // ignore this line by adding it as is
                ans_lines.push(lines.to_string());
            }
        }

        for (k, v) in &kv_to_add {
            ans_lines.push(format!("{}={}", k, v));
        }

        ans_lines.join("\n")
    }

    /// Saves the environment to a file by adding the changes.
    pub fn save_to_file(&self, env_path: &Path) -> io::Result<()> {
        log::info!("Saving changes to {}", env_path.display());

        let content = std::fs::read_to_string(env_path)?;
        let new_content = self.save_to_content(&content);

        std::fs::write(env_path, new_content)?;
        log::info!("Changes saved successfully.");
        Ok(())
    }

    pub fn new_default_file(env_path: &Path) -> io::Result<()> {
        // example env file content, used for creating a new env file.
        const BASE_ENV_FILE_CONTENT: &str = include_str!("../../.env.example");

        // create directories if they dont exist
        if !env_path.exists() {
            if let Some(dir) = env_path.parent() {
                std::fs::create_dir_all(dir)?;
            }
        }

        std::fs::write(env_path, BASE_ENV_FILE_CONTENT)
    }

    /// Asks for a secret key for the wallet if it does not exist in the environment.
    pub fn ask_for_key_if_required(&mut self) -> eyre::Result<()> {
        if self.get(DriaEnv::DKN_WALLET_KEY).is_none() {
            log::info!("Provide a secret key of your wallet.");
            settings::edit_wallet(self, false)?;
        }

        Ok(())
    }

    /// Returns the `host` and `port` values for the Ollama server w.r.t Dria environment.
    #[inline]
    pub fn get_ollama_config(&self) -> (&str, u16) {
        const DEFAULT_OLLAMA_HOST: &str = "http://127.0.0.1";
        const DEFAULT_OLLAMA_PORT: &str = "11434";

        let host = self
            .get(Self::OLLAMA_HOST_KEY)
            .unwrap_or(DEFAULT_OLLAMA_HOST);

        let port = self
            .get(Self::OLLAMA_PORT_KEY)
            .unwrap_or(DEFAULT_OLLAMA_PORT);

        (host, port.parse().expect("invalid port"))
    }

    /// Returns the model config with the chosen models.
    #[inline]
    pub fn get_model_config(&self) -> DriaWorkflowsConfig {
        DriaWorkflowsConfig::new_from_csv(self.get(Self::DKN_MODELS_KEY).unwrap_or_default())
    }

    /// Parses the wallet secret key to a [`libsecp256k1::SecretKey`], and returns it
    /// along with the [`libsecp256k1::PublicKey`] and its address.
    #[inline]
    pub fn get_account(
        &self,
    ) -> eyre::Result<(libsecp256k1::SecretKey, libsecp256k1::PublicKey, String)> {
        let secret_key = self
            .get(DriaEnv::DKN_WALLET_KEY)
            .ok_or_eyre("No wallet secret key found.")?;

        secret_key_to_account(secret_key)
    }
}

impl std::fmt::Display for DriaEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in &self.kv {
            writeln!(f, "{}={}", k, v)?;
        }
        Ok(())
    }
}
