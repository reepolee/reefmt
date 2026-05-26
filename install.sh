#!/usr/bin/env bash

set -euo pipefail

APP_NAME="reefmt"
INSTALL_DIR="$HOME/.local/bin"
TARGET="$INSTALL_DIR/$APP_NAME"

detect_bin_name() {
	local os
	local arch

	os="$(uname -s)"
	arch="$(uname -m)"

	case "$os" in
		Darwin)
			case "$arch" in
				arm64|aarch64)
					echo "${APP_NAME}-macos-arm64"
					;;
				x86_64)
					echo "${APP_NAME}-macos-x64"
					;;
				*)
					echo "Unsupported macOS architecture: $arch"
					exit 1
					;;
			esac
			;;

		Linux)
			case "$arch" in
				x86_64|amd64)
					echo "${APP_NAME}-linux-x64"
					;;
				aarch64|arm64)
					echo "${APP_NAME}-linux-arm64"
					;;
				*)
					echo "Unsupported Linux architecture: $arch"
					exit 1
					;;
			esac
			;;

		*)
			echo "Unsupported operating system: $os"
			exit 1
			;;
	esac
}

BIN_NAME="$(detect_bin_name)"

if [ ! -f "./$BIN_NAME" ]; then
	echo "Binary not found: ./$BIN_NAME"
	exit 1
fi

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

		if ! grep -Fq "$INSTALL_DIR" "$SHELL_RC" 2>/dev/null; then
			echo "" >> "$SHELL_RC"
			echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"

			echo "Added $INSTALL_DIR to PATH in $SHELL_RC"
		fi
		;;
esac

echo "Installed:"
echo "  ./$BIN_NAME → $TARGET"

echo ""
echo "Restart shell or run:"
echo "export PATH=\"$INSTALL_DIR:\$PATH\""
