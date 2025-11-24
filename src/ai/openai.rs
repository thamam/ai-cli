// OpenAI client implementation with SSE streaming support
// Endpoint: https://api.openai.com/v1/chat/completions
// Requires: AETHER_OPENAI_API_KEY environment variable

use super::client::{AIProvider, CompletionRequest};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct OpenAIClient {
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    delta: OpenAIDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompletionResponse {
    choices: Vec<OpenAICompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompletionChoice {
    message: OpenAICompletionMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAICompletionMessage {
    content: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }

    /// Create from environment variable AETHER_OPENAI_API_KEY
    pub fn from_env(model: String) -> anyhow::Result<Self> {
        let api_key = std::env::var("AETHER_OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("AETHER_OPENAI_API_KEY not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Build messages from CompletionRequest
    fn build_messages(&self, req: &CompletionRequest) -> Vec<OpenAIMessage> {
        let mut messages = Vec::new();

        // System prompt
        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: req.system_prompt.clone(),
        });

        // Context files if provided
        if !req.context_files.is_empty() {
            let context = req
                .context_files
                .iter()
                .map(|(name, content)| format!("File: {}\n```\n{}\n```", name, content))
                .collect::<Vec<_>>()
                .join("\n\n");

            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: format!("Context:\n{}", context),
            });
        }

        // Shell history if provided
        if !req.history.is_empty() {
            let history = req.history.join("\n");
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: format!("Recent shell history:\n{}", history),
            });
        }

        // User query
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: req.user_query.clone(),
        });

        messages
    }

    /// Stream completion from OpenAI API
    async fn stream_impl(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        let messages = self.build_messages(&req);

        let openai_req = OpenAIChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to connect to OpenAI: {}. Check your internet connection and API key.", e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "OpenAI API error (status {}): {}. Check your AETHER_OPENAI_API_KEY.",
                status,
                error_text
            );
        }

        // Convert byte stream to SSE token stream
        // OpenAI uses Server-Sent Events format: "data: {...}\n\n"
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
                            // "[DONE]" signals end of stream
                            if json_str.trim() == "[DONE]" {
                                continue;
                            }

                            match serde_json::from_str::<OpenAIStreamChunk>(json_str) {
                                Ok(chunk) => {
                                    for choice in chunk.choices {
                                        if let Some(content) = choice.delta.content {
                                            if !content.is_empty() {
                                                tokens.push(content);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(anyhow::anyhow!("Failed to parse SSE: {}", e));
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
impl AIProvider for OpenAIClient {
    async fn stream_completion(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, Result<String, anyhow::Error>>> {
        let stream = self.stream_impl(req).await?;
        Ok(Box::pin(stream))
    }

    async fn get_fix_suggestion(&self, error_log: String) -> anyhow::Result<String> {
        // Non-streaming request for Sentinel mode
        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are AETHER in Sentinel mode. Analyze error messages and suggest fixes. Be concise and actionable.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: format!("This command failed:\n{}\n\nSuggest a fix.", error_log),
            },
        ];

        let openai_req = OpenAIChatRequest {
            model: self.model.clone(),
            messages,
            stream: false, // Non-streaming for sentinel mode
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to OpenAI: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error (status {}): {}", status, error_text);
        }

        // For non-streaming, OpenAI returns standard completion format
        let completion: OpenAICompletionResponse = response.json().await?;

        Ok(completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "No response from model".to_string()))
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
