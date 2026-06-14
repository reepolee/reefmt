set -e

APP="reefmt"

# Auto-bump patch version in Cargo.toml
current_version=$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)
if [ -z "$current_version" ]; then
    echo "ERROR: Could not find version in Cargo.toml"
    exit 1
fi
major=$(echo "$current_version" | cut -d. -f1)
minor=$(echo "$current_version" | cut -d. -f2)
patch=$(echo "$current_version" | cut -d. -f3)
new_patch=$((patch + 1))
new_version="$major.$minor.$new_patch"
sed -i '' "s/version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
echo "Bumped version to $new_version"

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

	cp ./target/x86_64-pc-windows-gnu/release/${APP}.exe \
		./${APP}-windows-x64.exe

	echo "Built Windows x64:"
	file ./${APP}-windows-x64.exe | sed 's/.*: //'
}

build_linux() {
	cargo build --release --target x86_64-unknown-linux-gnu

	cp ./target/x86_64-unknown-linux-gnu/release/$APP \
		./${APP}-linux-x64

	echo "Built Linux x64:"
	file ./${APP}-linux-x64 | sed 's/.*: //'
}

build_universal() {
	build_native
	build_intel

	lipo -create \
		-output ./${APP}-macos-universal \
		./${APP}-macos-arm64 \
		./${APP}-macos-x64

	echo "Built macOS universal:"
	lipo -info ./${APP}-macos-universal | sed 's/.*: //'
}

show_outputs() {
	echo "---"
	echo "Build outputs:"
	ls -lh ./${APP}-*
}

case "${1:-native}" in
	native)
		build_native
		;;

	intel)
		build_intel
		;;

	universal)
		build_universal
		;;

	windows)
		build_windows
		;;

	linux)
		build_linux
		;;

	all)
		build_universal
		build_windows
		build_linux
		show_outputs
		;;

	*)
		echo "Usage: $0 {native|intel|universal|windows|linux|all}"
		exit 1
		;;
esac

rm -rf ./target
