#!/usr/bin/env bash
# Combined build + install script for Linux.
# Usage: bash linux.sh

set -euo pipefail

APP="reefmt"

cargo build --release
cp target/release/${APP} ./${APP}-linux-x64
file ./${APP}-linux-x64

# Install
install_dir="$HOME/.local/bin"
target="$install_dir/$APP"

mkdir -p "$install_dir"
cp "./${APP}-linux-x64" "$target"
chmod +x "$target"

if ! echo ":$PATH:" | grep -q ":$install_dir:"; then
	if [ -f "$HOME/.bashrc" ]; then
		shell_rc="$HOME/.bashrc"
	else
		shell_rc="$HOME/.profile"
	fi

	if ! grep -Fq "$install_dir" "$shell_rc" 2>/dev/null; then
		{
			echo
			echo 'export PATH="$HOME/.local/bin:$PATH"'
		} >> "$shell_rc"
		echo "Added $install_dir to PATH in $shell_rc"
	fi
fi

echo ""
echo "Installed:"
echo "  ./${APP}-linux-x64 → $target"
echo ""
echo "Verify:"
echo "  $APP --version"
echo ""
echo "If command is not found, reload your shell:"
echo "  source ~/.bashrc"

rm -rf target
