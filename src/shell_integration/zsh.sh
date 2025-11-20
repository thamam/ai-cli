#!/usr/bin/env zsh
# AETHER Shell Integration for Zsh
# This script should be sourced in your .zshrc

# Directory for aether temporary files
export AETHER_TMP_DIR="/tmp/aether"
mkdir -p "$AETHER_TMP_DIR"

# Track command execution time and status
__aether_preexec() {
    # Ignore hook commands themselves
    case "$1" in
        __aether_precmd*|__aether_preexec*) return ;;
    esac

    __AETHER_CMD_START_TIME=$EPOCHSECONDS
    __AETHER_LAST_CMD="$1"
}

__aether_precmd() {
    local exit_code=$?
    local duration=0

    if [[ -n "$__AETHER_CMD_START_TIME" ]]; then
        duration=$((EPOCHSECONDS - __AETHER_CMD_START_TIME))
        unset __AETHER_CMD_START_TIME
    fi

    # Always update session context (for Lens mode to use)
    cat > "$AETHER_TMP_DIR/session_context.json" <<EOF
{
  "last_command": "$__AETHER_LAST_CMD",
  "last_exit_code": $exit_code,
  "duration": $duration,
  "working_directory": "$PWD",
  "shell_type": "zsh",
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
  "shell_type": "zsh",
  "timestamp": $(date +%s)
}
EOF
    fi
}

# Add hooks
autoload -Uz add-zsh-hook
add-zsh-hook preexec __aether_preexec
add-zsh-hook precmd __aether_precmd

# Keybinding for Lens mode (Ctrl+Space)
__aether_lens_mode() {
    local result
    # Use absolute path if AETHER_BIN is set, otherwise try to find aether
    local aether_cmd="${AETHER_BIN:-$(command -v aether || echo aether)}"
    result=$("$aether_cmd" --mode lens --buffer "$BUFFER" --cursor-pos "$CURSOR" 2>/dev/null)

    if [[ -n "$result" ]]; then
        BUFFER="$result"
        CURSOR=${#BUFFER}
    fi

    zle reset-prompt
}

# Create the widget
zle -N __aether_lens_mode

# Bind to Ctrl+Space
bindkey '^ ' __aether_lens_mode

# Alias for pipe mode
# Use absolute path if AETHER_BIN is set, otherwise use aether from PATH
if [[ -n "$AETHER_BIN" ]]; then
    alias ae="$AETHER_BIN --mode pipe"
else
    alias ae='aether --mode pipe'
fi

# Sentinel mode trigger (error analysis)
# Note: ?? is easier in zsh but we use same pattern as bash for consistency
__aether_sentinel_trigger() {
    local aether_cmd="${AETHER_BIN:-$(command -v aether || echo aether)}"
    "$aether_cmd" --mode sentinel
}
alias '??'='__aether_sentinel_trigger'

echo "✨ AETHER initialized - Press Ctrl+Space to activate"
