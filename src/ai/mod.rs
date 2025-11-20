pub mod client;
pub mod ollama;
pub mod prompts;

// To be implemented in Phase 3
// pub mod openai;
// pub mod anthropic;
// pub mod gemini;

pub use client::{AIProvider, CompletionRequest, ModelType};
