use anyhow::Result;

/// Command executor with dry-run support
pub struct CommandExecutor;

impl CommandExecutor {
    /// Check if a command is potentially destructive
    pub fn is_destructive(command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf",
            "rm -fr",
            "drop table",
            "drop database",
            "delete from",
            "truncate",
            "mkfs",
            "dd if=",
            "> /dev/",
            ":(){ :|:& };:", // Fork bomb
            "chmod -R 777",
            "chown -R",
        ];

        let command_lower = command.to_lowercase();
        dangerous_patterns
            .iter()
            .any(|pattern| command_lower.contains(pattern))
    }

    /// Analyze a command and provide information about what it will do
    /// This is a simplified version - Phase 4 will have more sophisticated analysis
    pub fn analyze_command(command: &str) -> String {
        if Self::is_destructive(command) {
            format!("⚠️  WARNING: This command appears to be destructive!\n\nCommand: {}\n\nThis may delete files, modify permissions, or cause irreversible changes.", command)
        } else {
            format!("Command: {}\n\nThis command appears safe to execute.", command)
        }
    }

    /// Execute a command (placeholder for now)
    pub async fn execute(_command: &str) -> Result<String> {
        // TODO: Implement actual command execution
        // This will be used in later phases
        Ok("Command execution not yet implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destructive_command_detection() {
        assert!(CommandExecutor::is_destructive("rm -rf /"));
        assert!(CommandExecutor::is_destructive("DROP TABLE users"));
        assert!(CommandExecutor::is_destructive("dd if=/dev/zero of=/dev/sda"));
        assert!(!CommandExecutor::is_destructive("ls -la"));
        assert!(!CommandExecutor::is_destructive("echo 'hello world'"));
    }
}
