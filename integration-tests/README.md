<!--
    Copyright 2026 KONGSBERG

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

# Integration tests

This directory contains the integration tests for all the language backends supported by ic-idl.
Unlike the codegen tests, these are end-to-end tests that validate the structure and semantics of
the types generated for each language. As an example, we validate that every struct we expect to be
generated actually exists, that it has the correct members, correct default values, etc. There are
also functional tests that ensure equality and comparison operators behave as expected.

## Running the tests

The easiest way to run the tests is through `xtask`:

```bash
# run all tests
cargo xtask integration

# run only Python tests
cargo xtask integration -l python
```

## Dependencies

For language-specific tests, you'll need the following tools:

- **C++**: A C++17 toolchain and `cmake`
- **C#**: `dotnet`
- **Java**: `javac` and `maven`
- **Python**: `uv`
- **Rust**: `cargo`
- **TypeScript**: `bun`
