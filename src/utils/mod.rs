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

/// The launcher version, taken from the `Cargo.toml` file of the running binary.
pub const DKN_LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The latest compute node will always be at this file for a chosen directory.
pub const DKN_LATEST_COMPUTE_FILENAME: &str = "dkn-compute-node_latest";

/// The filename for the version tracker file, simply stores the string for the version.
pub const DKN_VERSION_TRACKER_FILENAME: &str = ".dkn-compute-version";

/// Progress bar (indicatif) template for download progress.
pub const PROGRESS_BAR_TEMPLATE: &str =
    "[{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} ({eta}) {msg}";

/// Progress bar characters for download progress.
pub const PROGRESS_BAR_CHARS: &str = "=>-";
