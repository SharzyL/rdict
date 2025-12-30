use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub api_key: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_response_language")]
    pub response_language: String,
}

fn default_model() -> String {
    "claude-haiku-4-5".to_string()
}

fn default_response_language() -> String {
    "Chinese".to_string()
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("rdict");

        std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    pub fn load_from_path(config_path: Option<PathBuf>) -> Result<Self> {
        let config_path = if let Some(path) = config_path {
            path
        } else {
            Self::config_path()?
        };

        if !config_path.exists() {
            // Create default config file
            let default_config = Self::default();
            let toml_str = toml::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            std::fs::write(&config_path, &toml_str)
                .context("Failed to write default config file")?;

            info!("Created default config file: {}", config_path.display());
            info!("Please edit the config file and add your API key, then run the program again");

            return Ok(default_config);
        }

        let content =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;

        toml::from_str(&content).context("Failed to parse config file")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: "your-api-key-here".to_string(),
                model: default_model(),
                response_language: default_response_language(),
            },
        }
    }
}
