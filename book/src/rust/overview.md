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

# Rust Backend Overview

The Rust backend turns IDL definitions into strongly typed modules that integrate
with the `intercom-cts` serialization runtime.

## Quick example

```bash
ic-idl schema.idl --rust-out src/generated
```

Generated layout:

```
src/generated/
├── lib.rs           # Re-exports generated modules
└── demo/schema.rs   # One file per input module/source
```

You can include the output in your crate with:

```rust
mod generated;
use generated::demo::Person;
```

## Key properties

- **Public data structures** – structs, unions, valuetypes, and exceptions are
  emitted with `pub` fields so you can use struct initialisation syntax.
- **Helpful derives** – the backend derives `Clone`, `Debug`, `Eq`, `PartialEq`,
  `Ord`, `PartialOrd`, and `Hash` when the target type supports it.
- **Deterministic constructors** – each nominal type exposes `new()` plus
  `Default` that fill in sensible defaults (zero/nil, empty collections, first
  enumerator).
- **Serialization ready** – every type implements `Marshal`/`Unmarshal`
  (intercom CTS) and ships with metadata describing bounds, unions, keys, etc.
- **Interfaces as traits** – IDL interfaces become Rust traits with methods,
  parameter structs, and return types mirroring the IDL signature.
- **Optional `#[must_use]`** – constructors are annotated with `#[must_use]` by
  default; passing `--must-use` instead places the attribute on the type itself.

## When to regenerate

Because the code is deterministic, a schema change will only affect the
relevant module(s). Keep generated files under version control so schema changes
are reviewable, and rerun the compiler as part of your build or release flow.

## Runtime dependency

Generated code references the `intercom-cts` crate found in
`library/rust/intercom-cts`. Add it to the crate that consumes the generated
module or depend on it from a workspace crate that re-exports the generated
code.

## Related documentation

- [Type mappings](./type-mappings.md)
- [Generated code deep dive](./generated-code.md)
- [Serialization](./serialization.md)
- [Build integration](./build-integration.md)
