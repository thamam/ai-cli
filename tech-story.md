This is a design challenge that requires balancing the **Unix philosophy** (do one thing well, text streams) with **Modern AI UX** (context-aware, multimodal, forgiving).

We are not building a new terminal. We are building a **Shell Overlay**—a tool that lives between the user and the shell (Bash/Zsh/Fish).

Here is the product design for **"AETHER"**.

***

## Product Identity: AETHER
**Tagline:** The Neural Fabric for your Shell.
**Core Philosophy:** "Don't leave the flow."

Aether is a TUI (Text User Interface) binary, written in Rust for millisecond latency. It is not a shell itself, but it hooks into your existing shell (Zsh/Fish/Bash) via a global hotkey (e.g., `Ctrl + Space`) to overlay an AI command center on top of Ghostty, Terminator, or iTerm2.

---

## The 3 Core Modes of Operation

To solve developer pain points, Aether operates in three distinct modes: **The Lens (Overlay)**, **The Pipe (Stream)**, and **The Sentinel (Background)**.

### 1. The Lens (The "Raycast/Spotlight" for CLI)
**Pain Point:** Forgetting specific flags (tar, ffmpeg, kubectl), context switching to Google/ChatGPT to write a command.

**UX Solution:**
When you press `Ctrl+Space`, Aether captures the terminal focus and renders a high-performance TUI modal over your current cursor position.

*   **Natural Language to Shell:** You type: *"Find all python files modified yesterday and tar them."*
    *   Aether displays the command: `find . -name "*.py" -mtime -1 | xargs tar -cvf archive.tar`
    *   **The "Spinning Head" Feature:** Aether provides a **"Dry Run" breakdown**. It uses an AST (Abstract Syntax Tree) parser to visualize exactly what the command will do before you run it (e.g., *"This will create a file named archive.tar containing 4 files"*).
*   **Semantic History Search:** Instead of `Ctrl+R` matching exact strings, Aether searches by intent.
    *   *Query:* "That time I fixed the docker network" -> *Result:* `docker network inspect bridge...`

### 2. The Pipe (The Unix Philosphy 2.0)
**Pain Point:** Parsing complex JSON/Logs, writing regex, using `awk/sed` when you aren't a wizard.

**UX Solution:**
Aether introduces a reserved command `ae`. You pipe data into it, and give it instructions in plain English.

*   **Example:** `cat server_logs.txt | ae "Group error messages by type and show a bar chart"`
*   **The Output:** Aether doesn't just text-vomit. It detects the output format. If you asked for a chart, it renders a **TUI ASCII Bar Chart** right there in the terminal using the piped data.
*   **Example:** `curl api.json | ae "extract the user emails as a CSV"` -> Output is clean CSV data.

### 3. The Sentinel (The Error Interceptor)
**Pain Point:** Cryptic compiler errors, stack traces, or the dreaded `Command failed`.

**UX Solution:**
Aether hooks into the shell's `PRECMD` (before prompt re-draw). If the previous exit code was non-zero, Aether subtly glows the prompt red.

*   **The "Fix It" Button:** You type `??` (or a mapped shortcut).
*   Aether analyzes the `stderr` buffer, reads the code file mentioned in the stack trace (context awareness), and suggests a patch.
*   **UX Magic:** It offers a **"git-style diff"** view. You can see exactly what it wants to change in your config file or code to fix the error, and press `Enter` to apply the patch directly to the file.

---

## The "Head-Spinning" Features

These are the differentiators that will make developers install Aether immediately.

### 1. Context-Aware RAG (Retrieval-Augmented Generation)
Aether indexes the current directory (honoring `.gitignore`).
*   **Scenario:** You ask, "Why isn't the login route working?"
*   Aether doesn't just hallucinate. It reads your `routes.ts` and `auth_controller.ts` and says: *"You are passing the JWT token in the body, but your controller expects it in the header."*
*   **Why it wins:** It acts like a Senior Dev sitting next to you, not a generic chatbot.

### 2. The "Suicide-Linux" Prevention Mechanism
Before executing high-risk commands (`rm -rf /`, `DROP DATABASE`, `kubectl delete namespace prod`), Aether intercepts execution.
*   It flashes an amber warning.
*   **AI Simulation:** It tells you: *"Warning: This command will delete 45,000 files including your operating system bootloader."*
*   It requires a semantic confirmation (e.g., "Type the name of the current directory to confirm") rather than just `y/n`.

### 3. Local-First Privacy (Ollmama Integration)
Developers are paranoid about sending proprietary code to OpenAI/Anthropic.
*   Aether detects if **Ollama** is running locally.
*   If yes, it defaults to a high-speed local model (like `Llama-3-8b-Instruct`) for zero-latency, offline, fully private assistance.
*   It switches to the cloud API only for "Heavy Reasoning" tasks when explicitly requested.

### 4. "Ghost-Typing" Tutorials
Instead of just dumping a script, Aether has a **"Teach Me"** mode.
*   It types the command out character by character (ghost text) and adds comments above the flags explaining what they do *as it types them*.
*   Great for junior devs onboarding onto complex infrastructure.

---

## The Visual Design (TUI)

Since this runs inside Ghostty/Terminator, it must look native but futuristic.

*   **Aesthetics:** Cyberpunk-lite. Minimalist borders. Uses the user's current terminal color scheme (reading ANSI colors) but adds a subtle glow effect using high-refresh-rate rendering (via libraries like `Ratatui` for Rust or `Bubble Tea` for Go).
*   **Animation:** When thinking, it doesn't use a spinner. It uses a "digital rain" or a waveform that reacts to the complexity of the query.

## Implementation Strategy (The "Sticky" Adoption)

To get mass adoption, friction must be zero.

1.  **Install:** `curl -sL get.aether.sh | bash` (Single binary).
2.  **Setup:** It detects your shell (zsh/bash/fish) and injects one line into `.rc` file to enable the hotkey hook.
3.  **The "Wow" Moment:** On first run, it scans your history file (`.bash_history`) and generates a "Personality Profile" telling you what kind of dev you are (e.g., "You are a Git Power User who hates writing commit messages") and pre-configures aliases for you.

## Summary of the UX Flow

1.  **Developer is stuck.**
2.  **Hit `Ctrl+Space`.** A sleek glass-pane window appears over the terminal.
3.  **Type:** "Undo the last 3 git commits but keep changes."
4.  **Aether:** Shows `git reset --soft HEAD~3`.
5.  **Developer:** Hits `Tab` to explain, or `Enter` to execute.
6.  **Aether:** Vanishes. The command runs in the native shell.

Aether solves the "Blank Canvas Paralysis" of the terminal without removing the power that makes the terminal great.
