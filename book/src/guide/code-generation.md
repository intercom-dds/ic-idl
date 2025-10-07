# Code Generation Overview

`ic-idl` performs the heavy lifting of turning validated IDL files into
language-specific artefacts. The compiler reuses the same HIR for every backend,
so all targets see the same resolved types, annotations, defaults, and ordering.

This page summarises what each backend produces and how to work with the
outputs. Detailed guidance for a given language lives in the corresponding
chapter (Rust, Python, C++, …).

## Supported backends

| Backend | Output | Notes |
|---------|--------|-------|
| Rust (`--rust-out <dir>`) | `lib.rs` plus one module file per source | Public structs/unions/exceptions, enums, traits for interfaces, `new()` and `Default` implementations, and `Marshal`/`Unmarshal` via `intercom-cts`. |
| Python (`--python-out <dir>`) | Package hierarchy with `.py` modules and `__init__.py` | Classes derive from `intercom_dds.intercom_types` base classes, properties perform runtime validation, enums wrap `Enum` with `auto()`. |
| C++ (`--cpp-out <dir>`) | Header (`.h` by default) and implementation (`.cpp`) per source file | Uses STL containers, emits type/metadata helpers, optionally generates stream operators and `{fmt}` formatters. Depends on headers under `library/cpp/defs`. |
| Protobuf (`--proto-out <dir>`) | `.proto` files | Emits proto3 syntax, groups mutually dependent types into the same file, adjusts names to avoid keyword clashes. |
| JSON (`--json-out <dir>`) | JSON representation of the schema | Mostly intended for tooling and debugging. |
| JSON Schema (`--json-schema-out <dir>`) | Draft-07 schema files | One schema per type/module, encodes field constraints captured in HIR. |
| Normalised IDL (`--idl-out <dir>`) | Reformatted IDL | Can target Doxygen (`--idl-doxygen`) or a legacy-compatible subset (`--idl-legacy`). |
| XML (`--xml-out <dir>`) | XML descriptor mirroring the HIR | Useful for external tooling that consumes XML metadata. |

Backends can be invoked simultaneously; `ic-idl` writes each artefact to the
supplied directory while preserving module structure.

## Common characteristics

- **Naming conventions** – unless you opt out, the compiler renames types and
  members to idiomatic forms (PascalCase for Rust types, `snake_case` fields,
  module hierarchies in lower_snake_case, etc.). Targets expose flags (e.g.
  `--no-rename`, `--use-pep8`) for projects that prefer to keep the original
  identifier casing.
- **Visibility** – Rust and C++ outputs expose fields publicly to encourage
  struct initialisation; Python uses properties with validation to emulate the
  same behaviour safely.
- **Defaults** – structs/unions/exceptions receive a `new()` constructor that
  initialises members to language-appropriate defaults (zero for numbers,
  empty strings/collections, `None`/`Option::None`, first enum variant, …).
- **Type information** – every backend records metadata about the type (size,
  flags, key fields) so the respective runtime libraries can perform
  serialization, versioning, or reflection.

## Tips for regeneration

- Run with `--list` to see which files *would* be created. This is handy for
  integrating into build rules that need to declare outputs before generating.
- Pass `--purge-dirs` if you want the compiler to remove existing files in the
  output directory first. The helper refuses to touch directories that contain
  `.git`/`.hg`.
- When generating into a workspace, add a `build.rs` that shells out to `ic-idl`
  (see [Rust build integration](../rust/build-integration.md) for an example) or
  run the compiler from your packaging scripts.
- Keep generated code under version control. The compiler is deterministic, so
  changes in the output make review of schema revisions easier.

## Where to go next

- [Rust backend](../rust/overview.md)
- [Python backend](../python/overview.md)
- [C++ backend](../cpp/overview.md)
- [Other backends](../other-backends/protobuf.md)

Those chapters discuss type mappings, runtime dependencies, and integration
strategies in depth.
