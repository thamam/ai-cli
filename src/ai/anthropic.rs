// Anthropic (Claude) client implementation with SSE streaming support
// Endpoint: https://api.anthropic.com/v1/messages
// Requires: AETHER_ANTHROPIC_API_KEY environment variable

use super::client::{AIProvider, CompletionRequest};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct AnthropicClient {
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicMessagesRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }

    /// Create from environment variable AETHER_ANTHROPIC_API_KEY
    pub fn from_env(model: String) -> anyhow::Result<Self> {
        let api_key = std::env::var("AETHER_ANTHROPIC_API_KEY")
            .map_err(|_| anyhow::anyhow!("AETHER_ANTHROPIC_API_KEY not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Build messages from CompletionRequest
    fn build_messages_and_system(&self, req: &CompletionRequest) -> (Vec<AnthropicMessage>, String) {
        let mut messages = Vec::new();
        let mut system_parts = vec![req.system_prompt.clone()];

        // Context files if provided
        if !req.context_files.is_empty() {
            let context = req
                .context_files
                .iter()
                .map(|(name, content)| format!("File: {}\n```\n{}\n```", name, content))
                .collect::<Vec<_>>()
                .join("\n\n");
            system_parts.push(format!("Context:\n{}", context));
        }

        // Shell history if provided
        if !req.history.is_empty() {
            let history = req.history.join("\n");
            system_parts.push(format!("Recent shell history:\n{}", history));
        }

        let system_prompt = system_parts.join("\n\n");

        // User query - Anthropic requires alternating user/assistant messages
        messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: req.user_query.clone(),
        });

        (messages, system_prompt)
    }

    /// Stream completion from Anthropic API
    async fn stream_impl(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        let (messages, system_prompt) = self.build_messages_and_system(&req);

        let anthropic_req = AnthropicMessagesRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 4096,
            system: Some(system_prompt),
            stream: true,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_req)
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to connect to Anthropic: {}. Check your internet connection and API key.", e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Anthropic API error (status {}): {}. Check your AETHER_ANTHROPIC_API_KEY.",
                status,
                error_text
            );
        }

        // Convert byte stream to SSE token stream
        // Anthropic uses Server-Sent Events format with event types
        let stream = response.bytes_stream().map(|result| {
            result
                .map_err(|e| anyhow::anyhow!("Stream read error: {}", e))
                .and_then(|bytes| {
                    // Parse SSE format
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();

                    for line in text.lines() {
                        // SSE format: "data: {...}"
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<AnthropicStreamEvent>(json_str) {
                                Ok(event) => {
                                    // Look for content_block_delta events with text
                                    if event.event_type == "content_block_delta" {
                                        if let Some(delta) = event.delta {
                                            if let Some(text) = delta.text {
                                                if !text.is_empty() {
                                                    tokens.push(text);
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // Ignore parse errors for non-delta events
                                    continue;
                                }
                            }
                        }
                    }

                    Ok(tokens.join(""))
                })
        });

        Ok(Box::pin(stream))
    }
}

#[async_trait]
impl AIProvider for AnthropicClient {
    async fn stream_completion(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, Result<String, anyhow::Error>>> {
        let stream = self.stream_impl(req).await?;
        Ok(Box::pin(stream))
    }

    async fn get_fix_suggestion(&self, error_log: String) -> anyhow::Result<String> {
        // Non-streaming request for Sentinel mode
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: format!("This command failed:\n{}\n\nSuggest a fix.", error_log),
        }];

        let anthropic_req = AnthropicMessagesRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 4096,
            system: Some("You are AETHER in Sentinel mode. Analyze error messages and suggest fixes. Be concise and actionable.".to_string()),
            stream: false,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Anthropic: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error (status {}): {}", status, error_text);
        }

        // For non-streaming, Anthropic returns messages response
        let anthropic_resp: AnthropicResponse = response.json().await?;

        Ok(anthropic_resp
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_else(|| "No response from model".to_string()))
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
