This architecture is designed to be fed directly into an LLM code agent (like Cursor/Claude Dev/Windsurf). It prioritizes **Rust** for performance/safety and uses a modular, trait-based architecture to ensure the code is testable and extensible.

### System Architecture: Project "AETHER"

**Tech Stack:**
*   **Language:** Rust (2021 Edition)
*   **TUI Engine:** `ratatui` (UI), `crossterm` (Events/Raw Mode)
*   **Async Runtime:** `tokio`
*   **LLM Client:** `reqwest` (or `async-openai` crate wrapper)
*   **Shell Integration:** Zsh/Bash/Fish scripts (embedded in binary)
*   **Local Embeddings (Optional for MVP):** `fastembed-rs`

---

### 1. High-Level Data Flow
The system does not run as a persistent daemon (to keep resource usage zero when idle). It utilizes a **State-on-Disk** architecture coupled with shell hooks.

1.  **Shell Hook (`pre-cmd` / `post-cmd`):** Captures exit codes, execution time, and stderr. Writes minimal snapshot to `/tmp/aether/context.json`.
2.  **Trigger (`Ctrl+Space`):** Shell binds hotkey to run `aether --mode lens`.
3.  **Aether Binary:**
    *   Loads context (history, file tree, last error).
    *   Renders TUI Overlay (Alternate Screen).
    *   Streams AI tokens.
    *   Exits and returns a buffer to the shell (command to run or nothing).

---

### 2. Crate & Module Structure
Instruct the LLM to create the following directory structure.

```text
aether/
├── Cargo.toml             # Workspace definition
├── src/
│   ├── main.rs            # Entry point, CLI args parsing (Clap)
│   ├── tui/               # UI Layer (Ratatui)
│   │   ├── mod.rs
│   │   ├── layout.rs      # Layout constraints
│   │   ├── components/    # Reusable widgets (Input, MarkdownView, DiffView)
│   │   └── renderer.rs    # Main render loop
│   ├── core/              # Business Logic
│   │   ├── mod.rs
│   │   ├── executor.rs    # Command execution/Dry-run simulation
│   │   └── config.rs      # Config loader (config.toml)
│   ├── ai/                # LLM Integration
│   │   ├── mod.rs
│   │   ├── client.rs      # Trait definition (ModelProvider)
│   │   ├── openai.rs      # OpenAI implementation
│   │   ├── ollama.rs      # Ollama implementation
│   │   └── prompts.rs     # System prompts and template logic
│   ├── context/           # RAG & Environment Awareness
│   │   ├── mod.rs
│   │   ├── fs_scanner.rs  # gitignore aware file walker
│   │   └── shell.rs       # Shell history and env var parser
│   └── shell_integration/ # Scripts to be injected into .rc files
│       ├── zsh.sh
│       └── bash.sh
```

---

### 3. Core Component Design (Rust Specifications)

#### A. The AI Abstraction Layer (`src/ai`)
We need a standardized interface so we can swap between GPT-4, Claude, or local Llama without breaking the UI.

```rust
// src/ai/client.rs
use async_trait::async_trait;
use futures::stream::BoxStream;

pub enum ModelType {
    Cloud(String), // e.g., "gpt-4o"
    Local(String), // e.g., "llama3"
}

#[derive(Debug)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub user_query: String,
    pub context_files: Vec<(String, String)>, // (filename, content)
    pub history: Vec<String>, // Last N shell commands
}

#[async_trait]
pub trait AIProvider {
    // Stream response for TUI "Digital Rain" effect
    async fn stream_completion(&self, req: CompletionRequest) -> anyhow::Result<BoxStream<'static, String>>;
    
    // For semantic analysis (Sentinel mode)
    async fn get_fix_suggestion(&self, error_log: String) -> anyhow::Result<String>;
}
```

#### B. The Context Engine (`src/context`)
This is crucial for the "RAG" capability. It shouldn't just dump the whole file system.

*   **`fs_scanner.rs`**: Use the `ignore` crate (same as `ripgrep` uses) to walk the directory efficiently, respecting `.gitignore`.
*   **Heuristic**:
    *   Limit file reads to text files < 100KB.
    *   If `git` is present, prioritize modified files (`git status`).
    *   **Token Budget**: Hard cap context sent to LLM at 8k-32k tokens (depending on model).

#### C. The TUI State Machine (`src/tui`)
The TUI needs to be modal.

```rust
// src/tui/state.rs
pub enum AppMode {
    Input,          // User typing query
    Thinking,       // Waiting for AI (Animation state)
    Review,         // Reviewing code/command (Dry Run)
    Diff,           // Viewing "Sentinel" patch suggestions
}

pub struct AppState {
    pub input_buffer: String,
    pub mode: AppMode,
    pub chat_history: Vec<ChatMessage>,
    pub suggested_command: Option<String>,
    pub diff_content: Option<String>,
    pub scroll_position: u16,
}
```

---

### 4. Shell Integration Architecture (The "Hook")
This is the mechanism that makes Aether feel native.

#### The Protocol (`src/shell_integration`)
Instead of trying to parse the shell's internal state from outside, we inject a small script that writes state to a temporary file.

**Bash/Zsh Script Logic:**
1.  **`preexec`**: Timestamp start time.
2.  **`precmd`**:
    *   Calculate duration.
    *   Capture exit code (`$?`).
    *   If exit code != 0, write the last command and the exit code to `/tmp/aether/last_session`.
3.  **Keybinding (`Ctrl+Space` or `Alt+a`)**:
    *   Runs: `BUFFER=$(aether --mode lens --cursor-pos $CURSOR --buffer "$BUFFER"); if [ -n "$BUFFER" ]; then READLINE_LINE="$BUFFER"; fi`
    *   *Note:* In Zsh this modifies the `BUFFER` variable directly.

---

### 5. Implementation Phases (For the Code Agent)

Instruct the code agent to build in this order to maintain testability:

#### Phase 1: The Skeleton & TUI
*   Set up `ratatui`.
*   Create the "Glass Pane" overlay (a bordered block centered on screen).
*   Implement the Input field handling.
*   *Goal:* You can run `./aether`, type text, and exit.

#### Phase 2: The Shell Integration
*   Write the Zsh/Bash shim scripts.
*   Implement the `aether --inject` command that prints the script.
*   Test piping data: `echo "hello" | aether --mode pipe`.

#### Phase 3: The Brain (AI Integration)
*   Implement `Ollama` client first (free/local for testing).
*   Connect the "Input" from Phase 1 to the AI client.
*   Implement the streaming response decoder.

#### Phase 4: The Sentinel (Error Handling)
*   Implement the logic to read `/tmp/aether/last_session`.
*   Create the "Diff View" widget in TUI using `syntect` for syntax highlighting.

---

### 6. Specific "Secret Sauce" Algorithms
Give these specific instructions to the LLM for the "Spinning Head" features:

**Algorithm 1: The Safety Check (Suicide Prevention)**
```rust
// Pseudo-code for the code agent
fn is_destructive(command: &str) -> bool {
    let dangerous_keywords = ["rm -rf", "drop table", "delete", "truncate"];
    // Use simple AST parsing or Regex
    // If sensitive, force an additional user interaction step in TUI
    // e.g., "Type 'confirm' to proceed"
}
```

**Algorithm 2: The Dry Run Visualizer**
*   Do not execute the command.
*   Use a library like `shell-parser` (if available in Rust) or basic heuristics.
*   If the command creates files, show a Tree View of the *projected* file structure.

### 7. Final Prompt for the LLM Agent
*Copy and paste this to your AI coding assistant:*

> "I need you to implement the 'Aether' CLI tool based on the following architecture. We will use Rust. Start by scaffolding the project structure with the defined workspaces.
>
> **Step 1:** Create the `Cargo.toml` and the module folder structure (`ai`, `core`, `tui`, `context`).
> **Step 2:** Implement the `Tui` trait using `ratatui` with a simple 'Query Mode' state.
> **Step 3:** Create the `AIProvider` trait in `ai/mod.rs` and a dummy implementation that returns echoed text after a delay.
>
> Please ensure all async code uses `tokio`. Do not implement the full logic yet, just the scaffold that compiles and runs a basic TUI window."
