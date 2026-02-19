use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

const BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("OpenAI API error ({status}): {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
}

/// Shared HTTP client for all OpenAI API calls.
pub struct OpenAIClient {
    api_key: String,
    http: Client,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build reqwest client"),
        }
    }

    /// Unified HTTP request method — handles GET and POST with optional body.
    pub async fn request<Req: Serialize, Resp: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Req>,
    ) -> Result<Resp, ApiError> {
        let url = format!("{BASE_URL}{path}");
        let mut builder = self
            .http
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.api_key));

        if let Some(b) = body {
            builder = builder.json(b);
        }

        let response = builder.send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, body });
        }

        Ok(response.json::<Resp>().await?)
    }
}

// ---------------------------------------------------------------------------
// Chat Completions API types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Deserialize)]
pub struct ChatChoice {
    pub message: ChatResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatResponseMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<Value>>,
}

#[derive(Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Embeddings API types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Value,
}

#[derive(Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
    pub usage: Option<EmbeddingUsage>,
}

#[derive(Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Deserialize)]
pub struct EmbeddingUsage {
    #[allow(dead_code)]
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Models API types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}

#[derive(Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub owned_by: Option<String>,
}

// ---------------------------------------------------------------------------
// Convenience builders
// ---------------------------------------------------------------------------

impl ChatRequest {
    pub fn new(model: &str, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            response_format: None,
            tools: None,
        }
    }
}

impl ChatMessage {
    pub fn system(text: &str) -> Self {
        Self {
            role: "system".into(),
            content: Some(Value::String(text.into())),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(text: &str) -> Self {
        Self {
            role: "user".into(),
            content: Some(Value::String(text.into())),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user_with_image(text: &str, image_url: &str, detail: &str) -> Self {
        Self {
            role: "user".into(),
            content: Some(serde_json::json!([
                { "type": "text", "text": text },
                {
                    "type": "image_url",
                    "image_url": { "url": image_url, "detail": detail }
                }
            ])),
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Response formatters
// ---------------------------------------------------------------------------

pub fn format_chat_response(resp: &ChatResponse) -> String {
    let mut parts = Vec::new();

    for choice in &resp.choices {
        if let Some(content) = &choice.message.content {
            parts.push(content.clone());
        }
        if let Some(tool_calls) = &choice.message.tool_calls {
            parts.push(format!(
                "Tool calls: {}",
                serde_json::to_string_pretty(tool_calls).unwrap_or_default()
            ));
        }
        if let Some(reason) = &choice.finish_reason {
            parts.push(format!("[finish_reason: {reason}]"));
        }
    }

    if let Some(usage) = &resp.usage {
        parts.push(format!(
            "[tokens: {} prompt + {} completion = {} total]",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        ));
    }

    parts.join("\n")
}

pub fn format_embedding_response(resp: &EmbeddingResponse) -> String {
    let mut parts = Vec::new();

    for item in &resp.data {
        let preview: Vec<String> = item
            .embedding
            .iter()
            .take(5)
            .map(|v| format!("{v:.6}"))
            .collect();
        parts.push(format!(
            "[{}] dim={} [{}, ...]",
            item.index,
            item.embedding.len(),
            preview.join(", ")
        ));
    }

    if let Some(usage) = &resp.usage {
        parts.push(format!("[tokens: {}]", usage.total_tokens));
    }

    parts.join("\n")
}
