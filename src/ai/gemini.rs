// Google Gemini client implementation with streaming support
// Endpoint: https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent
// Requires: AETHER_GEMINI_API_KEY environment variable

use super::client::{AIProvider, CompletionRequest};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct GeminiClient {
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiStreamResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }

    /// Create from environment variable AETHER_GEMINI_API_KEY
    pub fn from_env(model: String) -> anyhow::Result<Self> {
        let api_key = std::env::var("AETHER_GEMINI_API_KEY")
            .map_err(|_| anyhow::anyhow!("AETHER_GEMINI_API_KEY not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Build request from CompletionRequest
    fn build_request(&self, req: &CompletionRequest) -> GeminiGenerateRequest {
        let mut contents = Vec::new();

        // Build user message with context
        let mut user_text_parts = vec![req.user_query.clone()];

        // Context files if provided
        if !req.context_files.is_empty() {
            let context = req
                .context_files
                .iter()
                .map(|(name, content)| format!("File: {}\n```\n{}\n```", name, content))
                .collect::<Vec<_>>()
                .join("\n\n");
            user_text_parts.insert(0, format!("Context:\n{}", context));
        }

        // Shell history if provided
        if !req.history.is_empty() {
            let history = req.history.join("\n");
            user_text_parts.insert(0, format!("Recent shell history:\n{}", history));
        }

        contents.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: user_text_parts.join("\n\n"),
            }],
        });

        // System instruction (Gemini 1.5+ feature)
        let system_instruction = Some(GeminiContent {
            role: "user".to_string(), // Gemini uses "user" for system instructions
            parts: vec![GeminiPart {
                text: req.system_prompt.clone(),
            }],
        });

        GeminiGenerateRequest {
            contents,
            system_instruction,
        }
    }

    /// Stream completion from Gemini API
    async fn stream_impl(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
        let gemini_req = self.build_request(&req);

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}",
            self.model, self.api_key
        );

        let response = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&gemini_req)
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to connect to Gemini: {}. Check your internet connection and API key.", e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Gemini API error (status {}): {}. Check your AETHER_GEMINI_API_KEY.",
                status,
                error_text
            );
        }

        // Convert byte stream to NDJSON token stream
        // Gemini uses NDJSON format (one JSON object per line)
        let stream = response.bytes_stream().map(|result| {
            result
                .map_err(|e| anyhow::anyhow!("Stream read error: {}", e))
                .and_then(|bytes| {
                    // Parse NDJSON format
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();

                    for line in text.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<GeminiStreamResponse>(line) {
                            Ok(response) => {
                                for candidate in response.candidates {
                                    for part in candidate.content.parts {
                                        if !part.text.is_empty() {
                                            tokens.push(part.text);
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Ignore parse errors for empty/malformed lines
                                continue;
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
impl AIProvider for GeminiClient {
    async fn stream_completion(
        &self,
        req: CompletionRequest,
    ) -> anyhow::Result<BoxStream<'static, Result<String, anyhow::Error>>> {
        let stream = self.stream_impl(req).await?;
        Ok(Box::pin(stream))
    }

    async fn get_fix_suggestion(&self, error_log: String) -> anyhow::Result<String> {
        // Non-streaming request for Sentinel mode
        let gemini_req = GeminiGenerateRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: format!("This command failed:\n{}\n\nSuggest a fix.", error_log),
                }],
            }],
            system_instruction: Some(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: "You are AETHER in Sentinel mode. Analyze error messages and suggest fixes. Be concise and actionable.".to_string(),
                }],
            }),
        };

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&gemini_req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Gemini: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error (status {}): {}", status, error_text);
        }

        // For non-streaming, Gemini returns a single response
        let gemini_resp: GeminiStreamResponse = response.json().await?;

        Ok(gemini_resp
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_else(|| "No response from model".to_string()))
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
