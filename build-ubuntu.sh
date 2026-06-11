#!/usr/bin/env bash
set -e

cargo build --release
cp target/release/reefmt reefmt-linux-x64
file reefmt-linux-x64
rm -rf target
