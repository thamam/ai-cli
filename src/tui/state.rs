use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppMode {
    /// User is typing a query
    Input,
    /// Waiting for AI response (with animation)
    Thinking,
    /// Reviewing the suggested command or code (dry run mode)
    Review,
    /// Viewing a diff for patch application (Sentinel mode)
    Diff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[derive(Debug)]
pub struct AppState {
    /// Current input buffer
    pub input_buffer: String,

    /// Current application mode
    pub mode: AppMode,

    /// Chat history
    pub chat_history: Vec<ChatMessage>,

    /// AI-suggested command (if any)
    pub suggested_command: Option<String>,

    /// Explanation of the command
    pub command_explanation: Option<String>,

    /// Diff content for Sentinel mode
    pub diff_content: Option<String>,

    /// Vertical scroll position for diff view
    pub scroll_position: u16,

    /// Whether to show explanation in Review mode
    pub show_explanation: bool,

    /// Flag to trigger query processing
    pub should_process_query: bool,

    /// Flag to trigger command execution
    pub should_execute: bool,

    /// Flag to trigger patch application
    pub should_apply_patch: bool,

    /// Animation frame counter
    pub animation_frame: u8,

    /// Initial buffer from shell (for lens mode)
    pub initial_buffer: String,

    /// Initial cursor position
    pub cursor_pos: usize,
}

impl AppState {
    pub fn new(initial_buffer: String, cursor_pos: usize) -> Self {
        Self {
            input_buffer: String::new(),
            mode: AppMode::Input,
            chat_history: Vec::new(),
            suggested_command: None,
            command_explanation: None,
            diff_content: None,
            scroll_position: 0,
            show_explanation: false,
            should_process_query: false,
            should_execute: false,
            should_apply_patch: false,
            animation_frame: 0,
            initial_buffer,
            cursor_pos,
        }
    }

    pub fn add_message(&mut self, role: impl Into<String>, content: impl Into<String>) {
        self.chat_history.push(ChatMessage {
            role: role.into(),
            content: content.into(),
        });
    }

    pub fn reset_for_new_query(&mut self) {
        self.mode = AppMode::Input;
        self.suggested_command = None;
        self.command_explanation = None;
        self.diff_content = None;
        self.scroll_position = 0;
        self.show_explanation = false;
        self.should_process_query = false;
        self.should_execute = false;
        self.should_apply_patch = false;
    }
}
