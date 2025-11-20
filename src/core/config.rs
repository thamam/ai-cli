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
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
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
    fn get_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;

        Ok(PathBuf::from(home).join(".config").join("aether").join("config.toml"))
    }
}
