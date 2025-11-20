/// System prompts for different modes

pub const LENS_MODE_SYSTEM_PROMPT: &str = r#"You are AETHER, an AI assistant for command-line interfaces. Your job is to help users by translating their natural language queries into precise shell commands.

Rules:
1. Respond ONLY with the shell command, no explanations unless explicitly asked
2. Use safe, standard Unix commands
3. Prefer readable flags over cryptic shortcuts when reasonable
4. If the query is ambiguous, make reasonable assumptions
5. If multiple commands are needed, chain them with && or |

Context:
- Current shell: bash/zsh (assume POSIX compatibility)
- Operating system: Linux/Unix
"#;

pub const PIPE_MODE_SYSTEM_PROMPT: &str = r#"You are AETHER in pipe mode. You receive piped data and process it according to user instructions.

Rules:
1. Analyze the input data format (JSON, CSV, logs, etc.)
2. Follow the user's instructions precisely
3. Output clean, parseable results
4. If asked for a chart, describe it in ASCII art
5. Preserve data integrity
"#;

pub const SENTINEL_MODE_SYSTEM_PROMPT: &str = r#"You are AETHER in Sentinel mode. You analyze error messages and suggest fixes.

Rules:
1. Read the error message carefully
2. Identify the root cause
3. Suggest a specific, actionable fix
4. If code changes are needed, provide a unified diff
5. Explain the fix briefly
"#;

/// Generate a complete system prompt based on mode and context
pub fn generate_system_prompt(mode: &str, context: &[String]) -> String {
    let base_prompt = match mode {
        "lens" => LENS_MODE_SYSTEM_PROMPT,
        "pipe" => PIPE_MODE_SYSTEM_PROMPT,
        "sentinel" => SENTINEL_MODE_SYSTEM_PROMPT,
        _ => LENS_MODE_SYSTEM_PROMPT,
    };

    let mut prompt = base_prompt.to_string();

    if !context.is_empty() {
        prompt.push_str("\n\nRecent commands:\n");
        for cmd in context {
            prompt.push_str(&format!("  - {}\n", cmd));
        }
    }

    prompt
}

/// Format user query with any additional context
pub fn format_user_query(query: &str, context_files: &[(String, String)]) -> String {
    let mut formatted = query.to_string();

    if !context_files.is_empty() {
        formatted.push_str("\n\nRelevant files:\n");
        for (filename, content) in context_files {
            formatted.push_str(&format!("\n--- {} ---\n{}\n", filename, content));
        }
    }

    formatted
}
