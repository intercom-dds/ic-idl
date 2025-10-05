# Copyright 2025 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

# Integrating with Cargo builds

Most projects regenerate bindings as part of their build. This page outlines a
few common setups.

## Minimal project

```
my-project/
├── Cargo.toml
├── build.rs
├── schema.idl
└── src/
    └── main.rs
```

### `Cargo.toml`

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"

[dependencies]
intercom-cts = { path = "../ic-idl/library/rust/intercom-cts" }
```

Adjust the path to wherever you vendor the runtime. Alternatively, publish the
runtime crates to your own registry and depend on them by version.

### `build.rs`

Regenerate the bindings whenever the IDL changes.

```rust
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=schema.idl");

    let status = Command::new("ic-idl")
        .args(["--rust-out", "src/generated", "schema.idl"])
        .status()
        .expect("failed to spawn ic-idl");

    if !status.success() {
        panic!("ic-idl returned {status}");
    }
}
```

Install `ic-idl` somewhere on your `PATH` (or invoke it via `cargo run -p
ic-idl -- …`).

### `src/main.rs`

```rust
mod generated;

use generated::schema::{Directory, Person, Status};

fn main() {
    let person = Person {
        name: "Alice".into(),
        age: 30,
        status: Status::Active,
    };

    println!("{:?}", person);
}
```

## Multiple schema files

If you split your IDL across several files, iterate over them in `build.rs` and
regenerate each one:

```rust
const SCHEMAS: &[&str] = &["types.idl", "services.idl", "models.idl"];

fn main() {
    for schema in SCHEMAS {
        println!("cargo:rerun-if-changed={schema}");
        let status = std::process::Command::new("ic-idl")
            .args(["--rust-out", "src/generated", schema])
            .status()
            .expect("failed to spawn ic-idl");
        assert!(status.success(), "failed to generate {schema}");
    }
}
```

The backend merges the resulting Rust modules through the generated `lib.rs`.

## Avoiding unnecessary work

To skip regeneration when nothing changed, compare timestamps before launching
`ic-idl`:

```rust
use std::{fs, path::Path, process::Command, time::SystemTime};

fn main() {
    let schema = Path::new("schema.idl");
    let output = Path::new("src/generated/lib.rs");

    println!("cargo:rerun-if-changed={}", schema.display());

    let needs_regen = match (fs::metadata(schema), fs::metadata(output)) {
        (Ok(schema_meta), Ok(out_meta)) => schema_meta
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            > out_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        (Ok(_), Err(_)) => true,
        _ => false,
    };

    if needs_regen {
        let status = Command::new("ic-idl")
            .args(["--rust-out", "src/generated", schema.to_str().unwrap()])
            .status()
            .expect("failed to spawn ic-idl");
        if !status.success() {
            panic!("ic-idl returned {status}");
        }
    }
}
```

## Workspaces and generated crates

For larger systems you may want a dedicated crate that only contains generated
code:

```
workspace/
├── Cargo.toml
├── schema.idl
├── schema/       # generated crate
│   ├── Cargo.toml
│   ├── build.rs
│   └── src/lib.rs (empty, generated at build time)
└── app/
    ├── Cargo.toml
    └── src/main.rs
```

`schema/build.rs` looks identical to the earlier examples but emits into
`src/`. The consuming crate (`app`) depends on `schema` and uses the re-exported
API.

## Tips

- Use `--list` during development to make sure your build scripts register all
  generated files with Cargo (useful for `cargo:rerun-if-changed` bookkeeping).
- Remember to add the generated directory to `.gitignore` if you do not check it
  in.
- When cross-compiling, set `IC_IDL` or adjust the `Command` invocation so the
  build script picks up the right binary for the host platform.
