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

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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

# ic-idl

A generic IDL compiler.

## Building

Building `ic-idl` requires a C++17 toolchain and a Rust 1.80+ toolchain.

The system's default C++ toolchain will be used unless otherwise is specified.
This can be overridden by using the `CXX` environment variable, and custom
flags can be specified with `CXXFLAGS`.

For working with the C++ code, you can use the top-level `CMakeLists.txt` to
initialize a project and generate a `compile_commands.json`. The CMake project
*only* exists to generate said file -- it is not used by Cargo, and any
artifacts compiled using it will not be included in the `ic-idl` binary.

## Installation

To build a full release, you can use `xtask`:

```sh
cargo xtask release
```

This will compile a release version of `ic-idl` and create an archive that
contains the binary and serialization libraries.

## Development

Run all tests:

```
cargo test --workspace
```

Development documentation can be generated with:

```
cargo doc --document-private-items --no-deps --workspace
```

## MSRV

- MSRV for the compiler is 1.80.
- MSRV for the serialization library is 1.70.

The MSRV may change between minor version releases and is not considered a
semver-breaking change.

## Known bugs

- Annotations are parsed but not included in the AST.
- Variadic macros are not supported.
- \_Pragma is not supported.
- `#line` directives are ignored by the preprocessor.
- All diagnostics report "unknown" as filename.
