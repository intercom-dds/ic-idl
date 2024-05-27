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

Work in progress. Things are scattered all over the place. Relative paths are
used everywhere in CMake as a temporary hack to make things work.

## Building

Building `ic-idl` requires a C++17 toolchain and a Rust 1.74+ toolchain.

`ic-idl` is bootstrapped. To build a full-fledged version, we first need to
compile a reduced, C++-only version that is capable of emitting simplified type
definitions which will be used in the full-fledged version.

The bootstrap process is automated through the `xtask` application. To build a
full release:

```sh
cargo xtask release
```

The system's default C/C++ toolchain will be used unless otherwise is
specified. This can be overridden by using the `CC` and `CXX` environment
variables. Custom flags can be specified with `CFLAGS` and `CXXFLAGS`,
respectively.

### Development

Build a bootstrapped version first:

```sh
cargo xtask bootstrap
```

Once compiled, `cargo` can be invoked as usual, e.g.:

```
cargo test --workspace
```

To use the experimental new parser, you must enable the `hir` feature:

```
cargo run --features hir --no-default-features
```

Development documentation can be generated with:

```
cargo doc --document-private-items --no-deps --workspace
```

## MSRV

- MSRV for the compiler is 1.74.
- MSRV for the serialization library is 1.70.

The MSRV may change between minor version releases and is not considered a
semver-breaking change.
