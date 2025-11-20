// Eval-Driven Tests for AETHER
// These tests verify that the AI produces semantically correct commands

mod evals;

use aether::ai::{AIProvider, CompletionRequest, MockAIProvider};
use evals::{default_eval_cases, EvalResult};
use futures::StreamExt;

#[tokio::test]
async fn test_mock_provider_with_eval_cases() {
    let provider = MockAIProvider::new();
    let cases = default_eval_cases();

    for case in cases {
        println!("Testing: {}", case.description);

        let req = CompletionRequest {
            system_prompt: "You are a shell command generator".to_string(),
            user_query: case.user_query.clone(),
            context_files: case.context_files.clone(),
            history: vec![],
        };

        // Get response from mock provider
        let mut stream = provider.stream_completion(req).await.unwrap();
        let mut actual_command = String::new();
        while let Some(chunk) = stream.next().await {
            actual_command.push_str(&chunk.unwrap());
        }

        // Evaluate the response
        let result = case.evaluate(&actual_command);

        match result {
            EvalResult::Pass => {
                println!("  ✓ PASS: {}", actual_command);
            }
            EvalResult::Fail(errors) => {
                println!("  ✗ FAIL: {}", actual_command);
                for error in errors {
                    println!("    - {}", error);
                }
                panic!("Eval case failed: {}", case.description);
            }
        }
    }
}

#[tokio::test]
async fn test_specific_git_reset_case() {
    let provider = MockAIProvider::new();

    let req = CompletionRequest {
        system_prompt: "You are a shell command generator".to_string(),
        user_query: "undo last git commit".to_string(),
        context_files: vec![],
        history: vec![],
    };

    let mut stream = provider.stream_completion(req).await.unwrap();
    let mut command = String::new();
    while let Some(chunk) = stream.next().await {
        command.push_str(&chunk.unwrap());
    }

    // Verify semantics: must be a git reset command
    assert!(command.contains("git"));
    assert!(command.contains("reset") || command.contains("revert"));
}

#[tokio::test]
async fn test_list_files_case() {
    let provider = MockAIProvider::new();

    let req = CompletionRequest {
        system_prompt: "You are a shell command generator".to_string(),
        user_query: "list files".to_string(),
        context_files: vec![],
        history: vec![],
    };

    let mut stream = provider.stream_completion(req).await.unwrap();
    let mut command = String::new();
    while let Some(chunk) = stream.next().await {
        command.push_str(&chunk.unwrap());
    }

    // Verify it's an ls command
    assert!(command.contains("ls"));
}

#[test]
fn test_eval_framework_logic() {
    use evals::EvalCase;

    let case = EvalCase {
        description: "Test case".to_string(),
        user_query: "test".to_string(),
        context_files: vec![],
        expected_command_pattern: "git.*reset".to_string(),
        required_keywords: Some(vec!["git".to_string()]),
        forbidden_keywords: Some(vec!["rm".to_string()]),
    };

    // Should pass
    let result = case.evaluate("git reset --soft HEAD~1");
    assert!(matches!(result, EvalResult::Pass));

    // Should fail (missing pattern)
    let result = case.evaluate("ls -la");
    assert!(matches!(result, EvalResult::Fail(_)));

    // Should fail (forbidden keyword)
    let result = case.evaluate("git reset && rm -rf /");
    assert!(matches!(result, EvalResult::Fail(_)));
}
