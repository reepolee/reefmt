---
title: "Building from source"
---

# Building from source

SQLfmt is a Rust project (edition 2021). You only need this to work on SQLfmt
itself or to build for a platform that isn't in the prebuilt releases — most
users should follow [Installation](/sqlfmt/docs/getting-started/installation).

## Prerequisites

Install Rust via [rustup](https://rustup.rs/).

## Build targets (macOS / Linux)

The build script wraps `cargo` and supports several targets:

```bash
./build.sh native      # current macOS architecture (arm64)
./build.sh intel       # Intel macOS (requires the x86_64-apple-darwin target)
./build.sh universal   # macOS universal binary (arm64 + x64)
./build.sh linux       # Linux x64 (requires the cross toolchain)
./build.sh all         # every supported target
```

### Linux cross-compilation on macOS

```bash
rustup target add x86_64-apple-darwin x86_64-unknown-linux-gnu

brew tap messense/macos-cross-toolchains
brew install x86_64-unknown-linux-gnu
```

The build script checks for these dependencies and prints helpful errors if
anything is missing.

## Windows

Windows builds are produced natively on Windows; cross-compilation from macOS is
not supported:

```powershell
.\build.ps1
```

## Plain cargo

```bash
cargo build --release
# binary at ./target/release/sqlfmt
```

## Testing

SQLfmt ships integration tests that format sample inputs and compare them
against golden files:

```bash
cargo test
```

Test inputs and expected outputs live in `tests/data/` — each case is an
`.input.sql` file paired with a `.golden.sql` file. To add a case, create a new
pair and add a test function in `tests/integration_test.rs`.

## Contributing

Issues and pull requests are welcome on
[GitHub](https://github.com/reepolee/sqlfmt). SQLfmt is part of the Reepolee
toolchain — see [reepolee.com](https://www.reepolee.com) for the wider project.
