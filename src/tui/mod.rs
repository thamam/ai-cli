mod components;
mod layout;
mod renderer;
mod state;

pub use renderer::run_lens_mode;
pub use state::{AppMode, AppState};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub type CrosstermTerminal = Terminal<CrosstermBackend<io::Stdout>>;

/// Initialize the terminal for TUI rendering
pub fn init_terminal() -> Result<CrosstermTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore_terminal(terminal: &mut CrosstermTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Handle keyboard events
pub fn handle_key_event(
    state: &mut AppState,
    key_code: KeyCode,
    modifiers: KeyModifiers,
) -> bool {
    match state.mode {
        AppMode::Input => match key_code {
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                return true; // Exit
            }
            KeyCode::Esc => {
                return true; // Exit
            }
            KeyCode::Enter => {
                if !state.input_buffer.is_empty() {
                    state.mode = AppMode::Thinking;
                    state.should_process_query = true;
                }
            }
            KeyCode::Char(c) => {
                state.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                state.input_buffer.pop();
            }
            _ => {}
        },
        AppMode::Review => match key_code {
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                return true; // Exit
            }
            KeyCode::Esc => {
                return true; // Exit without executing
            }
            KeyCode::Enter => {
                state.should_execute = true;
                return true; // Exit and execute
            }
            KeyCode::Tab => {
                state.show_explanation = !state.show_explanation;
            }
            _ => {}
        },
        AppMode::Thinking => {
            // Can't interrupt thinking mode for now
            if key_code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                return true;
            }
        }
        AppMode::Diff => match key_code {
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                return true; // Exit
            }
            KeyCode::Esc => {
                return true; // Exit without applying
            }
            KeyCode::Enter => {
                state.should_apply_patch = true;
                return true; // Exit and apply patch
            }
            KeyCode::Up => {
                state.scroll_position = state.scroll_position.saturating_sub(1);
            }
            KeyCode::Down => {
                state.scroll_position = state.scroll_position.saturating_add(1);
            }
            _ => {}
        },
    }
    false
}
