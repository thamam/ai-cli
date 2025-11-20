// Eval-Driven Testing Framework
// Tests verify that AI responses match expected semantic patterns

use regex::Regex;
use serde::{Deserialize, Serialize};

/// An evaluation test case
/// Tests check if the AI's output matches semantic expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    /// Human-readable description of the test
    pub description: String,

    /// User's natural language query
    pub user_query: String,

    /// Mock context files (filename -> content)
    pub context_files: Vec<(String, String)>,

    /// Regex pattern that the command must match
    /// Example: "git reset.*HEAD" matches "git reset --soft HEAD~1"
    pub expected_command_pattern: String,

    /// Optional: List of required keywords (all must be present)
    pub required_keywords: Option<Vec<String>>,

    /// Optional: List of forbidden keywords (none should be present)
    pub forbidden_keywords: Option<Vec<String>>,
}

impl EvalCase {
    /// Check if a command passes this eval case
    pub fn evaluate(&self, actual_command: &str) -> EvalResult {
        let mut errors = Vec::new();

        // Check regex pattern
        match Regex::new(&self.expected_command_pattern) {
            Ok(re) => {
                if !re.is_match(actual_command) {
                    errors.push(format!(
                        "Command '{}' does not match pattern '{}'",
                        actual_command, self.expected_command_pattern
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("Invalid regex pattern: {}", e));
            }
        }

        // Check required keywords
        if let Some(required) = &self.required_keywords {
            for keyword in required {
                if !actual_command.contains(keyword) {
                    errors.push(format!(
                        "Missing required keyword '{}' in command '{}'",
                        keyword, actual_command
                    ));
                }
            }
        }

        // Check forbidden keywords
        if let Some(forbidden) = &self.forbidden_keywords {
            for keyword in forbidden {
                if actual_command.contains(keyword) {
                    errors.push(format!(
                        "Found forbidden keyword '{}' in command '{}'",
                        keyword, actual_command
                    ));
                }
            }
        }

        if errors.is_empty() {
            EvalResult::Pass
        } else {
            EvalResult::Fail(errors)
        }
    }
}

/// Result of an evaluation
#[derive(Debug, PartialEq)]
pub enum EvalResult {
    Pass,
    Fail(Vec<String>),
}

/// Load eval cases from a JSON file
pub fn load_eval_cases(path: &str) -> anyhow::Result<Vec<EvalCase>> {
    let content = std::fs::read_to_string(path)?;
    let cases: Vec<EvalCase> = serde_json::from_str(&content)?;
    Ok(cases)
}

/// Create default eval cases for testing
pub fn default_eval_cases() -> Vec<EvalCase> {
    vec![
        EvalCase {
            description: "Undo last git commit (soft reset)".to_string(),
            user_query: "undo last git commit".to_string(),
            context_files: vec![],
            expected_command_pattern: "git reset.*(--soft|HEAD)".to_string(),
            required_keywords: Some(vec!["git".to_string(), "reset".to_string()]),
            forbidden_keywords: Some(vec!["rm".to_string(), "delete".to_string()]),
        },
        EvalCase {
            description: "List files in current directory".to_string(),
            user_query: "list files".to_string(),
            context_files: vec![],
            expected_command_pattern: "ls.*".to_string(),
            required_keywords: Some(vec!["ls".to_string()]),
            forbidden_keywords: None,
        },
        EvalCase {
            description: "Find Python files".to_string(),
            user_query: "find python files".to_string(),
            context_files: vec![],
            expected_command_pattern: "find.*\\.py".to_string(),
            required_keywords: Some(vec!["find".to_string(), ".py".to_string()]),
            forbidden_keywords: None,
        },
        EvalCase {
            description: "Git status command".to_string(),
            user_query: "show git status".to_string(),
            context_files: vec![],
            expected_command_pattern: "git status".to_string(),
            required_keywords: Some(vec!["git".to_string(), "status".to_string()]),
            forbidden_keywords: None,
        },
        EvalCase {
            description: "Complex tar operation".to_string(),
            user_query: "find all python files modified yesterday and tar them".to_string(),
            context_files: vec![],
            expected_command_pattern: "find.*\\.py.*tar".to_string(),
            required_keywords: Some(vec![
                "find".to_string(),
                ".py".to_string(),
                "tar".to_string(),
            ]),
            forbidden_keywords: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_case_pass() {
        let case = EvalCase {
            description: "Test git reset".to_string(),
            user_query: "undo commit".to_string(),
            context_files: vec![],
            expected_command_pattern: "git reset.*HEAD".to_string(),
            required_keywords: Some(vec!["git".to_string()]),
            forbidden_keywords: Some(vec!["rm".to_string()]),
        };

        let result = case.evaluate("git reset --soft HEAD~1");
        assert_eq!(result, EvalResult::Pass);
    }

    #[test]
    fn test_eval_case_fail_pattern() {
        let case = EvalCase {
            description: "Test git reset".to_string(),
            user_query: "undo commit".to_string(),
            context_files: vec![],
            expected_command_pattern: "git reset.*HEAD".to_string(),
            required_keywords: None,
            forbidden_keywords: None,
        };

        let result = case.evaluate("git status");
        match result {
            EvalResult::Fail(_) => assert!(true),
            EvalResult::Pass => panic!("Should have failed"),
        }
    }

    #[test]
    fn test_eval_case_fail_forbidden() {
        let case = EvalCase {
            description: "Test safe command".to_string(),
            user_query: "list files".to_string(),
            context_files: vec![],
            expected_command_pattern: "ls.*".to_string(),
            required_keywords: None,
            forbidden_keywords: Some(vec!["rm".to_string()]),
        };

        let result = case.evaluate("ls -la && rm -rf /");
        match result {
            EvalResult::Fail(_) => assert!(true),
            EvalResult::Pass => panic!("Should have failed due to forbidden keyword"),
        }
    }

    #[test]
    fn test_default_eval_cases() {
        let cases = default_eval_cases();
        assert!(!cases.is_empty());
        assert!(cases.len() >= 5);
    }
}
