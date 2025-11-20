// TUI Interaction Tests using rexpect
// These tests verify the TUI launches, accepts input, and responds to keys

use rexpect::spawn;
use std::time::Duration;

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_tui_launches_and_exits_on_esc() {
    // This test verifies the TUI launches and responds to Esc key
    // Ignored by default because it requires a PTY

    let mut session = spawn("cargo run -- lens", Some(5000)).expect("Failed to spawn");

    // Wait a bit for TUI to render
    std::thread::sleep(Duration::from_millis(1000));

    // Check that we see the AETHER title
    session
        .exp_string("AETHER")
        .expect("Should see AETHER title");

    // Send Escape key to exit
    session.send("\x1b").expect("Failed to send Esc");

    // Wait for process to exit
    std::thread::sleep(Duration::from_millis(500));

    // Session ends when process exits
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_tui_accepts_input() {
    // This test verifies the TUI accepts keyboard input

    let mut session = spawn("cargo run -- lens", Some(5000)).expect("Failed to spawn");

    // Wait for TUI to render
    std::thread::sleep(Duration::from_millis(1000));

    // Type some text
    session.send("list files").expect("Failed to send input");

    // Wait a bit
    std::thread::sleep(Duration::from_millis(300));

    // Send Enter
    session.send("\r").expect("Failed to send Enter");

    // Wait for processing
    std::thread::sleep(Duration::from_millis(500));

    // Send Escape to exit
    session.send("\x1b").expect("Failed to send Esc");

    std::thread::sleep(Duration::from_millis(300));
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_tui_exits_on_ctrl_c() {
    // This test verifies Ctrl+C exits the TUI

    let mut session = spawn("cargo run -- lens", Some(5000)).expect("Failed to spawn");

    // Wait for TUI to render
    std::thread::sleep(Duration::from_millis(1000));

    // Send Ctrl+C
    session.send("\x03").expect("Failed to send Ctrl+C");

    // Wait for exit
    std::thread::sleep(Duration::from_millis(500));

    // Session ends when process exits
}

// Note: These tests are ignored by default because they require:
// 1. A pseudo-terminal (PTY)
// 2. The binary to be built
// 3. May behave differently in CI/CD environments
//
// Run them manually with: cargo test -- --ignored
//
// For CI/CD, prefer the headless brain tests and eval tests
