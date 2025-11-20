use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Cloud(String),  // e.g., "gpt-4o"
    Local(String),  // e.g., "llama3"
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub user_query: String,
    pub context_files: Vec<(String, String)>, // (filename, content)
    pub history: Vec<String>,                 // Last N shell commands
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub finish_reason: Option<String>,
}

/// Trait for AI provider implementations
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Stream completion tokens for TUI "Digital Rain" effect
    async fn stream_completion(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, Result<String, anyhow::Error>>>;

    /// Get a fix suggestion for an error log (Sentinel mode)
    async fn get_fix_suggestion(&self, error_log: String) -> anyhow::Result<String>;

    /// Get model information
    fn model_name(&self) -> &str;
}
