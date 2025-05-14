mod releases;
pub use releases::*;

mod ollama;
pub use ollama::*;

mod env;
pub use env::*;

mod process;
pub use process::*;

mod updates;
pub use updates::*;

mod selectable;
pub use selectable::*;

pub mod referrals;

pub mod crypto;

mod signal;
pub use signal::*;

mod fdlimit;
pub use fdlimit::configure_fdlimit;

/// The launcher version, taken from the `Cargo.toml` file of the running binary.
pub const DKN_LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The latest compute node will always be at this file for a chosen directory.
#[cfg(unix)]
pub const DKN_LATEST_COMPUTE_FILE: &str = "dkn-compute-node_latest";
#[cfg(windows)]
pub const DKN_LATEST_COMPUTE_FILE: &str = "dkn-compute-node_latest.exe";

/// The filename for the version tracker file, simply stores the string for the version.
pub const DKN_VERSION_TRACKER_FILE: &str = ".dkn-compute-version";

/// Progress bar (indicatif) template for download progress.
pub const PROGRESS_BAR_TEMPLATE: &str =
    "[{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} ({eta}) {msg}";

/// Progress bar characters for download progress.
pub const PROGRESS_BAR_CHARS: &str = "=>-";

/// `UserAgent` header value for the launcher, used for HTTP requests.
pub const LAUNCHER_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
