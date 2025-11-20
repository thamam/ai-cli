// Library exports for AETHER
// Makes modules available for testing and potential library use

pub mod ai;
pub mod context;
pub mod core;
pub mod tui;

// Re-export commonly used types
pub use ai::{AIProvider, CompletionRequest, MockAIProvider};
pub use core::{Brain, CommandAnalysis, Config};
pub use context::{FileScanner, ShellContext};
