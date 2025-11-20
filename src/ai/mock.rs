// Mock AI provider for deterministic testing
use super::client::{AIProvider, CompletionRequest};
use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use std::collections::HashMap;

/// Mock AI provider that returns deterministic responses
/// Perfect for E2E testing without requiring a live LLM
pub struct MockAIProvider {
    /// Predefined responses for specific queries
    responses: HashMap<String, String>,
    /// Default response if query not found
    default_response: String,
}

impl MockAIProvider {
    pub fn new() -> Self {
        let mut responses = HashMap::new();

        // Predefined test cases
        responses.insert(
            "list files".to_string(),
            "ls -la".to_string(),
        );
        responses.insert(
            "find python files".to_string(),
            "find . -name '*.py'".to_string(),
        );
        responses.insert(
            "undo last git commit".to_string(),
            "git reset --soft HEAD~1".to_string(),
        );
        responses.insert(
            "undo last 3 git commits".to_string(),
            "git reset --soft HEAD~3".to_string(),
        );
        responses.insert(
            "show git status".to_string(),
            "git status".to_string(),
        );
        responses.insert(
            "find all python files modified yesterday and tar them".to_string(),
            "find . -name '*.py' -mtime -1 | xargs tar -cvf archive.tar".to_string(),
        );
        responses.insert(
            "delete all log files".to_string(),
            "find . -name '*.log' -delete".to_string(),
        );

        Self {
            responses,
            default_response: "echo 'Command not found in mock responses'".to_string(),
        }
    }

    /// Add a custom response for testing
    pub fn with_response(mut self, query: String, response: String) -> Self {
        self.responses.insert(query, response);
        self
    }

    /// Set default response
    pub fn with_default(mut self, response: String) -> Self {
        self.default_response = response;
        self
    }

    /// Get the response for a query (case-insensitive matching)
    fn get_response(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();

        // Try exact match first
        if let Some(response) = self.responses.get(&query_lower) {
            return response.clone();
        }

        // Try fuzzy match (contains)
        for (key, value) in &self.responses {
            if query_lower.contains(key) || key.contains(&query_lower) {
                return value.clone();
            }
        }

        // Return default
        self.default_response.clone()
    }
}

impl Default for MockAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProvider for MockAIProvider {
    async fn stream_completion(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, Result<String, anyhow::Error>>> {
        let response = self.get_response(&req.user_query);

        // Simulate streaming by breaking the response into chunks
        let chunks: Vec<String> = response
            .chars()
            .collect::<Vec<_>>()
            .chunks(5)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect();

        let stream = stream::iter(chunks.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    async fn get_fix_suggestion(&self, _error_log: String) -> anyhow::Result<String> {
        Ok("Mock fix suggestion: Check your configuration file".to_string())
    }

    fn model_name(&self) -> &str {
        "mock-provider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_exact_match() {
        let provider = MockAIProvider::new();
        assert_eq!(provider.get_response("list files"), "ls -la");
    }

    #[test]
    fn test_mock_provider_fuzzy_match() {
        let provider = MockAIProvider::new();
        let response = provider.get_response("how do I list files?");
        assert_eq!(response, "ls -la");
    }

    #[test]
    fn test_mock_provider_custom_response() {
        let provider = MockAIProvider::new()
            .with_response("test query".to_string(), "test response".to_string());
        assert_eq!(provider.get_response("test query"), "test response");
    }

    #[tokio::test]
    async fn test_mock_provider_streaming() {
        let provider = MockAIProvider::new();
        let req = CompletionRequest {
            system_prompt: "test".to_string(),
            user_query: "list files".to_string(),
            context_files: vec![],
            history: vec![],
        };

        let mut stream = provider.stream_completion(req).await.unwrap();
        use futures::StreamExt;

        let mut collected = String::new();
        while let Some(chunk) = stream.next().await {
            collected.push_str(&chunk.unwrap());
        }

        assert_eq!(collected, "ls -la");
    }
}
