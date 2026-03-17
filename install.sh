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

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh | sh

echo ""
echo "Setup complete! Try it out:"
echo ""
echo "  steel --help"
echo "  steel login"
echo "  steel browser start"
echo ""
