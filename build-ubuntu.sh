#!/usr/bin/env bash
set -e

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
sed -i "s/version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
echo "Bumped version to $new_version"

cargo build --release
cp target/release/reefmt reefmt-linux-x64
file reefmt-linux-x64
rm -rf target
