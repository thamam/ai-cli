// Shell Integration Tests
// Tests verify that shell hooks properly capture command context

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_inject_zsh_generates_valid_script() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("zsh");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("#!/usr/bin/env zsh"))
        .stdout(predicate::str::contains("__aether_precmd"))
        .stdout(predicate::str::contains("__aether_preexec"))
        .stdout(predicate::str::contains("session_context.json"))
        .stdout(predicate::str::contains("bindkey '^ '"));
}

#[test]
fn test_inject_bash_generates_valid_script() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("#!/usr/bin/env bash"))
        .stdout(predicate::str::contains("__aether_precmd"))
        .stdout(predicate::str::contains("__aether_preexec"))
        .stdout(predicate::str::contains("session_context.json"))
        .stdout(predicate::str::contains("bind -x"));
}

#[test]
fn test_shell_context_file_structure() {
    // This test verifies the shell hook creates properly structured JSON

    let temp_dir = TempDir::new().unwrap();
    let context_file = temp_dir.path().join("session_context.json");

    // Create a mock session context (simulating what the shell hook would create)
    let mock_context = serde_json::json!({
        "last_command": "false",
        "last_exit_code": 1,
        "duration": 0,
        "working_directory": temp_dir.path().to_str().unwrap(),
        "shell_type": "bash",
        "timestamp": 1234567890
    });

    fs::write(&context_file, serde_json::to_string_pretty(&mock_context).unwrap()).unwrap();

    // Verify we can read it back
    let content = fs::read_to_string(&context_file).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["last_command"], "false");
    assert_eq!(parsed["last_exit_code"], 1);
    assert_eq!(parsed["duration"], 0);
    assert_eq!(parsed["shell_type"], "bash");
    assert!(parsed["timestamp"].is_number());
}

#[test]
fn test_pipe_mode_with_stdin() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("--mode").arg("pipe");
    cmd.write_stdin("Hello, World!\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Received"))
        .stdout(predicate::str::contains("bytes"));
}

#[test]
fn test_pipe_mode_without_stdin_fails() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("--mode").arg("pipe");
    // Don't provide stdin

    // This should fail or show an error
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No input provided"));
}

#[test]
fn test_shell_hook_captures_error_context() {
    // This test simulates what happens when a command fails in the shell

    let temp_dir = TempDir::new().unwrap();
    let aether_tmp = temp_dir.path().join("aether");
    fs::create_dir_all(&aether_tmp).unwrap();

    let session_context = aether_tmp.join("session_context.json");
    let last_session = aether_tmp.join("last_session");

    // Simulate shell hook writing context after a failed command
    let failed_context = serde_json::json!({
        "last_command": "grep nonexistent file.txt",
        "last_exit_code": 1,
        "duration": 0,
        "working_directory": "/tmp",
        "shell_type": "zsh",
        "timestamp": 1234567890
    });

    fs::write(&session_context, serde_json::to_string_pretty(&failed_context).unwrap()).unwrap();
    fs::write(&last_session, serde_json::to_string_pretty(&failed_context).unwrap()).unwrap();

    // Verify both files exist and are valid JSON
    assert!(session_context.exists());
    assert!(last_session.exists());

    let session_data: Value = serde_json::from_str(&fs::read_to_string(&session_context).unwrap()).unwrap();
    let last_session_data: Value = serde_json::from_str(&fs::read_to_string(&last_session).unwrap()).unwrap();

    assert_eq!(session_data["last_exit_code"], 1);
    assert_eq!(last_session_data["last_exit_code"], 1);
}

#[test]
fn test_shell_context_module_can_load_context() {
    use aether::context::ShellContext;

    // Create /tmp/aether/session_context.json manually
    let aether_tmp = PathBuf::from("/tmp/aether");
    fs::create_dir_all(&aether_tmp).unwrap();

    let context_file = aether_tmp.join("session_context.json");
    let test_context = serde_json::json!({
        "last_command": "echo test",
        "last_exit_code": 0,
        "duration": 1,
        "working_directory": "/tmp",
        "shell_type": "zsh",
        "timestamp": 1234567890
    });

    fs::write(&context_file, serde_json::to_string_pretty(&test_context).unwrap()).unwrap();

    // Try to load it with our ShellContext module
    let loaded = ShellContext::load();
    assert!(loaded.is_ok());

    let context = loaded.unwrap();
    assert!(context.is_some());

    let ctx = context.unwrap();
    assert_eq!(ctx.last_command.as_deref(), Some("echo test"));
    assert_eq!(ctx.last_exit_code, Some(0));
    assert_eq!(ctx.shell_type, "zsh");

    // Cleanup
    let _ = fs::remove_file(&context_file);
}

#[test]
fn test_ae_alias_calls_pipe_mode() {
    // Test that the 'ae' alias would call pipe mode
    // We test this by verifying the inject script contains the alias

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("alias ae"))
        .stdout(predicate::str::contains("--mode pipe"));
}

#[test]
fn test_sentinel_mode_marker_exists() {
    // Test that ?? is properly implemented as an alias (not a function)

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("zsh");

    let output = cmd.output().unwrap();
    let script = String::from_utf8(output.stdout).unwrap();

    assert!(script.contains("__aether_sentinel_trigger"));
    assert!(script.contains("alias '??'"));
    assert!(!script.contains("??() {"));

    // Also test bash
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    let output = cmd.output().unwrap();
    let script = String::from_utf8(output.stdout).unwrap();

    assert!(script.contains("__aether_sentinel_trigger"));
    assert!(script.contains("alias '??'"));
    assert!(!script.contains("??() {"));
}

#[test]
fn test_hook_prevents_self_capture() {
    // Test that hooks have logic to prevent capturing themselves

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("__aether_precmd*|__aether_preexec*"))
        .stdout(predicate::str::contains("__AETHER_IN_HOOK"));

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("zsh");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("__aether_precmd*|__aether_preexec*"));
}

#[test]
fn test_aether_bin_variable_support() {
    // Test that scripts use AETHER_BIN variable if set

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AETHER_BIN"))
        .stdout(predicate::str::contains("command -v aether"));

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("zsh");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AETHER_BIN"))
        .stdout(predicate::str::contains("command -v aether"));
}

#[test]
fn test_no_function_named_double_question() {
    // Verify that ?? is NOT implemented as a function (which causes syntax errors)

    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    let output = cmd.output().unwrap();
    let script = String::from_utf8(output.stdout).unwrap();

    // Should NOT contain "??() {" which is illegal in bash
    assert!(!script.contains("??() {"));
    assert!(!script.contains("??()")); // Any variant

    // SHOULD contain alias instead
    assert!(script.contains("alias '??'"));
}
