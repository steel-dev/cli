#!/bin/sh
set -eu

echo "Steel CLI — browser automation for AI agents"
echo "https://github.com/steel-dev/cli"
echo ""

if ! command -v curl > /dev/null; then
    echo "error: curl is required but not installed."
    exit 1
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
