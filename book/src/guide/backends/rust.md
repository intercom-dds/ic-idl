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

# Rust Backend

The Rust backend emits idiomatic, `#![forbid(unsafe)]`-ready code backed by the
`intercom-cts` runtime. Each input module becomes a Rust module; the compiler
also creates a top-level `lib.rs` that re-exports everything for ergonomic use.

## Generating code

```bash
ic-idl schema.idl --rust-out src/generated
```

`src/generated/lib.rs` publicly re-exports the generated modules. Include the
folder in your project with `mod generated;` (binary crate) or `pub mod
 generated;` (library crate).

## What is generated?

- **Structs, unions, valuetypes, exceptions** – emitted as `pub struct` with
  `pub` fields.
- **Enums** – emitted with `#[repr(Int)]`, `Clone + Copy + Eq + Ord + Hash`
  derives, plus `FromStr`/`Display` helpers.
- **Unions** – emitted as Rust enums with one variant per case.
- **Interfaces** – emitted as traits with methods and typed parameters.
- **Constants, aliases, bitmasks** – mapped to idiomatic Rust `const`, `type`,
  and bitflag wrappers.
- **Metadata & serialization** – every type carries `TypeInfo`, implements
  `Marshal`/`Unmarshal`, and derives `Default` via a `new()` constructor.

Example struct output:

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Person {
    pub name: ::std::string::String,
    pub age: i32,
}

impl Person {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: ::std::string::String::new(),
            age: 0,
        }
    }
}

impl ::std::default::Default for Person {
    fn default() -> Self {
        Self::new()
    }
}
```

Additional traits such as `Eq`, `Ord`, and `Hash` are derived when the member
types support them.

The constructors default to zero/empty values or the first enumerator; unions
default to their first alternative.

## Naming conventions

Unless `--no-rename` is used, the backend converts identifiers to idiomatic Rust
cases:

- Types, modules, interfaces → `PascalCase`
- Fields, parameters, operations → `snake_case`
- Constants → `UPPER_SNAKE_CASE`

The renamer also escapes Rust keywords by appending `_`.

## Command-line options

- `--no-rename` – keep the original identifier spelling.
- `--must-use` – add `#[must_use]` to every generated nominal type (structs,
  enums, unions, exceptions, interfaces). When this flag is active the
  `new()` constructors drop their own attribute because the warning is enforced
  on the type instead.

## Runtime dependency

Add the serialization crate to your `Cargo.toml`:

```toml
[dependencies]
intercom-cts = { path = "library/rust/intercom-cts" }
```

The workspace uses a path dependency; when packaging your own project you can
vendor the runtime (see the `library/rust/` workspace) or publish it to an
internal registry.

## Further reading

- [Type mappings](../../rust/type-mappings.md)
- [Generated code details](../../rust/generated-code.md)
- [Serialization](../../rust/serialization.md)
- [Build integration](../../rust/build-integration.md)
