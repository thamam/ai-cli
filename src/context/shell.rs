use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellContext {
    pub last_command: Option<String>,
    pub last_exit_code: Option<i32>,
    pub last_error: Option<String>,
    pub working_directory: PathBuf,
    pub shell_type: String,
}

impl ShellContext {
    /// Load context from the temporary file written by shell hooks
    pub fn load() -> Result<Option<Self>> {
        let context_path = Self::get_context_path();

        if !context_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&context_path)?;
        let context: ShellContext = serde_json::from_str(&content)?;
        Ok(Some(context))
    }

    /// Save context to temporary file
    pub fn save(&self) -> Result<()> {
        let context_path = Self::get_context_path();

        // Ensure directory exists
        if let Some(parent) = context_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&context_path, content)?;

        Ok(())
    }

    /// Get the path to the context file
    fn get_context_path() -> PathBuf {
        PathBuf::from("/tmp/aether/context.json")
    }

    /// Get the path to the last session file (for Sentinel mode)
    pub fn get_last_session_path() -> PathBuf {
        PathBuf::from("/tmp/aether/last_session")
    }

    /// Read shell history (bash/zsh)
    pub fn read_history(limit: usize) -> Result<Vec<String>> {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;

        // Try both bash and zsh history files
        let history_paths = vec![
            PathBuf::from(&home).join(".bash_history"),
            PathBuf::from(&home).join(".zsh_history"),
        ];

        for history_path in history_paths {
            if history_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&history_path) {
                    let commands: Vec<String> = content
                        .lines()
                        .rev()
                        .take(limit)
                        .map(|s| s.to_string())
                        .collect();
                    return Ok(commands);
                }
            }
        }

        Ok(Vec::new())
    }
}
