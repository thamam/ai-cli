use super::{
    components::{CommandReview, DiffView, HelpFooter, InputBox, ThinkingAnimation},
    handle_key_event, init_terminal, layout,
    restore_terminal,
    state::{AppMode, AppState},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

/// Main entry point for Lens mode (Raycast-like overlay)
pub async fn run_lens_mode(initial_buffer: String, cursor_pos: usize) -> Result<()> {
    let mut terminal = init_terminal()?;
    let mut state = AppState::new(initial_buffer, cursor_pos);

    let result = run_app(&mut terminal, &mut state).await;

    restore_terminal(&mut terminal)?;

    // If user pressed Enter to execute, print the command to stdout
    // so the shell can capture it
    if state.should_execute {
        if let Some(cmd) = state.suggested_command {
            println!("{}", cmd);
        }
    }

    result
}

async fn run_app(
    terminal: &mut super::CrosstermTerminal,
    state: &mut AppState,
) -> Result<()> {
    loop {
        // Render the UI
        terminal.draw(|f| render_ui(f, state))?;

        // Handle events with timeout for animation
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(state, key.code, key.modifiers) {
                    break; // Exit requested
                }
            }
        }

        // Update animation frame
        state.animation_frame = state.animation_frame.wrapping_add(1);

        // Process query if needed (Phase 3 - AI integration)
        if state.should_process_query {
            state.should_process_query = false;
            // TODO: Phase 3 - Call AI provider here
            // For now, just echo the query as a dummy command
            process_dummy_query(state);
        }
    }

    Ok(())
}

/// Render the appropriate UI based on the current mode
fn render_ui(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // Create centered glass pane overlay
    let modal_area = layout::centered_rect(80, 70, size);

    // Render the modal background
    render_modal_background(f, modal_area);

    // Render content based on mode
    match state.mode {
        AppMode::Input => render_input_mode(f, modal_area, state),
        AppMode::Thinking => render_thinking_mode(f, modal_area, state),
        AppMode::Review => render_review_mode(f, modal_area, state),
        AppMode::Diff => render_diff_mode(f, modal_area, state),
    }
}

fn render_modal_background(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " AETHER - The Neural Fabric for your Shell ",
            Style::default().fg(Color::Cyan),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
}

fn render_input_mode(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = layout::main_layout(area);

    // Header
    let header = Paragraph::new("Type your query in natural language")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(header, chunks[0]);

    // Input box
    let input_widget = InputBox::new(&state.input_buffer)
        .title("Query")
        .focused(true);
    f.render_widget(input_widget, chunks[1]);

    // Footer
    f.render_widget(HelpFooter, chunks[2]);
}

fn render_thinking_mode(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = layout::main_layout(area);

    // Query display
    let query_display = Paragraph::new(format!("Query: {}", state.input_buffer))
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        );
    f.render_widget(query_display, chunks[0]);

    // Thinking animation
    let animation = ThinkingAnimation::new(state.animation_frame);
    f.render_widget(animation, chunks[1]);

    // Footer
    f.render_widget(HelpFooter, chunks[2]);
}

fn render_review_mode(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = layout::main_layout(area);

    // Query display
    let query_display = Paragraph::new(format!("Query: {}", state.input_buffer))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(query_display, chunks[0]);

    // Command review
    if let Some(command) = &state.suggested_command {
        let mut review = CommandReview::new(command);
        if let Some(explanation) = &state.command_explanation {
            review = review.explanation(explanation, state.show_explanation);
        }
        f.render_widget(review, chunks[1]);
    }

    // Footer with additional help
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": Execute | "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": Toggle Explanation | "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": Cancel"),
    ]);
    let footer = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

fn render_diff_mode(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = layout::main_layout(area);

    // Header
    let header = Paragraph::new("Review the suggested patch")
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(header, chunks[0]);

    // Diff view
    if let Some(diff) = &state.diff_content {
        let diff_widget = DiffView::new(diff, state.scroll_position);
        f.render_widget(diff_widget, chunks[1]);
    }

    // Footer with controls
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": Apply | "),
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(": Scroll | "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": Cancel"),
    ]);
    let footer = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

/// Dummy query processor for Phase 1 testing
/// In Phase 3, this will be replaced with actual AI integration
fn process_dummy_query(state: &mut AppState) {
    // For now, just echo the input as a simple shell command
    let dummy_command = format!("echo 'You asked: {}'", state.input_buffer);
    let dummy_explanation = format!(
        "This is a dummy command that echoes your query: '{}'",
        state.input_buffer
    );

    state.suggested_command = Some(dummy_command);
    state.command_explanation = Some(dummy_explanation);
    state.mode = AppMode::Review;

    // Add to chat history
    let user_query = state.input_buffer.clone();
    let assistant_response = state.suggested_command.clone().unwrap();
    state.add_message("user", user_query);
    state.add_message("assistant", assistant_response);
}
