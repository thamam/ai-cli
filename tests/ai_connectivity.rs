// AI Connectivity Tests
// These tests verify connections to AI providers (Ollama, OpenAI, etc.)
// Run with: cargo test -- --ignored

use aether::ai::client::{AIProvider, CompletionRequest};
use aether::ai::ollama::OllamaClient;
use futures::StreamExt;

#[tokio::test]
#[ignore] // Requires Ollama to be running locally
async fn test_ollama_connectivity() {
    // This test verifies that we can connect to Ollama and stream responses
    // Prerequisites:
    // 1. Ollama must be running: ollama serve
    // 2. A model must be available: ollama pull llama3

    let client = OllamaClient::new(
        "http://localhost:11434".to_string(),
        "llama3".to_string(),
    );

    let req = CompletionRequest {
        system_prompt: "You are a helpful assistant.".to_string(),
        user_query: "Say 'Hello, World!' and nothing else.".to_string(),
        context_files: vec![],
        history: vec![],
    };

    // Test streaming
    let stream = client
        .stream_completion(req)
        .await
        .expect("Failed to create stream");

    let mut stream = Box::pin(stream);
    let mut full_response = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(token) => {
                full_response.push_str(&token);
                print!("{}", token); // Show streaming in real-time
            }
            Err(e) => {
                panic!("Stream error: {}", e);
            }
        }
    }

    println!("\n\nFull response: {}", full_response);

    // Verify we got a response
    assert!(!full_response.is_empty(), "Should receive a response");
    assert!(
        full_response.to_lowercase().contains("hello"),
        "Response should contain 'hello'"
    );
}

#[tokio::test]
#[ignore] // Requires Ollama to be running locally
async fn test_ollama_sentinel_mode() {
    // Test non-streaming mode for Sentinel (error analysis)

    let client = OllamaClient::new(
        "http://localhost:11434".to_string(),
        "llama3".to_string(),
    );

    let error_log = r#"
Command: ls /nonexistent
Error: ls: cannot access '/nonexistent': No such file or directory
Exit code: 2
"#;

    let suggestion = client
        .get_fix_suggestion(error_log.to_string())
        .await
        .expect("Failed to get fix suggestion");

    println!("Suggestion: {}", suggestion);

    // Verify we got a meaningful response
    assert!(!suggestion.is_empty(), "Should receive a suggestion");
}

#[tokio::test]
#[ignore] // Requires Ollama
async fn test_ollama_with_context() {
    // Test that context files are properly included

    let client = OllamaClient::new(
        "http://localhost:11434".to_string(),
        "llama3".to_string(),
    );

    let req = CompletionRequest {
        system_prompt: "You are a code assistant.".to_string(),
        user_query: "What does the add function do?".to_string(),
        context_files: vec![(
            "math.rs".to_string(),
            "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        )],
        history: vec!["cargo build".to_string(), "cargo test".to_string()],
    };

    let stream = client
        .stream_completion(req)
        .await
        .expect("Failed to create stream");

    let mut stream = Box::pin(stream);
    let mut full_response = String::new();

    while let Some(result) = stream.next().await {
        if let Ok(token) = result {
            full_response.push_str(&token);
        }
    }

    println!("Response with context: {}", full_response);

    // Verify the model understood the context
    assert!(!full_response.is_empty(), "Should receive a response");
}

#[tokio::test]
#[ignore] // Requires Ollama
async fn test_ollama_handles_connection_error() {
    // Test error handling when Ollama is not running

    let client = OllamaClient::new(
        "http://localhost:99999".to_string(), // Invalid port
        "llama3".to_string(),
    );

    let req = CompletionRequest {
        system_prompt: "Test".to_string(),
        user_query: "Test".to_string(),
        context_files: vec![],
        history: vec![],
    };

    let result = client.stream_completion(req).await;

    // Should return an error, not panic
    assert!(result.is_err(), "Should return error for bad connection");

    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("Failed to connect") || error_msg.contains("Ollama"),
            "Error message should be descriptive: {}",
            error_msg
        );
    }
}

// Note: To run these tests, you need:
// 1. Install Ollama: https://ollama.ai
// 2. Start Ollama: ollama serve
// 3. Pull a model: ollama pull llama3
// 4. Run tests: cargo test --test ai_connectivity -- --ignored --nocapture
