#!/usr/bin/env bash
# Combined build + install script for macOS.
# Usage: bash macos.sh {native|intel|universal|windows|linux|all}
#   native    (default) Build and install for arm64
#   intel     Build and install for x64
#   universal Build universal (arm64 + x64), install native
#   windows   Cross-compile Windows binary (no install)
#   linux     Cross-compile Linux binary (no install)
#   all       Build all targets, install native

set -euo pipefail

APP="reefmt"

build_native() {
	cargo build --release
	cp ./target/release/$APP ./${APP}-macos-arm64
	echo "Built macOS arm64:"
	file ./${APP}-macos-arm64 | sed 's/.*: //'
}

build_intel() {
	cargo build --release --target x86_64-apple-darwin
	cp ./target/x86_64-apple-darwin/release/$APP ./${APP}-macos-x64
	echo "Built macOS x64:"
	file ./${APP}-macos-x64 | sed 's/.*: //'
}

build_windows() {
	cargo build --release --target x86_64-pc-windows-gnu
	cp ./target/x86_64-pc-windows-gnu/release/${APP}.exe ./${APP}-windows-x64.exe
	echo "Built Windows x64:"
	file ./${APP}-windows-x64.exe | sed 's/.*: //'
}

build_linux() {
	cargo build --release --target x86_64-unknown-linux-gnu
	cp ./target/x86_64-unknown-linux-gnu/release/$APP ./${APP}-linux-x64
	echo "Built Linux x64:"
	file ./${APP}-linux-x64 | sed 's/.*: //'
}

build_universal() {
	build_native
	build_intel
	lipo -create -output ./${APP}-macos-universal ./${APP}-macos-arm64 ./${APP}-macos-x64
	echo "Built macOS universal:"
	lipo -info ./${APP}-macos-universal | sed 's/.*: //'
}

show_outputs() {
	echo "---"
	echo "Build outputs:"
	ls -lh ./${APP}-*
}

install_native() {
	local arch
	arch="$(uname -m)"

	local bin_name
	case "$arch" in
		arm64|aarch64) bin_name="${APP}-macos-arm64" ;;
		x86_64)       bin_name="${APP}-macos-x64" ;;
		*)            echo "Unsupported architecture: $arch"; exit 1 ;;
	esac

	if [ ! -f "./$bin_name" ]; then
		echo "Binary not found: ./$bin_name. Build it first."
		exit 1
	fi

	local install_dir="$HOME/.local/bin"
	local target="$install_dir/$APP"

	mkdir -p "$install_dir"
	cp "./$bin_name" "$target"
	chmod +x "$target"

	if ! echo ":$PATH:" | grep -q ":$install_dir:"; then
		local shell_rc=""
		if [ -n "${ZSH_VERSION:-}" ]; then
			shell_rc="$HOME/.zshrc"
		elif [ -n "${BASH_VERSION:-}" ]; then
			shell_rc="$HOME/.bashrc"
		else
			shell_rc="$HOME/.profile"
		fi

		if ! grep -Fq "$install_dir" "$shell_rc" 2>/dev/null; then
			{
				echo
				echo "export PATH=\"$install_dir:\$PATH\""
			} >> "$shell_rc"
			echo "Added $install_dir to PATH in $shell_rc"
		fi
	fi

	echo "Installed:"
	echo "  ./$bin_name → $target"
	echo ""
	echo "Restart shell or run:"
	echo "export PATH=\"$install_dir:\$PATH\""
}

case "${1:-native}" in
	native)
		build_native
		install_native
		;;
	intel)
		build_intel
		install_native
		;;
	universal)
		build_universal
		install_native
		;;
	all)
		build_universal
		build_windows
		build_linux
		show_outputs
		install_native
		;;
	windows)
		build_windows
		;;
	linux)
		build_linux
		;;
	*)
		echo "Usage: $0 {native|intel|universal|windows|linux|all}"
		exit 1
		;;
esac

rm -rf ./target
