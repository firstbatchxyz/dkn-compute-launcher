use inquire::Select;

use crate::{utils::Selectable, DriaEnv};

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
        let Selectable::Some(module) = Select::new(
            "Select a module to change log level:",
            Selectable::new(LogModules::all()),
        )
        .with_help_message("↑↓ to move, ENTER to select")
        .prompt()?
        else {
            break;
        };

        // find existing log level for this module
        let existing_log_level = log_levels
            .iter()
            .find(|level| level.starts_with(&format!("{}=", module.as_rust_log())))
            .and_then(|level| level.split('=').nth(1)) // get rhs
            .unwrap_or(LogLevels::Off.as_rust_log());

        // find starting cursor based on existing level
        let starting_cursor = LogLevels::all()
            .iter()
            .position(|level| level.as_rust_log() == existing_log_level)
            .unwrap_or(0);

        // choose a log level
        let Selectable::Some(choice) =
            Select::new("Choose log level:", Selectable::new(LogLevels::all()))
                .with_help_message("↑↓ to move, ENTER to select")
                .with_starting_cursor(starting_cursor)
                .prompt()?
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
        log::info!("No changes made.");
    }

    Ok(())
}

/// An enum to represent modules that we care about logging
#[derive(Debug, Clone, enum_iterator::Sequence)]
enum LogModules {
    DknComputeNode,
    DknP2P,
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
            Self::DknComputeNode => "dkn_compute",
            Self::DknP2P => "dkn_p2p",
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
            Self::DknComputeNode => write!(f, "Dria Compute Node: Core"),
            Self::DknP2P => write!(f, "Dria Compute Node: P2P"),
            Self::DknWorkflows => write!(f, "Dria Compute Node: Workflows"),
            Self::Libp2p => write!(f, "Low-level Lib2p Modules"),
            Self::OllamaWorkflows => write!(f, "Ollama Workflows"),
        }
    }
}

#[derive(Debug, Clone, enum_iterator::Sequence)]
enum LogLevels {
    Off = 0,
    Error,
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
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl std::fmt::Display for LogLevels {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevels::Off =>   write!(f, "off    no logging"),
            LogLevels::Error => write!(f, "error  very serious errors"),
            LogLevels::Warn =>  write!(f, "warn   hazardous situations"),
            LogLevels::Info =>  write!(f, "info   useful information"),
            LogLevels::Debug => write!(f, "debug  debug-level information"),
            LogLevels::Trace => write!(f, "trace  low-level information"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "run manually"]
    fn test_log_level_editor() {
        let mut env = DriaEnv::new_from_env();
        env.set(LOG_LEVELS_KEY, "dkn_compute=info,dkn_compute_launcher=info");
        eprintln!("Old log levels: {:?}", env.get(LOG_LEVELS_KEY).unwrap());
        edit_log_level(&mut env).unwrap();
        eprintln!("New log levels: {:?}", env.get(LOG_LEVELS_KEY).unwrap());
    }
}
