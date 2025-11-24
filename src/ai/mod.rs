pub mod anthropic;
pub mod client;
pub mod gemini;
pub mod mock;
pub mod ollama;
pub mod openai;
pub mod prompts;

pub use anthropic::AnthropicClient;
pub use client::{AIProvider, CompletionRequest, ModelType};
pub use gemini::GeminiClient;
pub use mock::MockAIProvider;
pub use ollama::OllamaClient;
pub use openai::OpenAIClient;
