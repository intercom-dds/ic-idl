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

# Architecture Overview

`ic-idl` is organised as a collection of focused crates that form a classic
front-end → middle-end → backend compiler pipeline. This page maps each stage to
its implementation crates, highlights shared infrastructure, and shows how code
generation integrates with the bundled runtimes and tooling.

## High-level flow

```
IDL source → Lexer/Preprocessor → Parser (AST) → HIR Lowering & Validation →
Transform & Lint → Codegen Backends → Generated artefacts + metadata
```

The CLI (`crates/ic-idl`) orchestrates the steps, manages diagnostics, and
invokes whichever backends the user selected on the command line.

## Compilation pipeline

| Stage | Primary crates | Responsibilities |
|-------|----------------|------------------|
| Lexing | `ic-lexer` | Turns UTF-8 text into tokens; tracks trivia and spans. |
| Preprocessing | `ic-preproc` | Handles `#include`, `#define`, conditionals, and pragma handling; expands macros while preserving expansion info. |
| Parsing | `ic-parse`, `ic-syntax` | Builds a tolerant AST using Chumsky-based parsers; maintains comments for documentation. |
| HIR lowering | `ic-hir`, `ic-hir-tree`, `ic-expr` | Resolves names, evaluates constants, builds the canonical `ResolvedGraph`, attaches annotations, enforces semantic rules. |
| Linting | `ic-lint` | Issues warnings/errors about style, DDS compliance, and semantic edge cases; honours `-W` flags from the CLI. |
| Transformations | `ic-hir-xform` | Normalises HIR (module squashing, enum prefix stripping, type flag inference, implicit defaults) before codegen sees it. |
| Emission | `ic-emit`, `ic-idl` | Schedules backend runs, manages file writing with change detection, prints diagnostics. |

### Backends

Each backend lives in its own crate and consumes the transformed HIR. The CLI
passes backend-specific options directly to these crates.

| Backend | Crate | Output |
|---------|-------|--------|
| Rust | `ic-codegen-rust` | `lib.rs` + per-module files with structs/enums/unions, traits for interfaces, CTS metadata. |
| C++ | `ic-codegen-cpp` | Header/implementation pairs, metadata tables, optional stream/`fmt` helpers. |
| Python | `ic-codegen-python` | Package hierarchy with `.py` modules, properties, runtime integration. |
| Protobuf | `ic-codegen-protobuf` | Proto3 `.proto` files grouped by strongly-connected components. |
| JSON | `ic-codegen-json` | Machine-readable JSON description of the schema. |
| JSON Schema | `ic-codegen-json-schema` | Draft 2019-09 JSON Schema files per type. |
| Normalised IDL | `ic-codegen-idl` | Re-serialised IDL, with optional Doxygen or legacy formatting. |
| XML | `ic-codegen-xml` | XML tree describing the HIR (used by downstream tooling). |

Backends share helper utilities via `ic-emit` (pretty-printing, indentation) and
`ic-hir-xform` (keyword escaping, case rules).

## Shared infrastructure

| Area | Crates | Notes |
|------|--------|-------|
| Diagnostics | `ic-diagnostic` | Structures and colourised rendering for errors/warnings. |
| Memory | `ic-alloc` | String interning, arenas, graph utilities used across the pipeline. |
| Virtual file system | `ic-vfs` | Source map management, span lookup, include resolution. |
| CLI / option parsing | `ic-cli`, `ic-cli-derive` | Derive-based command-line parsing with colour support. |
| Macros | `ic-macros` | Procedural macros to reduce boilerplate inside HIR/AST definitions. |
| E2E harness | `e2e-tests` crate, `crates/xtask/src/e2e.rs` | Builds the compiler, runs all backends, compiles generated code with host toolchains. |

## Runtimes and supporting libraries

Generated artefacts rely on the runtime libraries under `library/`:

- `library/rust` workspace: `intercom-cts` (serialization core),
  `intercom-build`, and `intercom-derive` derive macros for generated Rust code.
- `library/cpp/defs`: CTS headers and inline implementations used by the C++
  backend.
- Python bindings expect the external `intercom-dds` package, which mirrors the
  CTS concepts for Python.

The release helper (`cargo xtask release`) bundles these runtimes together with
the CLI so downstream consumers get a complete toolchain.

## Tooling & automation

- **`crates/xtask`**: developer utilities for formatting, license headers,
  release packaging, dependency audits (`deny`), and running the end-to-end
  suite.
- **CI (`ci/`)**: GitLab pipelines build on Alpine/Windows containers, install
  the required toolchains, run linting, build release artefacts, and execute the
  `cargo nextest` + `cargo xtask e2e` matrix.
- **Examples & fixtures**: `tests/idl/` holds small, focused IDL snippets used by
  parser/HIR tests and the end-to-end corpus.

## Extending the compiler

1. **New lint or transform** – add it under `ic-lint` or `ic-hir-xform`, expose a
   configuration flag if needed, and extend tests under the corresponding crate.
2. **New backend** – create a `ic-codegen-*` crate, consume `ResolvedGraph`, and
   register the backend in `crates/ic-idl` with a CLI flag.
3. **Runtime feature** – update the relevant library (`intercom-cts`, C++ CTS),
   then ensure every backend surfaces the capability consistently.
4. **CLI option** – extend the command structs in `crates/ic-idl/src/config.rs`
   and document the flag in the book (CLI chapter).

Always add targeted unit tests and, where applicable, a new entry in the e2e
corpus to guard against regressions.

## Data flow recap

```
CompilerOptions ──▶ File loading (ic-vfs)
                 └─▶ Lexer / Preprocessor (tokens + expansions)
                 └─▶ Parser (AST)
                 └─▶ HIR lowering (ResolvedGraph)
                 └─▶ Lints + transforms (normalised HIR)
                 └─▶ Backends (generated files + deps)
```

Diagnostics accumulate along the way; the CLI prints warnings (unless disabled)
then errors with annotated source spans. Generation continues after warnings so
users can see as many issues as possible in a single run.

---

This overview complements the backend-specific chapters and the contributing
guide. For a deep dive into runtime serialization semantics, see the backend
sections and the runtime source under `library/`.
