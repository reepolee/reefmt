#!/usr/bin/env bash
# Release script for macOS and Linux.
# Builds the native binary for the current platform and publishes it as a GitHub Release.
# Version is auto-bumped (patch) only when the tag for the current version doesn't exist yet.
#
# Usage: bash release.sh [--draft]
#   --draft  Create the release as a draft (default: published)
#
# Prerequisites:
#   - gh CLI (https://cli.github.com) — authenticated via `gh auth login`
#   - git
#
# Workflow (run on each machine after pushing code):
#   1. macOS (first): bash release.sh    → bumps version, creates tag + release, uploads
#   2. Linux:          bash release.sh    → builds, uploads to existing release
#   3. Windows:        .\release.ps1     → builds, uploads to existing release

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
# Read current version from Cargo.toml
# ──────────────────────────────────────────────

bump_version() {
	local current="$1"
	local major="${current%%.*}"
	local rest="${current#*.}"
	local minor="${rest%%.*}"
	local patch="${rest#*.}"
	local new_patch=$((patch + 1))
	echo "$major.$minor.$new_patch"
}

version=$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)
if [ -z "$version" ]; then
	echo "ERROR: Could not find version in Cargo.toml" >&2
	exit 1
fi

os="$(uname -s)"
arch="$(uname -m)"

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
			x86_64|amd64)      binary_name="${APP}-linux-x64" ;;
			arm64|aarch64)     binary_name="${APP}-linux-arm64" ;;
			*)                 echo "Unsupported arch: $arch" >&2; exit 1 ;;
		esac
		;;
	*)
		echo "Unsupported OS: $os" >&2
		exit 1
		;;
esac

# ──────────────────────────────────────────────
# Detect code changes since last release
# ──────────────────────────────────────────────

git fetch --tags 2>/dev/null || true
latest_tag=$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || echo "")

if [ -n "$latest_tag" ]; then
	# Verify local version matches the latest tag before proceeding.
	# If you run the release script without pulling first, versions will diverge.
	tag_version="${latest_tag#v}"
	if [ "$tag_version" != "$version" ]; then
		echo "ERROR: Local version ($version) differs from latest tag ($tag_version)." >&2
		echo "  Run 'git pull' first to sync, then try again." >&2
		exit 1
	fi

	new_commits=$(git rev-list HEAD "^$latest_tag" --count 2>/dev/null || echo "0")
else
	# No prior tag → this is the first release ever
	new_commits=1
fi

tag="v$version"

if [ "$new_commits" -gt 0 ]; then
	# Code has changed since last release → bump version
	new_version=$(bump_version "$version")
	echo "═══ reefmt release $new_version for $os ($arch) ═══"
	echo "  (Bumping from $version → $new_version, $new_commits commits since $latest_tag)"

	# Update Cargo.toml
	sed -i '' "s/version = \"$version\"/version = \"$new_version\"/" Cargo.toml 2>/dev/null || \
	sed -i "s/version = \"$version\"/version = \"$new_version\"/" Cargo.toml

	version="$new_version"
	tag="v$version"
	do_bump=true
else
	# No code changes → just upload the binary
	echo "═══ reefmt release $version for $os ($arch) ═══"
	echo "  (No new commits since $latest_tag. Uploading binary only.)"
	do_bump=false
fi

# ──────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────

echo ""
echo "→ Building $binary_name..."
cargo build --release
cp "./target/release/$APP" "./$binary_name"
file "./$binary_name"

# ──────────────────────────────────────────────
# Commit version bump (first machine only)
# ──────────────────────────────────────────────

if [ "$do_bump" = true ]; then
	echo ""
	echo "→ Committing version bump..."
	git add Cargo.toml
	git commit -m "Bump version to $version"
	echo "  Committed: Bump version to $version"
fi

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

# Push tag and (if bumped) the version bump commit together
echo "  Pushing tag $tag to origin..."
git push origin "$tag"

if [ "$do_bump" = true ]; then
	echo "  Pushing version bump commit..."
	git push origin HEAD
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
# Install locally (to PATH)
# ──────────────────────────────────────────────

echo ""
echo "→ Installing locally..."
install_dir="$HOME/.local/bin"
mkdir -p "$install_dir"
cp "./target/release/$APP" "$install_dir/$APP"
chmod +x "$install_dir/$APP"

if ! echo ":$PATH:" | grep -q ":$install_dir:"; then
	shell_rc=""
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
		echo "  Added $install_dir to PATH in $shell_rc"
	fi
	echo "  Restart shell or run: export PATH=\"$install_dir:\$PATH\""
fi

echo "  Installed to $install_dir/$APP"

# ──────────────────────────────────────────────
# Done
# ──────────────────────────────────────────────

echo ""
echo "✅ Done! Released $binary_name → $tag"
echo "   View at: https://github.com/$(git remote get-url origin | sed -E 's|.*github.com[/:]||; s|\.git$||')/releases/tag/$tag"
