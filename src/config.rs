use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api_key: String,
}

pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home)
        .join(".config")
        .join("mcp-server-gpt-chat")
        .join("config.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    let content = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read config file: {}\n\
             Create it with your OpenAI API key.\n\
             Example:\n\n\
             api_key = \"sk-...\"",
            path.display()
        )
    })?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}
