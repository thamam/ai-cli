use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Custom input widget with glass-pane style
pub struct InputBox<'a> {
    content: &'a str,
    title: &'a str,
    focused: bool,
}

impl<'a> InputBox<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            title: "Query",
            focused: true,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for InputBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let input_text = if self.focused {
            format!("{}_", self.content)
        } else {
            self.content.to_string()
        };

        let paragraph = Paragraph::new(input_text)
            .block(block)
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);
    }
}

/// Loading animation widget (digital rain effect)
pub struct ThinkingAnimation {
    frame: u8,
}

impl ThinkingAnimation {
    pub fn new(frame: u8) -> Self {
        Self { frame }
    }

    pub fn get_animation_char(&self) -> &str {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        frames[(self.frame as usize) % frames.len()]
    }
}

impl Widget for ThinkingAnimation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Thinking")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let animation_char = self.get_animation_char();
        let text = format!("{} Processing your query...", animation_char);

        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::Magenta))
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

/// Command review widget
pub struct CommandReview<'a> {
    command: &'a str,
    explanation: Option<&'a str>,
    show_explanation: bool,
}

impl<'a> CommandReview<'a> {
    pub fn new(command: &'a str) -> Self {
        Self {
            command,
            explanation: None,
            show_explanation: false,
        }
    }

    pub fn explanation(mut self, explanation: &'a str, show: bool) -> Self {
        self.explanation = Some(explanation);
        self.show_explanation = show;
        self
    }
}

impl Widget for CommandReview<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Command Preview")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let mut lines = vec![
            Line::from(vec![
                Span::styled("$ ", Style::default().fg(Color::Yellow)),
                Span::styled(self.command, Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
        ];

        if self.show_explanation {
            if let Some(explanation) = self.explanation {
                lines.push(Line::from(Span::styled(
                    "Explanation:",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(explanation));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "Press Tab to toggle explanation",
                Style::default().fg(Color::Gray),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Enter: Execute | Esc: Cancel",
            Style::default().fg(Color::Gray),
        )));

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

/// Diff view widget for Sentinel mode
pub struct DiffView<'a> {
    diff_content: &'a str,
    scroll_position: u16,
}

impl<'a> DiffView<'a> {
    pub fn new(diff_content: &'a str, scroll_position: u16) -> Self {
        Self {
            diff_content,
            scroll_position,
        }
    }
}

impl Widget for DiffView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Suggested Patch")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let lines: Vec<Line> = self
            .diff_content
            .lines()
            .skip(self.scroll_position as usize)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| {
                let style = if line.starts_with('+') {
                    Style::default().fg(Color::Green)
                } else if line.starts_with('-') {
                    Style::default().fg(Color::Red)
                } else if line.starts_with("@@") {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(line, style))
            })
            .collect();

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text).block(block);

        paragraph.render(area, buf);
    }
}

/// Help footer widget
pub struct HelpFooter;

impl Widget for HelpFooter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": Exit | "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::raw(": Quit | "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Submit"),
        ]);

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));

        paragraph.render(area, buf);
    }
}
