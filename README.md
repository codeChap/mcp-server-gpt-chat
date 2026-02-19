# mcp-server-gpt-chat

An MCP (Model Context Protocol) server for the OpenAI ChatGPT API. Built in Rust, exposes chat completions, vision, embeddings, and model listing as MCP tools.

Communicates via stdio using JSON-RPC 2.0, like all MCP servers.

## Tools

| Tool | Description |
|------|-------------|
| `chat` | Send a chat completion request to ChatGPT with optional multi-turn history, system prompt, structured output (JSON schema), and model selection |
| `chat_with_vision` | Analyse an image with ChatGPT's vision capabilities given an image URL and text prompt |
| `embedding` | Generate text embeddings using OpenAI's embedding model |
| `list_models` | List all available OpenAI models and their IDs |

### chat

Send a chat completion request. Supports multi-turn conversations via a JSON message history array, system prompts, structured output via JSON schema, temperature control, and model selection.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `prompt` | string | yes | The user message to send |
| `model` | string | no | Model to use (default: `gpt-4o`) |
| `system_prompt` | string | no | System prompt to set context |
| `messages` | string | no | Full conversation history as JSON array of `{role, content}` objects |
| `temperature` | float | no | Sampling temperature (0.0 - 2.0) |
| `max_tokens` | integer | no | Maximum tokens to generate |
| `response_schema` | string | no | JSON schema string to enforce structured output |

### chat_with_vision

Analyse an image using ChatGPT's vision capabilities.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `prompt` | string | yes | Text prompt describing what to analyse |
| `image_url` | string | yes | URL of the image (must be http:// or https://) |
| `model` | string | no | Model to use (default: `gpt-4o`) |
| `detail` | string | no | Image detail level: `low` or `high` (default: `high`) |
| `temperature` | float | no | Sampling temperature (0.0 - 2.0) |
| `max_tokens` | integer | no | Maximum tokens to generate |

### embedding

Generate text embeddings.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `input` | string | yes | Text to embed as JSON: a single string or array of strings |
| `model` | string | no | Embedding model to use (default: `text-embedding-3-small`) |

### list_models

List all available OpenAI models. No parameters.

## Prerequisites

- Rust (edition 2024)
- An OpenAI API key from [platform.openai.com](https://platform.openai.com/)

## Setup

Create the config file:

```bash
mkdir -p ~/.config/mcp-server-gpt-chat
```

Create `~/.config/mcp-server-gpt-chat/config.toml`:

```toml
api_key = "sk-..."
```

## Build

```bash
cargo build --release
```

This produces `target/release/gpt-chat`.

For development:

```bash
cargo build              # debug build
cargo run                # run in dev mode
RUST_LOG=debug cargo run # run with debug logging
```

## MCP Configuration

Add to your Claude Desktop config (`~/.config/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "gpt-chat": {
      "command": "/path/to/gpt-chat"
    }
  }
}
```

## Project Structure

```
src/
  main.rs    - entry point, config loading, stdio transport setup
  server.rs  - MCP tool definitions (chat, chat_with_vision, embedding, list_models)
  api.rs     - OpenAI HTTP client, request/response types, response formatters
  params.rs  - tool parameter types with serde and JSON Schema derives
  config.rs  - TOML config loading
```

## License

MIT
