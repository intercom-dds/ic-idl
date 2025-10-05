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

# Introduction

`ic-idl` is a multi-language Interface Definition Language (IDL) compiler. It
parses OMG IDL 4.x sources, validates them through a modern Rust front-end, and
emits ready-to-use code, schemas, and supporting assets for several
implementation languages.

## What does it do?

At a high level `ic-idl`:

- runs a C-style preprocessor, parser, semantic analyser, and linter over your
  `.idl` files;
- lowers the result to a High-level IR (HIR) that captures resolved types,
  annotations, and ordering;
- applies normalisation passes so all backends see a consistent view of the
  schema; and
- invokes one or more code generators to produce Rust, C++, Python, Protocol
  Buffers, JSON(+Schema), XML, or normalised IDL output.

Everything is orchestrated by a single binary (`crates/ic-idl`). Helper crates in
`library/` provide the runtime pieces that generated code depends on (for
example the Rust `intercom-cts` serialization stack and the C++ CTS headers).

## Highlights

- **One source of truth** – author types, constants, interfaces, and
  annotations once in IDL and keep all targets in sync automatically.
- **Battle-tested front-end** – more than 1,400 compiler tests cover lexing,
  preprocessing, parsing, HIR lowering, linting, and transforms.
- **Cross-language validation** – around 250 end-to-end tests generate code for
  every backend and compile/execute it to guard against regressions.
- **Extensible architecture** – each stage lives in its own crate, making it
  straightforward to add new lints, transforms, or backends.
- **Pragmatic defaults** – generated code follows the conventions of the target
  language (naming, visibility, type choices) while retaining hooks for custom
 isation through annotations.

## How to use this book

- Start with [Getting Started](./getting-started/installation.md) to build the
  compiler.
- Follow the [Quick Start](./getting-started/quickstart.md) to generate your
  first outputs.
- Dive into the [Language Reference](./guide/language-reference.md) for IDL
  syntax and annotations.
- Visit the backend chapters (`./rust/`, `./python/`, `./cpp/`, …) when you need
  language-specific guidance.
- The [Developer Documentation](./dev/architecture.md) explains the
  implementation for contributors and integrators.

## Project status

`ic-idl` is used in-house and is approaching its first public release. The CLI
and IDL surface are stabilising, but the project is still evolving: we do not
ship crates.io packages or pre-built binaries yet, and some backends expose
runtime APIs that may change before the initial stable release. Refer to the
release notes in the repository for compatibility updates.

## License

The project is distributed under the BSD 3-Clause license. Every generated
artifact carries the same header for clarity.
