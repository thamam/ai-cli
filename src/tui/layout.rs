use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Calculate the centered modal area (glass pane effect)
/// Returns a Rect that's 80% width and 60% height, centered on screen
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create the main layout for the lens mode
/// Returns [header, content, footer]
pub fn main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(area)
        .to_vec()
}

/// Create layout for review mode
/// Returns [command_area, explanation_area]
pub fn review_layout(area: Rect, show_explanation: bool) -> Vec<Rect> {
    if show_explanation {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Command
                Constraint::Min(5),    // Explanation
            ])
            .split(area)
            .to_vec()
    } else {
        vec![area]
    }
}
