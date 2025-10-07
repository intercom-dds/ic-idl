# Installation

These steps show how to fetch and build `ic-idl` from source. We do not ship
pre-built binaries yet, so building locally is the supported workflow.

## Prerequisites

- **Rust toolchain** – Rust 1.87 or newer (matches the workspace `rust-version`)
  with `cargo` and `rustup`. Install from [rustup.rs](https://rustup.rs/).
- **A C++17 compiler** – required for compiling the bundled CTS runtime and for
  the C++ end-to-end tests (e.g. `clang++`, `g++`, or MSVC).
- **Git** – to clone the repository.

Optional tools that unlock additional functionality:

- **Python 3.10+** – needed when exercising the Python backend or running the
  Python e2e suite.
- **Protocol Buffers compiler (`protoc`)** – required if you intend to compile
  the generated `.proto` files as part of testing.
- **`cargo-nextest`** – used by the recommended test command (`cargo install
  --locked cargo-nextest`).

## Build from source

```bash
# Clone the repository
git clone https://github.com/kongsberg/ic-idl.git
cd ic-idl

# Compile a release build of the CLI
cargo build -p ic-idl --release
```

The resulting binary lives at `target/release/ic-idl`. You can execute it
in-place, or install it into your Cargo binary directory:

```bash
cargo install --path crates/ic-idl --locked
```

Because the crate is marked `publish = false`, `cargo install` must be invoked
with the local path as shown above.

## Creating release archives

The repository contains an `xtask` helper that assembles a full installation
layout (CLI + runtime libraries + license file):

```bash
cargo xtask release
```

This produces `install/ic-idl_<version>.tar.gz` alongside an expanded
`install/ic-idl/` tree. The command reuses your existing build artefacts; remove
`target/` if you need a clean rebuild.

## Running without installing

You can invoke the compiler directly via Cargo while iterating:

```bash
cargo run -p ic-idl -- --help
cargo run -p ic-idl -- --rust-out out/ schema.idl
```

Cargo will rebuild the binary when sources change and forward the remaining
arguments to `ic-idl`.

## Verifying the build

```bash
./target/release/ic-idl --version
```

Typical output:

```
ic-idl 0.1.0 (abc1234 2025-03-04)
target: x86_64-unknown-linux-gnu
build type: release
```

If you installed with `cargo install`, ensure `$HOME/.cargo/bin` is present in
`$PATH` so the shell can locate the executable.

## Troubleshooting

- **"command not found"** – add `$HOME/.cargo/bin` to your `PATH` or run the
  binary via its full path (`target/release/ic-idl`).
- **Old Rust toolchain** – run `rustup update stable` and re-run the build if
  `cargo` reports that the required feature (edition 2024, lint settings, …)
  is unavailable.
- **C++ compilation errors during tests** – make sure you have a modern C++17
  compiler and the standard library headers installed (Debian/Ubuntu:
  `sudo apt install build-essential`).
- **Python backend tests skipped** – install Python 3.10 or later and rerun the
  e2e suite with `cargo xtask e2e`.

For platform-specific quirks, consult the continuous integration configuration
under `ci/` and the Dockerfiles in that directory—they document the minimal
set of packages used in automation.
