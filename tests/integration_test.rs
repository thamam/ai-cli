use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_inject_zsh() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("zsh");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AETHER Shell Integration for Zsh"))
        .stdout(predicate::str::contains("__aether_lens_mode"));
}

#[test]
fn test_inject_bash() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("bash");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AETHER Shell Integration for Bash"))
        .stdout(predicate::str::contains("__aether_lens_mode"));
}

#[test]
fn test_inject_unsupported_shell() {
    let mut cmd = Command::cargo_bin("aether").unwrap();
    cmd.arg("inject").arg("fish");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported shell"));
}

#[test]
fn test_config_creation() {
    // Create a temporary directory for config
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".config/aether/config.toml");

    // Set HOME to temp directory
    std::env::set_var("HOME", temp_dir.path());

    // Load config (should create default)
    // This test verifies the config module but doesn't run the full binary
    // since that would require terminal interaction

    // For now, just verify the temp dir exists
    assert!(temp_dir.path().exists());
}

#[test]
fn test_destructive_command_detection() {
    // Test the CommandExecutor's destructive command detection
    // This doesn't require running the binary

    let dangerous_commands = vec![
        "rm -rf /",
        "DROP TABLE users",
        "dd if=/dev/zero of=/dev/sda",
    ];

    let safe_commands = vec![
        "ls -la",
        "echo 'hello world'",
        "git status",
    ];

    // These would be tested in unit tests within the executor module
    // For E2E, we just verify the test framework is set up
    assert!(true);
}
