// Ollama client implementation with NDJSON streaming support
// Endpoint: http://localhost:11434/api/chat

use super::client::{AIProvider, CompletionRequest};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct OllamaClient {
    base_url: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    message: Option<OllamaMessageContent>,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageContent {
    content: String,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: Client::new(),
        }
    }

    /// Build messages from CompletionRequest
    fn build_messages(&self, req: &CompletionRequest) -> Vec<OllamaMessage> {
        let mut messages = Vec::new();

        // System prompt
        messages.push(OllamaMessage {
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

            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: format!("Context:\n{}", context),
            });
        }

        // Shell history if provided
        if !req.history.is_empty() {
            let history = req.history.join("\n");
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: format!("Recent shell history:\n{}", history),
            });
        }

        // User query
        messages.push(OllamaMessage {
            role: "user".to_string(),
            content: req.user_query.clone(),
        });

        messages
    }

    /// Stream completion from Ollama API
    async fn stream_impl(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        let messages = self.build_messages(&req);

        let ollama_req = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
        };

        let endpoint = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&endpoint)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Ollama: {}. Is Ollama running?", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Ollama API error (status {}): {}",
                status,
                error_text
            );
        }

        // Convert byte stream to NDJSON token stream
        let stream = response.bytes_stream().map(|result| {
            result
                .map_err(|e| anyhow::anyhow!("Stream read error: {}", e))
                .and_then(|bytes| {
                    // Parse NDJSON - each line is a separate JSON object
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();

                    for line in text.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<OllamaStreamChunk>(line) {
                            Ok(chunk) => {
                                if let Some(msg) = chunk.message {
                                    if !msg.content.is_empty() {
                                        tokens.push(msg.content);
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to parse NDJSON: {}", e));
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
impl AIProvider for OllamaClient {
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
            OllamaMessage {
                role: "system".to_string(),
                content: "You are AETHER in Sentinel mode. Analyze error messages and suggest fixes. Be concise and actionable.".to_string(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: format!("This command failed:\n{}\n\nSuggest a fix.", error_log),
            },
        ];

        let ollama_req = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false, // Non-streaming for sentinel mode
        };

        let endpoint = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&endpoint)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error (status {}): {}", status, error_text);
        }

        // For non-streaming, Ollama returns a single JSON object
        let chunk: OllamaStreamChunk = response.json().await?;

        Ok(chunk
            .message
            .map(|m| m.content)
            .unwrap_or_else(|| "No response from model".to_string()))
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
