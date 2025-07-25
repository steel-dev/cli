#!/usr/bin/env bash

set -e

CLI_PACKAGE="@steel-dev/cli"
NVM_DIR="$HOME/.nvm"

OS="$(uname -s)"

install_node_with_nvm() {
  if [ ! -d "$NVM_DIR" ]; then
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
  else
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
  fi

  nvm install --lts
  nvm use --lts
}

install_node_with_choco() {
  if ! command -v choco &> /dev/null; then
    echo "Chocolatey not found. Please install it manually from https://chocolatey.org/install"
    exit 1
  fi
  choco install -y nodejs-lts
}

case "$OS" in
  Darwin)
    if ! command -v node &> /dev/null; then
      install_node_with_nvm
    else
      echo "Node is installed"
    fi
    ;;
  Linux)
    if ! command -v node &> /dev/null; then
      install_node_with_nvm
    else
      echo "Node is installed"
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*)
    if ! command -v node &> /dev/null; then
      install_node_with_choco
    else
      echo "Node is installed"
    fi
    ;;
  *)
    echo "❌ Unsupported OS: $OS"
    exit 1
    ;;
esac

echo "Checking node and npm"
node -v || { echo "Node installation failed"; exit 1; }
npm -v || { echo "npm installation failed"; exit 1; }

echo "⚒️🔥Installing: $CLI_PACKAGE"
npm install -g "$CLI_PACKAGE"

if command -v steel &> /dev/null; then
  steel --help
else
  echo "CLI command 'steel' not found"
  exit 1
fi
