<!--
    Copyright 2024 KONGSBERG

    Redistribution and use in source and binary forms, with or without
    modification, are permitted provided that the following conditions are met:

    1. Redistributions of source code must retain the above copyright notice,
       this list of conditions and the following disclaimer.

    2. Redistributions in binary form must reproduce the above copyright notice,
       this list of conditions and the following disclaimer in the documentation
       and/or other materials provided with the distribution.

    3. Neither the name of the copyright holder nor the names of its contributors
       may be used to endorse or promote products derived from this software
       without specific prior written permission.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
    ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
    WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
    DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
    FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
    DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
    SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
    CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
    OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
    OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
-->

# `ic-idl`

**Generic, multi-target OMG IDL compiler written in Rust.**

ic-idl parses [OMG IDL4](https://www.omg.org/spec/IDL/4.2) interface definitions and generates type
definitions, interfaces, and serialization code for multiple target languages. It supports C++,
Rust, C#, Java, Python, and TypeScript. It can also convert IDL to other schema formats including
Protobuf, JSON, JSON Schema, and XML.

In addition to being a full IDL compiler, ic-idl is designed as a modular compilation pipeline.
Components such as the lexer, C-compliant preprocessor, IDL parser, and type-resolved IR are
available as standalone crates for projects that only need IDL parsing or analysis. The pipeline is
designed to be easily extended with new code generation backends, and can even be extended to
support other input languages beyond IDL.

## Installation

To create a release archive with the binary and runtime libraries:

```sh
cargo xtask release
```

To build and run the compiler from source:

```sh
cargo run --release
```

## Development

Run unit tests:

```sh
cargo nextest run --workspace --all-targets
```

---

Run codegen tests:

```sh
cargo xtask codegen
```

See the `codegen-tests` directory for more information.

---

Run integration tests:

```sh
cargo xtask integration
```

See the `integration-tests` directory for more information.

---

Generate development documentation:

```sh
cargo doc --document-private-items --no-deps --workspace
```
