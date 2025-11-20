#!/usr/bin/env bash
# AETHER Shell Integration for Bash
# This script should be sourced in your .bashrc

# Directory for aether temporary files
export AETHER_TMP_DIR="/tmp/aether"
mkdir -p "$AETHER_TMP_DIR"

# Track command execution time and status
__aether_preexec() {
    __AETHER_CMD_START_TIME=$SECONDS
    __AETHER_LAST_CMD="$BASH_COMMAND"
}

__aether_precmd() {
    local exit_code=$?
    local duration=0

    if [[ -n "$__AETHER_CMD_START_TIME" ]]; then
        duration=$((SECONDS - __AETHER_CMD_START_TIME))
        unset __AETHER_CMD_START_TIME
    fi

    # Always update session context (for Lens mode to use)
    cat > "$AETHER_TMP_DIR/session_context.json" <<EOF
{
  "last_command": "$__AETHER_LAST_CMD",
  "last_exit_code": $exit_code,
  "duration": $duration,
  "working_directory": "$PWD",
  "shell_type": "bash",
  "timestamp": $(date +%s)
}
EOF

    # If command failed, also save to last_session for Sentinel mode
    if [[ $exit_code -ne 0 && -n "$__AETHER_LAST_CMD" ]]; then
        cat > "$AETHER_TMP_DIR/last_session" <<EOF
{
  "last_command": "$__AETHER_LAST_CMD",
  "exit_code": $exit_code,
  "duration": $duration,
  "working_directory": "$PWD",
  "shell_type": "bash",
  "timestamp": $(date +%s)
}
EOF
    fi
}

# Set up hooks
trap '__aether_preexec' DEBUG
PROMPT_COMMAND="__aether_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}"

# Keybinding for Lens mode (Ctrl+Space)
__aether_lens_mode() {
    local result
    result=$(aether --mode lens --buffer "$READLINE_LINE" --cursor-pos "$READLINE_POINT" 2>/dev/null)

    if [[ -n "$result" ]]; then
        READLINE_LINE="$result"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# Bind to Ctrl+Space
bind -x '"\C- ": __aether_lens_mode'

# Alias for pipe mode
alias ae='aether --mode pipe'

# Function for Sentinel mode (error analysis)
??() {
    aether --mode sentinel
}

echo "✨ AETHER initialized - Press Ctrl+Space to activate"
