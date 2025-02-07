use inquire::Select;

use crate::DriaEnv;

pub fn edit_log_level(dria_env: &mut DriaEnv) -> eyre::Result<()> {
    // the log levels are stored within `RUST_LOG` as used by `env_logger`.
    const LOG_LEVELS_KEY: &str = "RUST_LOG";

    let existing_log_levels = dria_env.get(LOG_LEVELS_KEY).unwrap_or_default();
    let mut is_changed = false;

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
            .with_help_message("↑↓ to move, enter to select, type to filter, ESC to quit")
            .prompt_skippable()?
        else {
            continue;
        };

        is_changed = true;

        println!("{} -> {}", module, choice)
        // save to RUST_LOG string
        // TODO: !!!
    }

    if is_changed {
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
            Self::Debug => write!(f, "Debug: Lower priority information"),
            Self::Trace => write!(
                f,
                "Trace: Very low priority, often extremely verbose, information"
            ),
        }
    }
}
