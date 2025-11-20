// The "Brain" - Core AI logic decoupled from UI
// Can be called headless for testing and programmatic use

use crate::ai::{AIProvider, CompletionRequest};
use crate::context::{FileScanner, ShellContext};
use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;

/// The Brain processes user queries and generates commands
/// It's decoupled from the TUI so it can be tested headless
pub struct Brain {
    ai_provider: Arc<dyn AIProvider>,
}

impl Brain {
    pub fn new(ai_provider: Arc<dyn AIProvider>) -> Self {
        Self { ai_provider }
    }

    /// Process a query and return the suggested command
    /// This is the headless version - no UI, just logic
    pub async fn process_query(
        &self,
        query: &str,
        include_context: bool,
    ) -> Result<String> {
        // Build the request
        let mut req = CompletionRequest {
            system_prompt: crate::ai::prompts::generate_system_prompt("lens", &[]),
            user_query: query.to_string(),
            context_files: vec![],
            history: vec![],
        };

        // Add context if requested
        if include_context {
            // Load shell history
            if let Ok(history) = ShellContext::read_history(5) {
                req.history = history;
            }

            // Scan current directory for relevant files
            if let Ok(cwd) = std::env::current_dir() {
                if let Ok(files) = FileScanner::scan_directory(&cwd, 10) {
                    req.context_files = files;
                }
            }
        }

        // Get streaming response and collect
        let mut stream = self.ai_provider.stream_completion(req).await?;
        let mut response = String::new();

        while let Some(chunk) = stream.next().await {
            response.push_str(&chunk?);
        }

        Ok(response.trim().to_string())
    }

    /// Process an error log and suggest a fix (Sentinel mode)
    pub async fn process_error(&self, error_log: &str) -> Result<String> {
        self.ai_provider.get_fix_suggestion(error_log.to_string()).await
    }

    /// Analyze a command for safety (dry-run)
    pub fn analyze_command(&self, command: &str) -> CommandAnalysis {
        CommandAnalysis {
            command: command.to_string(),
            is_destructive: crate::core::executor::CommandExecutor::is_destructive(command),
            description: crate::core::executor::CommandExecutor::analyze_command(command),
        }
    }
}

/// Analysis of a command
#[derive(Debug, Clone)]
pub struct CommandAnalysis {
    pub command: String,
    pub is_destructive: bool,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::MockAIProvider;

    #[tokio::test]
    async fn test_brain_headless() {
        let provider = Arc::new(MockAIProvider::new());
        let brain = Brain::new(provider);

        let result = brain.process_query("list files", false).await.unwrap();
        assert_eq!(result, "ls -la");
    }

    #[tokio::test]
    async fn test_brain_git_reset() {
        let provider = Arc::new(MockAIProvider::new());
        let brain = Brain::new(provider);

        let result = brain.process_query("undo last git commit", false).await.unwrap();
        assert!(result.contains("git"));
        assert!(result.contains("reset"));
    }

    #[test]
    fn test_brain_command_analysis() {
        let provider = Arc::new(MockAIProvider::new());
        let brain = Brain::new(provider);

        let analysis = brain.analyze_command("rm -rf /");
        assert!(analysis.is_destructive);

        let analysis = brain.analyze_command("ls -la");
        assert!(!analysis.is_destructive);
    }
}
