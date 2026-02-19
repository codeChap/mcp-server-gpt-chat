use reqwest::Method;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_handler, tool_router,
};
use serde_json::Value;

use crate::api::{
    ChatMessage, ChatRequest, ChatResponse, EmbeddingRequest, EmbeddingResponse,
    ModelsResponse, OpenAIClient, format_chat_response, format_embedding_response,
};
use crate::params::{ChatParams, EmbeddingParams, VisionParams};

const DEFAULT_MODEL: &str = "gpt-4o";
const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";

#[derive(Clone)]
pub struct GptServer {
    client: std::sync::Arc<OpenAIClient>,
    tool_router: ToolRouter<Self>,
}

// ---------------------------------------------------------------------------
// Shared helpers — keep tool methods DRY
// ---------------------------------------------------------------------------

impl GptServer {
    /// Validate temperature is within the allowed range.
    fn validate_temperature(temp: Option<f32>) -> Result<(), McpError> {
        if let Some(t) = temp {
            if !(0.0..=2.0).contains(&t) {
                return Err(McpError::invalid_params(
                    format!("temperature must be between 0.0 and 2.0, got {t}"),
                    None,
                ));
            }
        }
        Ok(())
    }

    /// Build the messages vec from optional system prompt, optional history, and current prompt.
    fn build_messages(
        system_prompt: Option<&str>,
        history_json: Option<&str>,
        prompt: &str,
    ) -> Result<Vec<ChatMessage>, String> {
        let mut messages = Vec::new();

        if let Some(sys) = system_prompt {
            messages.push(ChatMessage::system(sys));
        }

        if let Some(json) = history_json {
            let parsed: Vec<ChatMessage> = serde_json::from_str(json)
                .map_err(|e| format!("Invalid messages JSON: {e}"))?;
            messages.extend(parsed);
        }

        messages.push(ChatMessage::user(prompt));
        Ok(messages)
    }

    /// Build a ChatRequest with shared optional fields applied.
    fn build_chat_request(
        model: Option<&str>,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        response_schema: Option<&str>,
    ) -> Result<ChatRequest, String> {
        let mut req = ChatRequest::new(model.unwrap_or(DEFAULT_MODEL), messages);
        req.temperature = temperature;
        req.max_tokens = max_tokens;

        if let Some(schema_str) = response_schema {
            let schema: Value = serde_json::from_str(schema_str)
                .map_err(|e| format!("Invalid response_schema JSON: {e}"))?;
            req.response_format = Some(serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "structured_output",
                    "strict": true,
                    "schema": schema
                }
            }));
        }

        Ok(req)
    }

    /// Send a chat request and return the formatted result.
    async fn do_chat(&self, req: &ChatRequest) -> Result<CallToolResult, McpError> {
        match self
            .client
            .request::<_, ChatResponse>(Method::POST, "/chat/completions", Some(req))
            .await
        {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_chat_response(&resp),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

#[tool_router]
impl GptServer {
    pub fn new(client: OpenAIClient) -> Self {
        Self {
            client: std::sync::Arc::new(client),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Send a chat completion request to ChatGPT. Supports multi-turn conversations, \
                           structured output via JSON schema, and model selection.")]
    async fn chat(
        &self,
        Parameters(p): Parameters<ChatParams>,
    ) -> Result<CallToolResult, McpError> {
        Self::validate_temperature(p.temperature)?;

        let messages = Self::build_messages(
            p.system_prompt.as_deref(),
            p.messages.as_deref(),
            &p.prompt,
        )
        .map_err(|e| McpError::invalid_params(e, None))?;

        let req = Self::build_chat_request(
            p.model.as_deref(),
            messages,
            p.temperature,
            p.max_tokens,
            p.response_schema.as_deref(),
        )
        .map_err(|e| McpError::invalid_params(e, None))?;

        self.do_chat(&req).await
    }

    #[tool(description = "Analyse an image with ChatGPT's vision capabilities. \
                           Provide an image URL and a text prompt.")]
    async fn chat_with_vision(
        &self,
        Parameters(p): Parameters<VisionParams>,
    ) -> Result<CallToolResult, McpError> {
        if !p.image_url.starts_with("http://") && !p.image_url.starts_with("https://") {
            return Err(McpError::invalid_params(
                "image_url must start with http:// or https://",
                None,
            ));
        }
        Self::validate_temperature(p.temperature)?;

        let detail = p.detail.as_deref().unwrap_or("high");
        let messages = vec![ChatMessage::user_with_image(&p.prompt, &p.image_url, detail)];

        let req = Self::build_chat_request(
            p.model.as_deref(),
            messages,
            p.temperature,
            p.max_tokens,
            None,
        )
        .map_err(|e| McpError::invalid_params(e, None))?;

        self.do_chat(&req).await
    }

    #[tool(description = "Generate text embeddings using OpenAI's embedding model.")]
    async fn embedding(
        &self,
        Parameters(p): Parameters<EmbeddingParams>,
    ) -> Result<CallToolResult, McpError> {
        let input: Value = serde_json::from_str(&p.input).map_err(|e| {
            McpError::invalid_params(
                format!("Invalid input JSON (must be a quoted string or array of strings): {e}"),
                None,
            )
        })?;

        let req = EmbeddingRequest {
            model: p.model.unwrap_or_else(|| DEFAULT_EMBEDDING_MODEL.into()),
            input,
        };

        match self
            .client
            .request::<_, EmbeddingResponse>(Method::POST, "/embeddings", Some(&req))
            .await
        {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_embedding_response(&resp),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "List all available OpenAI models and their IDs.")]
    async fn list_models(&self) -> Result<CallToolResult, McpError> {
        match self
            .client
            .request::<(), ModelsResponse>(Method::GET, "/models", None)
            .await
        {
            Ok(resp) => {
                let lines: Vec<String> = resp
                    .data
                    .iter()
                    .map(|m| {
                        let owner = m.owned_by.as_deref().unwrap_or("openai");
                        format!("- {} ({})", m.id, owner)
                    })
                    .collect();
                Ok(CallToolResult::success(vec![Content::text(
                    lines.join("\n"),
                )]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }
}

// ---------------------------------------------------------------------------
// MCP ServerHandler
// ---------------------------------------------------------------------------

#[tool_handler]
impl ServerHandler for GptServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "gpt-chat".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "OpenAI ChatGPT MCP server. Tools: chat, chat_with_vision, \
                 embedding, list_models."
                    .into(),
            ),
        }
    }
}
