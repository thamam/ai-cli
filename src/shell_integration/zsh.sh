#!/usr/bin/env zsh
# AETHER Shell Integration for Zsh
# This script should be sourced in your .zshrc

# Directory for aether temporary files
export AETHER_TMP_DIR="/tmp/aether"
mkdir -p "$AETHER_TMP_DIR"

# Track command execution time and status
__aether_preexec() {
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

    # If command failed, save context for Sentinel mode
    if [[ $exit_code -ne 0 && -n "$__AETHER_LAST_CMD" ]]; then
        cat > "$AETHER_TMP_DIR/last_session" <<EOF
{
  "last_command": "$__AETHER_LAST_CMD",
  "exit_code": $exit_code,
  "duration": $duration,
  "working_directory": "$PWD",
  "shell_type": "zsh"
}
EOF
    fi

    # Update context file
    cat > "$AETHER_TMP_DIR/context.json" <<EOF
{
  "last_command": "$__AETHER_LAST_CMD",
  "last_exit_code": $exit_code,
  "working_directory": "$PWD",
  "shell_type": "zsh"
}
EOF
}

# Add hooks
autoload -Uz add-zsh-hook
add-zsh-hook preexec __aether_preexec
add-zsh-hook precmd __aether_precmd

# Keybinding for Lens mode (Ctrl+Space)
__aether_lens_mode() {
    local result
    result=$(aether --mode lens --buffer "$BUFFER" --cursor-pos "$CURSOR" 2>/dev/null)

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
alias ae='aether --mode pipe'

# Function for Sentinel mode (error analysis)
??() {
    aether --mode sentinel
}

echo "✨ AETHER initialized - Press Ctrl+Space to activate"
