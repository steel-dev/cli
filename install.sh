#!/bin/sh
set -eu

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

echo ""
echo "Setup complete!"
if [ "$_added" = true ]; then
    echo "Added $INSTALL_DIR to PATH in $_rc"
    echo ""
    echo "Restart your shell or run:"
    echo "  source $_rc"
else
    echo "Add this to your shell config:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi
echo ""
echo "Then try it out:"
echo ""
echo "  steel --help"
echo "  steel login"
echo "  steel browser start"
echo ""
