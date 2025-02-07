use inquire::Select;

use crate::DriaEnv;

const LOG_LEVELS_KEY: &str = "RUST_LOG";

pub fn edit_log_level(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    // the log levels are stored within `RUST_LOG` as used by `env_logger`.

    let mut is_changed = false;

    // `env_logger` uses a comma-separated list of `module=level` pairs
    // see: https://docs.rs/env_logger/latest/env_logger/#enabling-logging
    let mut log_levels = dria_env
        .get(LOG_LEVELS_KEY)
        .unwrap_or_default()
        .split(",")
        .collect::<Vec<&str>>();

    println!("Existing log levels: {:?}", log_levels);

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

        is_changed = true;

        println!("{} -> {}", module, choice.as_rust_log())
        // TODO: !!!
        // update log levels
        // match module {
        //     LogModules::Compute => {

        //     },
        //     LogModules::LibP2P => {

        //     },
        //     LogModules::Workflows => {
        //     },
        // };
    }

    if is_changed {
        let new_log_levels = log_levels.join(",");

        dria_env.set(LOG_LEVELS_KEY, &new_log_levels);
    } else {
        println!("No changes made.");
    }

    Ok(())
}

/// An enum to represent modules that we care about logging as a w
#[derive(Debug, Clone, enum_iterator::Sequence)]
enum LogModules {
    /// All `dkn` modules
    Compute,
    /// All `libp2p` modules
    LibP2P,
    /// Ollama workflows
    Workflows,
}

impl LogModules {
    #[inline]
    pub fn all() -> Vec<Self> {
        enum_iterator::all::<Self>().collect()
    }
}

// display impl for `inquire`
impl std::fmt::Display for LogModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compute => write!(f, "Compute Node modules"),
            Self::LibP2P => write!(f, "Libp2p modules"),
            Self::Workflows => write!(f, "Workflow modules"),
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
            Self::Error => write!(f, "Error: Very serious errors"),
            Self::Warn => write!(f, "Warn: Hazardous situations"),
            Self::Info => write!(f, "Info: Useful information"),
            Self::Debug => write!(f, "Debug: Debug-level information"),
            Self::Trace => write!(f, "Trace: Very low priority information"),
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

        edit_log_level(&mut env).unwrap();
    }
}
