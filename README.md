# AETHER - The Neural Fabric for your Shell

> **Status:** Phase 1 Complete ✅ - TUI Framework & Scaffolding

AETHER is a Rust-based TUI (Text User Interface) tool that acts as an AI-powered shell overlay, providing intelligent assistance without replacing your existing shell (Bash/Zsh/Fish).

## Project Philosophy

**"Don't leave the flow."**

AETHER operates as a shell overlay triggered by hotkey (Ctrl+Space), providing AI assistance while keeping you in your terminal workflow.

## Core Features (Planned)

### 1. The Lens (Raycast for CLI)
- **Natural language to shell commands**: Type what you want, get the right command
- **Semantic history search**: Search your history by intent, not just exact matches
- **Dry-run visualization**: See what a command will do before running it

### 2. The Pipe (Unix Philosophy 2.0)
- **AI-powered data processing**: `cat data.json | ae "extract emails as CSV"`
- **Smart output formatting**: Automatic charts, tables, and data visualization
- **Context-aware parsing**: Understands JSON, CSV, logs, and more

### 3. The Sentinel (Error Interceptor)
- **Automatic error analysis**: Type `??` after a failed command
- **Code-aware fixes**: Reads relevant files and suggests patches
- **Git-style diff view**: Review and apply fixes with a single keystroke

## Architecture

### Tech Stack
- **Language:** Rust 2021 Edition
- **TUI:** `ratatui` + `crossterm`
- **Async:** `tokio`
- **AI Clients:** Ollama (local-first), OpenAI, Anthropic, Gemini
- **File Walking:** `ignore` crate (gitignore-aware, same as ripgrep)

### Module Structure

```
src/
├── main.rs              # Entry point, CLI parsing
├── tui/                 # UI layer (ratatui)
│   ├── components/      # Reusable widgets
│   ├── layout.rs        # Layout definitions
│   ├── renderer.rs      # Main render loop
│   └── state.rs         # Application state machine
├── ai/                  # LLM integration
│   ├── client.rs        # AIProvider trait
│   ├── ollama.rs        # Ollama client
│   ├── openai.rs        # OpenAI client (TODO)
│   ├── anthropic.rs     # Anthropic client (TODO)
│   ├── gemini.rs        # Gemini client (TODO)
│   └── prompts.rs       # System prompts
├── core/                # Business logic
│   ├── config.rs        # Configuration management
│   └── executor.rs      # Command execution & safety
├── context/             # RAG & environment awareness
│   ├── fs_scanner.rs    # Gitignore-aware file walker
│   └── shell.rs         # Shell history & context
└── shell_integration/   # Shell hooks
    ├── zsh.sh
    └── bash.sh
```

## Implementation Status

### ✅ Phase 1: TUI Foundation (COMPLETE)
- [x] Project structure with Cargo.toml
- [x] Basic TUI with ratatui (glass pane overlay)
- [x] AppState and modal system (Input/Thinking/Review/Diff)
- [x] Component widgets (InputBox, ThinkingAnimation, CommandReview, DiffView)
- [x] Event handling and keyboard navigation
- [x] Shell integration script generation
- [x] Config schema and loader
- [x] Unit and integration tests

**Current Status:** The TUI framework compiles and runs. Basic input/output cycle works with dummy responses.

### 🚧 Phase 2: Shell Integration (TODO)
- [ ] Finalize Zsh/Bash shell hooks
- [ ] Context capture mechanism (/tmp/aether/context.json)
- [ ] Pipe mode implementation (`ae` command)
- [ ] E2E tests for shell integration

### 🚧 Phase 3: AI Integration (TODO)
- [ ] AIProvider trait abstraction
- [ ] Ollama client with streaming
- [ ] OpenAI client
- [ ] Gemini client
- [ ] Anthropic client
- [ ] Context engine (RAG)
- [ ] System prompts and templates
- [ ] Connect TUI to AI streaming
- [ ] E2E tests for AI integration

### 🚧 Phase 4: Sentinel & Safety (TODO)
- [ ] Sentinel mode (error detection)
- [ ] DiffView with syntax highlighting
- [ ] Safety checks for destructive commands
- [ ] Dry-run visualizer
- [ ] E2E tests for Sentinel mode

## Building & Testing

### Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Test Shell Integration
```bash
# Generate Zsh integration script
cargo run -- inject zsh

# Generate Bash integration script
cargo run -- inject bash
```

### Try the TUI (Phase 1)
```bash
# Run lens mode (will show dummy responses)
cargo run -- --mode lens

# Or use the lens subcommand
cargo run -- lens
```

## Installation (Not Yet Ready)

Once complete, installation will be:

```bash
# Download and install
curl -sL get.aether.sh | bash

# Add to your shell config
aether inject zsh >> ~/.zshrc
# or
aether inject bash >> ~/.bashrc

# Restart shell
source ~/.zshrc
```

## Configuration

Configuration file: `~/.config/aether/config.toml`

Example:
```toml
[ai]
default_provider = "ollama"

[ai.ollama]
enabled = true
base_url = "http://localhost:11434"
model = "llama3"

[ai.openai]
enabled = false
api_key = "sk-..."
model = "gpt-4"

[ui]
modal_width_percent = 80
modal_height_percent = 70
animations = true

[safety]
detect_destructive_commands = true
confirm_destructive = true
default_dry_run = false
```

## Development Roadmap

1. **Phase 1 ✅**: TUI framework and scaffolding
2. **Phase 2**: Shell integration and pipe mode
3. **Phase 3**: AI provider implementations
4. **Phase 4**: Sentinel mode and safety features
5. **Phase 5**: Polish, documentation, and release

## Testing Strategy

Following **Eval-Driven Design**:
- **E2E tests**: Primary focus for user-facing functionality
- **Unit tests**: For complex logic where E2E is impractical
- **Integration tests**: Shell integration, AI providers

## Contributing

This project is in active development. Contributions welcome!

## License

MIT

---

**Built with Rust 🦀 for blazing-fast terminal performance** 
