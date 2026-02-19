mod api;
mod config;
mod params;
mod server;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};

use api::OpenAIClient;
use server::GptServer;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load()?;
    let client = OpenAIClient::new(cfg.api_key);
    let server = GptServer::new(client);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
