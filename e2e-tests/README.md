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

# End-to-end tests

This directory contains the e2e test suite for validating generated code across all backends. Each
file placed in the `corpus` directory will be considered its own test case, and will be tested
against all available backends.

## Running the tests

The easiest way to run the tests is through `xtask`:

```bash
# build ic-idl first
cargo build

# run all tests
cargo xtask e2e

# run only Python tests
cargo xtask e2e -l python
```

You can also run `pytest` directly if you prefer:

```bash
cd e2e-tests
uv run pytest . -n auto
```

## Dependencies

The test runner uses [uv](https://docs.astral.sh/uv/) to manage Python dependencies. You'll need
`uv` installed, but it handles everything else (`pytest`, `ruff`, `ty`, etc.) automatically.

For language-specific tests, you'll need the corresponding toolchain installed:

- **Python**: `ruff` and `ty`
- **Java**: `javac` (JDK 8 or newer)
- **C#**: `dotnet` (.NET 8.0 SDK)
- **TypeScript**: `tsc` (usually via `npm install -g typescript`)
- **Rust**: `cargo`
- **Protobuf**: `protoc`

Tests will be skipped if a toolchain isn't available, so you don't need everything installed to run
the suite.
