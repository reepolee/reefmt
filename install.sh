#!/usr/bin/env bash

set -euo pipefail

BIN_NAME="reefmt"
INSTALL_DIR="$HOME/.local/bin"
TARGET="$INSTALL_DIR/$BIN_NAME"

mkdir -p "$INSTALL_DIR"

cp "./$BIN_NAME" "$TARGET"
chmod +x "$TARGET"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "$INSTALL_DIR already in PATH"
    ;;
  *)
    SHELL_RC=""

    if [ -n "${ZSH_VERSION:-}" ]; then
      SHELL_RC="$HOME/.zshrc"
    elif [ -n "${BASH_VERSION:-}" ]; then
      SHELL_RC="$HOME/.bashrc"
    else
      SHELL_RC="$HOME/.profile"
    fi

    echo "" >> "$SHELL_RC"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"

    echo "Added $INSTALL_DIR to PATH in $SHELL_RC"
    ;;
esac

echo "Installed to $TARGET"
echo ""
echo "Restart shell or run:"
echo "export PATH=\"$INSTALL_DIR:\$PATH\""
