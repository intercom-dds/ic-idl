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
