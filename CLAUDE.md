# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build --release    # Production build (binary: target/release/gpt-chat)
cargo build              # Debug build
cargo run                # Run via stdio
RUST_LOG=debug cargo run # Run with debug logging
```

Rust edition 2024. No test suite currently.

## Configuration

TOML config at `~/.config/mcp-server-gpt-chat/config.toml`:

```toml
api_key = "sk-..."
```

## Architecture

MCP server wrapping the OpenAI API (chat completions, vision, embeddings, model listing). Communicates via stdio using JSON-RPC 2.0.

- `src/main.rs` — entry point: loads config, creates API client, starts stdio MCP transport via `rmcp`
- `src/config.rs` — reads TOML config from `~/.config/mcp-server-gpt-chat/config.toml`
- `src/api.rs` — `OpenAIClient` (generic HTTP client for OpenAI v1 API), request/response types for chat completions, embeddings, and models, plus response formatters
- `src/server.rs` — `GptServer` with four `#[tool]`-annotated MCP tools: `chat`, `chat_with_vision`, `embedding`, `list_models`. Uses `rmcp` `#[tool_router]` and `#[tool_handler]` macros.
- `src/params.rs` — tool parameter structs with `schemars::JsonSchema` derives (these generate the MCP tool input schemas)

The sibling project `mcp-server-grok-chat` is nearly identical in structure but targets the xAI API.

## Key Dependencies

- `rmcp` — MCP protocol handling (server, stdio transport, tool macros)
- `reqwest` — HTTP client for OpenAI API
- `schemars` v1 — JSON Schema generation for tool parameters
- `serde`/`serde_json` — serialization
- `toml` — config parsing
