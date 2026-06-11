#!/usr/bin/env bash

set -euo pipefail

APP_NAME="reefmt"
INSTALL_DIR="$HOME/.local/bin"
TARGET="$INSTALL_DIR/$APP_NAME"
BIN_NAME="${APP_NAME}-linux-x64"

if [ ! -f "./$BIN_NAME" ]; then
    echo "Binary not found: ./$BIN_NAME"
    exit 1
fi

mkdir -p "$INSTALL_DIR"

cp "./$BIN_NAME" "$TARGET"
chmod +x "$TARGET"

if ! echo ":$PATH:" | grep -q ":$INSTALL_DIR:"; then
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    else
        SHELL_RC="$HOME/.profile"
    fi

    if ! grep -Fq "$INSTALL_DIR" "$SHELL_RC" 2>/dev/null; then
        {
            echo
            echo 'export PATH="$HOME/.local/bin:$PATH"'
        } >> "$SHELL_RC"

        echo "Added $INSTALL_DIR to PATH in $SHELL_RC"
    fi
fi

echo
echo "Installed:"
echo "  ./$BIN_NAME → $TARGET"
echo
echo "Verify:"
echo "  reefmt --version"
echo
echo "If command is not found, reload your shell:"
echo "  source ~/.bashrc"
