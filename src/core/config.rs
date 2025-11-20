use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// AI provider configuration
    pub ai: AIConfig,

    /// UI preferences
    pub ui: UIConfig,

    /// Safety settings
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Default provider: "ollama", "openai", "anthropic", "gemini"
    pub default_provider: String,

    /// Ollama configuration
    pub ollama: OllamaConfig,

    /// OpenAI configuration
    pub openai: OpenAIConfig,

    /// Anthropic configuration
    pub anthropic: AnthropicConfig,

    /// Gemini configuration
    pub gemini: GeminiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Modal width percentage (0-100)
    pub modal_width_percent: u16,

    /// Modal height percentage (0-100)
    pub modal_height_percent: u16,

    /// Enable animations
    pub animations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable destructive command detection
    pub detect_destructive_commands: bool,

    /// Require confirmation for destructive commands
    pub confirm_destructive: bool,

    /// Enable dry-run mode by default
    pub default_dry_run: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AIConfig {
                default_provider: "ollama".to_string(),
                ollama: OllamaConfig {
                    enabled: true,
                    base_url: "http://localhost:11434".to_string(),
                    model: "llama3".to_string(),
                },
                openai: OpenAIConfig {
                    enabled: false,
                    api_key: None,
                    model: "gpt-4".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                },
                anthropic: AnthropicConfig {
                    enabled: false,
                    api_key: None,
                    model: "claude-3-5-sonnet-20241022".to_string(),
                },
                gemini: GeminiConfig {
                    enabled: false,
                    api_key: None,
                    model: "gemini-1.5-pro".to_string(),
                },
            },
            ui: UIConfig {
                modal_width_percent: 80,
                modal_height_percent: 70,
                animations: true,
            },
            safety: SafetyConfig {
                detect_destructive_commands: true,
                confirm_destructive: true,
                default_dry_run: false,
            },
        }
    }
}

impl Config {
    /// Load configuration using the config crate
    /// Supports multiple sources: config file, environment variables, defaults
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // If config doesn't exist, create default
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
        }

        // Use config crate to load from multiple sources
        let settings = config::Config::builder()
            // Start with defaults
            .add_source(config::File::from(config_path).required(false))
            // Add environment variables (with prefix AETHER_)
            .add_source(
                config::Environment::with_prefix("AETHER")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        // Deserialize into our Config struct
        let config: Config = settings.try_deserialize()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get the configuration file path
    pub fn get_config_path() -> Result<PathBuf> {
        // Check for AETHER_CONFIG_PATH env var first
        if let Ok(custom_path) = std::env::var("AETHER_CONFIG_PATH") {
            return Ok(PathBuf::from(custom_path));
        }

        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;

        Ok(PathBuf::from(home)
            .join(".config")
            .join("aether")
            .join("config.toml"))
    }
}
