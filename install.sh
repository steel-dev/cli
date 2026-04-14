#!/bin/sh
: <<'DOCSTRING'
Steel CLI — browser automation for AI agents
https://github.com/steel-dev/cli

Installs the Steel CLI binary and, in an interactive terminal, runs
`steel init` to log in, verify connectivity, and install coding-agent skills.

Flags:
  --non-interactive    Skip the interactive `steel init` step (install only)
  --agent              Run `steel init --agent` (print the onboarding guide
                       to stdout — intended for AI coding agents)

REQUIRES: curl
INSTALLS TO: ~/.steel/bin

After install, PATH is NOT updated in the current shell. Restart your shell
or run: export PATH="$HOME/.steel/bin:$PATH"
DOCSTRING
set -eu

STEEL_NON_INTERACTIVE="no"
STEEL_AGENT_MODE="no"

for arg in "$@"; do
    case "$arg" in
        --non-interactive) STEEL_NON_INTERACTIVE="yes" ;;
        --agent) STEEL_AGENT_MODE="yes" ;;
        *) ;;
    esac
done

# Detect non-interactive / CI environments and skip interactive setup.
# Mirrors the Atuin installer pattern: probe for a controlling TTY, fall
# back to non-interactive if none is available.
if [ "$STEEL_NON_INTERACTIVE" != "yes" ]; then
    if [ -t 0 ] || { true </dev/tty; } 2>/dev/null; then
        STEEL_NON_INTERACTIVE="no"
    else
        STEEL_NON_INTERACTIVE="yes"
    fi
fi

echo "Steel CLI — browser automation for AI agents"
echo "https://github.com/steel-dev/cli"
echo ""

if ! command -v curl > /dev/null; then
    echo "error: curl is required but not installed."
    exit 1
fi

if command -v steel > /dev/null 2>&1; then
    existing=$(command -v steel)
    if [ -L "$existing" ]; then
        real=$(readlink -f "$existing" 2>/dev/null || readlink "$existing")
    else
        real="$existing"
    fi
    is_node=false
    case "$real" in
        */node_modules/*) is_node=true ;;
    esac
    if [ "$is_node" = false ] && head -1 "$real" 2>/dev/null | grep -q "node\|nodejs"; then
        is_node=true
    fi
    if [ "$is_node" = true ]; then
        echo "Detected old Node.js Steel CLI at: $existing"
        echo "Removing it automatically..."
        echo ""
        if npm uninstall -g @steel-dev/cli 2>/dev/null; then
            echo "Old Node.js CLI removed successfully."
        else
            echo "Warning: could not auto-remove. Please run manually:"
            echo "  npm uninstall -g @steel-dev/cli"
            echo ""
            exit 1
        fi
        echo ""
    fi
fi

echo "Installing steel binary..."

STEEL_CLI_NO_MODIFY_PATH=1 curl --proto '=https' --tlsv1.2 -LsSf https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh | STEEL_CLI_NO_MODIFY_PATH=1 sh > /dev/null 2>&1

INSTALL_DIR="$HOME/.steel/bin"

_added=false
if [ -n "${SHELL:-}" ]; then
    case "$SHELL" in
        */zsh)
            _rc="$HOME/.zshrc"
            ;;
        */bash)
            if [ "$(uname)" = "Darwin" ]; then
                _rc="$HOME/.bash_profile"
            else
                _rc="$HOME/.bashrc"
            fi
            ;;
        */fish)
            _rc="$HOME/.config/fish/conf.d/steel.fish"
            ;;
        *)
            _rc="$HOME/.profile"
            ;;
    esac

    _line="export PATH=\"$INSTALL_DIR:\$PATH\""
    if [ "${SHELL##*/}" = "fish" ]; then
        _line="fish_add_path $INSTALL_DIR"
    fi

    if [ -f "$_rc" ] && grep -qF "$INSTALL_DIR" "$_rc" 2>/dev/null; then
        _added=true
    elif [ -n "$_rc" ]; then
        mkdir -p "$(dirname "$_rc")"
        echo "$_line" >> "$_rc"
        _added=true
    fi
fi

STEEL_BIN="$INSTALL_DIR/steel"

echo ""
echo "Installed steel to $STEEL_BIN"
if [ "$_added" = true ]; then
    echo "Added $INSTALL_DIR to PATH in $_rc"
fi
echo ""

# Agent mode just prints the onboarding guide to stdout, which needs no TTY,
# so it runs regardless of whether the surrounding shell is interactive.
# Human mode drives `dialoguer` prompts, so we only run it when /dev/tty
# exists.
if [ "$STEEL_AGENT_MODE" = "yes" ]; then
    "$STEEL_BIN" init --agent
elif [ "$STEEL_NON_INTERACTIVE" != "yes" ]; then
    "$STEEL_BIN" init </dev/tty
fi

if [ "$STEEL_AGENT_MODE" = "yes" ]; then
    cat << 'EOF'

Add the Steel binary to PATH for this session:
  export PATH="$HOME/.steel/bin:$PATH"

EOF
else
    cat << 'EOF'

===============================================================================

  Restart your shell or open a new terminal so `steel` is on PATH.
  Or run: export PATH="$HOME/.steel/bin:$PATH"

===============================================================================
EOF
fi
