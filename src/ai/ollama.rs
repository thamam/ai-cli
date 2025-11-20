// Ollama client implementation (Phase 3)
// Will implement in Phase 3 with full streaming support

use super::client::{AIProvider, CompletionRequest};
use async_trait::async_trait;
use futures::stream::BoxStream;

pub struct OllamaClient {
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self { base_url, model }
    }
}

#[async_trait]
impl AIProvider for OllamaClient {
    async fn stream_completion(
        &self,
        _req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, Result<String, anyhow::Error>>> {
        // TODO: Implement in Phase 3
        anyhow::bail!("Ollama client not yet implemented")
    }

    async fn get_fix_suggestion(&self, _error_log: String) -> anyhow::Result<String> {
        // TODO: Implement in Phase 3
        anyhow::bail!("Ollama client not yet implemented")
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
