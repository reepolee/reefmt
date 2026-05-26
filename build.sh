set -e

build_native() {
	cargo build --release
	# binary at ./target/release/reefmt
	cp ./target/release/reefmt ./reefmt
	echo "Built native (aarch64-apple-darwin): $(file ./reefmt | sed 's/.*: //')"
	rm -rf ./target
}

build_intel() {
	cargo build --release --target x86_64-apple-darwin
	# binary at ./target/x86_64-apple-darwin/release/reefmt
	cp ./target/x86_64-apple-darwin/release/reefmt ./reefmt-intel
	echo "Built x86_64-apple-darwin: $(file ./reefmt-intel | sed 's/.*: //')"
	rm -rf ./target
}

build_windows() {
	cargo build --release --target x86_64-pc-windows-gnu
	# binary at ./target/x86_64-pc-windows-gnu/release/reefmt.exe
	cp ./target/x86_64-pc-windows-gnu/release/reefmt.exe ./reefmt-windows-x64.exe
	echo "Built x86_64-pc-windows-gnu: $(file ./reefmt-windows-x64.exe | sed 's/.*: //')"
	rm -rf ./target
}

build_universal() {
	build_native
	build_intel
	lipo -create -output ./reefmt-universal ./reefmt ./reefmt-intel
	cp ./reefmt-universal ./reefmt
	rm ./reefmt-intel ./reefmt-universal
	echo "Universal binary: $(lipo -info ./reefmt | sed 's/.*: //')"
}

case "${1:-native}" in
	native)
		build_native
		;;
	intel)
		build_intel
		;;
	windows)
		build_windows
		;;
	all)
		build_universal
		build_windows
		echo "---"
		echo "All builds complete:"
		ls -lh ./reefmt ./reefmt-windows-x64.exe
		;;
	*)
		echo "Usage: $0 {native|intel|windows|all}"
		echo "  native  - build for aarch64-apple-darwin (Apple Silicon, default)"
		echo "  intel   - build for x86_64-apple-darwin (Intel Mac)"
		echo "  windows - cross-compile for x86_64-pc-windows-gnu"
		echo "  all     - build all three, create universal macOS binary"
		exit 1
		;;
esac
