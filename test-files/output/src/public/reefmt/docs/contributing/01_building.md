---
title: "Building from source"
---

# Building from source

Reefmt is a Rust project (edition 2021). You only need this if you want to hack
on Reefmt itself or produce a binary for a platform that isn't in the prebuilt
releases — most users should just follow
[Installation](/reefmt/docs/getting-started/installation).

## Prerequisites

Install Rust via [rustup](https://rustup.rs/).

## Build with cargo

Build directly with cargo for a release binary:

```bash
cargo build --release
# binary at ./target/release/reefmt
```

Copy the result somewhere on your `PATH` to test it locally:

```bash
cp target/release/reefmt ~/.local/bin/          # macOS / Linux
Copy-Item .\target\release\reefmt.exe ~\bin\    # Windows
```

## Release

The release scripts build the native binary for the current platform, bump the
version, tag it, and publish a GitHub Release with `gh`. Windows binaries are
built natively on Windows — cross-compilation is not supported, so the workflow
runs once per platform against the same release:

```bash
bash release.sh     # macOS / Linux
.\release.ps1       # Windows
```

Add `--draft` (`-Draft` on Windows) to create the release as a draft, or
`--minor` to bump the minor version instead of the patch version.

## Contributing

Issues and pull requests are welcome on
[GitHub](https://github.com/reepolee/reefmt). Reefmt is part of the Reepolee
toolchain — see [reepolee.com](https://www.reepolee.com) for the wider project.
