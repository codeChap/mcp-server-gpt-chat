use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for the `chat` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChatParams {
    #[schemars(description = "The user message / prompt to send to ChatGPT")]
    pub prompt: String,

    #[schemars(description = "Optional system prompt to set context/behaviour")]
    pub system_prompt: Option<String>,

    #[schemars(
        description = "Full conversation history as JSON array of {role, content} objects. \
                        When provided, 'prompt' is appended as the final user message."
    )]
    pub messages: Option<String>,

    #[schemars(
        description = "Model to use. Defaults to gpt-4o. \
                        Options: gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-4, gpt-3.5-turbo, o1, o1-mini"
    )]
    pub model: Option<String>,

    #[schemars(description = "Sampling temperature (0.0 - 2.0)")]
    pub temperature: Option<f32>,

    #[schemars(description = "Maximum tokens to generate")]
    pub max_tokens: Option<u32>,

    #[schemars(
        description = "Optional JSON schema string to enforce structured output. \
                        The model response will conform to this schema."
    )]
    pub response_schema: Option<String>,
}

/// Parameters for the `chat_with_vision` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VisionParams {
    #[schemars(description = "Text prompt describing what to analyse in the image")]
    pub prompt: String,

    #[schemars(description = "URL of the image to analyse (must be http:// or https://)")]
    pub image_url: String,

    #[schemars(description = "Image detail level: \"low\" or \"high\" (default: \"high\")")]
    pub detail: Option<String>,

    #[schemars(
        description = "Model to use. Defaults to gpt-4o. \
                        Must be a vision-capable model."
    )]
    pub model: Option<String>,

    #[schemars(description = "Sampling temperature (0.0 - 2.0)")]
    pub temperature: Option<f32>,

    #[schemars(description = "Maximum tokens to generate")]
    pub max_tokens: Option<u32>,
}

/// Parameters for the `embedding` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmbeddingParams {
    #[schemars(description = "Text to embed as JSON: a single string or array of strings.")]
    pub input: String,

    #[schemars(description = "Embedding model to use (default: text-embedding-3-small)")]
    pub model: Option<String>,
}
