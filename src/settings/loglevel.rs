use inquire::Select;

use crate::DriaEnv;

// the log levels are stored within `RUST_LOG` as used by `env_logger`
const LOG_LEVELS_KEY: &str = "RUST_LOG";

pub fn edit_log_level(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    let mut is_changed = false;

    // `env_logger` uses a comma-separated list of `module=level` pairs
    // see: https://docs.rs/env_logger/latest/env_logger/#enabling-logging
    let mut log_levels: Vec<String> = dria_env
        .get(LOG_LEVELS_KEY)
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty()) // just in case
        .map(String::from)
        .collect();

    loop {
        // choose a module
        let Some(module) = Select::new("Select a module to change log level:", LogModules::all())
            .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
            .prompt_skippable()?
        else {
            break;
        };

        // choose a log level
        let Some(choice) = Select::new("Choose log level:", LogLevels::all())
            .with_help_message("↑↓ to move, enter to select, type to filter, ESC to go back")
            .prompt_skippable()?
        else {
            continue;
        };

        // update module's log-level
        is_changed = true;
        let prefix = format!("{}=", module.as_rust_log());
        let new_level = format!("{}{}", prefix, choice.as_rust_log());
        if let Some(idx) = log_levels
            .iter()
            .position(|level| level.starts_with(&prefix))
        {
            // update existing level
            log_levels[idx] = new_level;
        } else {
            // add new level
            log_levels.push(new_level);
        }
    }

    // save changes
    if is_changed {
        let new_log_levels = log_levels.join(",");
        dria_env.set(LOG_LEVELS_KEY, new_log_levels);
    } else {
        println!("No changes made.");
    }

    Ok(())
}

/// An enum to represent modules that we care about logging as a w
#[derive(Debug, Clone, enum_iterator::Sequence)]
enum LogModules {
    DknCompute,
    Dknp2p,
    DknWorkflows,
    Libp2p,
    OllamaWorkflows,
}

impl LogModules {
    #[inline]
    pub fn all() -> Vec<Self> {
        enum_iterator::all::<Self>().collect()
    }

    pub fn as_rust_log(&self) -> &'static str {
        match self {
            Self::DknCompute => "dkn_compute",
            Self::Dknp2p => "dkn_p2p",
            Self::DknWorkflows => "dkn_workflows",
            Self::Libp2p => "libp2p",
            Self::OllamaWorkflows => "ollama_workflows",
        }
    }
}

// display impl for `inquire`
impl std::fmt::Display for LogModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DknCompute => write!(f, "Dria Compute Node: Core"),
            Self::Dknp2p => write!(f, "Dria Compute Node: P2P"),
            Self::DknWorkflows => write!(f, "Dria Compute Node: Workflows"),
            Self::Libp2p => write!(f, "Low-level Lib2p Modules"),
            Self::OllamaWorkflows => write!(f, "Ollama Workflows"),
        }
    }
}

#[derive(Debug, Clone, enum_iterator::Sequence)]
enum LogLevels {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevels {
    #[inline]
    pub fn all() -> Vec<Self> {
        enum_iterator::all::<Self>().collect()
    }

    pub fn as_rust_log(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl std::fmt::Display for LogLevels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error: very serious errors"),
            Self::Warn => write!(f, "warn: hazardous situations"),
            Self::Info => write!(f, "info: useful information"),
            Self::Debug => write!(f, "debug: debug-level information"),
            Self::Trace => write!(f, "trace: low-level information"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "run manually"]
    fn test_log_level_editor() {
        let mut env = DriaEnv::new();
        env.set(LOG_LEVELS_KEY, "dkn_compute=info,dkn_launcher=info");
        println!("Old log levels: {:?}", env.get(LOG_LEVELS_KEY).unwrap());
        edit_log_level(&mut env).unwrap();
        println!("New log levels: {:?}", env.get(LOG_LEVELS_KEY).unwrap());
    }
}
