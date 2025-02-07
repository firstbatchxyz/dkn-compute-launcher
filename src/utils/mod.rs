mod update;

mod releases;
pub use releases::*;

mod ollama;
pub use ollama::spawn_ollama;
