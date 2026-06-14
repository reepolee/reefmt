#!/usr/bin/env bash
# Release script for macOS and Linux.
# Builds the native binary for the current platform and publishes it as a GitHub Release.
#
# Usage: bash release.sh [--draft]
#   --draft  Create the release as a draft (default: published)
#
# Prerequisites:
#   - gh CLI (https://cli.github.com) — authenticated via `gh auth login`
#   - git
#
# Workflow (run on each machine after pushing code):
#   1. macOS:  bash release.sh            → builds, creates tag + release, uploads
#   2. Linux:  bash release.sh            → builds, uploads to existing release
#   3. Windows: .\release.ps1            → builds, uploads to existing release

set -euo pipefail

APP="reefmt"

# ──────────────────────────────────────────────
# Validate prerequisites
# ──────────────────────────────────────────────

if ! command -v gh &>/dev/null; then
	echo "ERROR: gh CLI not found. Install it from https://cli.github.com/" >&2
	exit 1
fi

if ! gh auth status &>/dev/null; then
	echo "ERROR: gh CLI is not authenticated. Run: gh auth login" >&2
	exit 1
fi

# ──────────────────────────────────────────────
# Read version from Cargo.toml
# ──────────────────────────────────────────────

version=$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)
if [ -z "$version" ]; then
	echo "ERROR: Could not find version in Cargo.toml" >&2
	exit 1
fi

tag="v$version"

os="$(uname -s)"
arch="$(uname -m)"

echo "═══ reefmt release $version for $os ($arch) ═══"

# ──────────────────────────────────────────────
# Determine binary name for this platform
# ──────────────────────────────────────────────

case "$os" in
	Darwin)
		case "$arch" in
			arm64|aarch64) binary_name="${APP}-macos-arm64" ;;
			x86_64)       binary_name="${APP}-macos-x64" ;;
			*)            echo "Unsupported arch: $arch" >&2; exit 1 ;;
		esac
		;;
	Linux)
		case "$arch" in
			x86_64|amd64) binary_name="${APP}-linux-x64" ;;
			*)           echo "Unsupported arch: $arch" >&2; exit 1 ;;
		esac
		;;
	*)
		echo "Unsupported OS: $os" >&2
		exit 1
		;;
esac

# ──────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────

echo ""
echo "→ Building $binary_name..."
cargo build --release
cp "./target/release/$APP" "./$binary_name"
file "./$binary_name"

# ──────────────────────────────────────────────
# Create and push git tag
# ──────────────────────────────────────────────

echo ""
echo "→ Tagging $tag..."

if git rev-parse "$tag" >/dev/null 2>&1; then
	echo "  Tag $tag already exists locally."
else
	git tag "$tag"
	echo "  Created tag $tag locally."
fi

# Push the tag if it hasn't been pushed yet
if ! git ls-remote --tags origin "$tag" | grep -q "$tag"; then
	git push origin "$tag"
	echo "  Pushed tag $tag to origin."
else
	echo "  Tag $tag already exists on origin."
fi

# ──────────────────────────────────────────────
# Create or upload to GitHub Release
# ──────────────────────────────────────────────

echo ""
echo "→ Publishing release $tag..."

draft_flag=""
if [ "${1:-}" = "--draft" ]; then
	draft_flag="--draft"
	echo "  (Draft mode)"
fi

asset_path="./$binary_name"
asset_name="$binary_name"

if gh release view "$tag" >/dev/null 2>&1; then
	echo "  Release $tag already exists. Uploading asset..."
	gh release upload "$tag" "$asset_path#$asset_name" --clobber
else
	echo "  Creating release $tag..."
	gh release create "$tag" \
		"$asset_path#$asset_name" \
		--title "$tag" \
		--notes "Release $tag" \
		$draft_flag
fi

# ──────────────────────────────────────────────
# Done
# ──────────────────────────────────────────────

echo ""
echo "✅ Done! Released $binary_name → $tag"
echo "   View at: https://github.com/$(git remote get-url origin | sed -E 's|.*github.com[/:]||; s|\.git$||')/releases/tag/$tag"
