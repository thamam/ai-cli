#!/usr/bin/env bash
# AETHER Shell Integration for Bash
# This script should be sourced in your .bashrc

# Directory for aether temporary files
export AETHER_TMP_DIR="/tmp/aether"
mkdir -p "$AETHER_TMP_DIR"

# Track command execution time and status
__aether_preexec() {
    # Ignore if we're in a hook (prevent self-capture)
    [[ "$__AETHER_IN_HOOK" == "1" ]] && return

    # Ignore hook commands themselves
    case "$BASH_COMMAND" in
        __aether_precmd*|__aether_preexec*) return ;;
    esac

    __AETHER_CMD_START_TIME=$SECONDS
    __AETHER_LAST_CMD="$BASH_COMMAND"
}

__aether_precmd() {
    # CRITICAL: Must capture exit code FIRST before any other commands
    local exit_code=$?

    # Now safe to set other variables
    __AETHER_IN_HOOK=1

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

    __AETHER_IN_HOOK=0  # Clear hook flag
}

# Set up hooks
# Important: Use a variable to track if we're in a hook to avoid self-capture
__AETHER_IN_HOOK=0

trap '__aether_preexec' DEBUG
PROMPT_COMMAND="__aether_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}"

# Keybinding for Lens mode (Ctrl+Space)
__aether_lens_mode() {
    local result
    # Use absolute path if AETHER_BIN is set, otherwise try to find aether
    local aether_cmd="${AETHER_BIN:-$(command -v aether || echo aether)}"
    result=$("$aether_cmd" --mode lens --buffer "$READLINE_LINE" --cursor-pos "$READLINE_POINT" 2>/dev/null)

    if [[ -n "$result" ]]; then
        READLINE_LINE="$result"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# Bind to Ctrl+Space
bind -x '"\C- ": __aether_lens_mode'

# Alias for pipe mode
# Use absolute path if AETHER_BIN is set, otherwise use aether from PATH
if [[ -n "$AETHER_BIN" ]]; then
    alias ae="$AETHER_BIN --mode pipe"
else
    alias ae='aether --mode pipe'
fi

# Sentinel mode trigger (error analysis)
# Note: Cannot use ?? as a function name in bash, so we use an alias
__aether_sentinel_trigger() {
    local aether_cmd="${AETHER_BIN:-$(command -v aether || echo aether)}"
    "$aether_cmd" --mode sentinel
}
alias '??'='__aether_sentinel_trigger'

echo "✨ AETHER initialized - Press Ctrl+Space to activate"
