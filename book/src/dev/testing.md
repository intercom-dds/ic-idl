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

# Testing strategy

`ic-idl` has several layers of automated tests. Running the full suite locally
before sending a change avoids most CI surprises.

## Commands

```bash
cargo nextest run --workspace --all-targets   # fast unit & integration tests
cargo xtask e2e                               # generate+compile all backends
cargo xtask deny                              # dependency / license audit
cargo xtask ipr --fix                         # verify license headers
```

`cargo nextest` exercises every crate: preprocessor edge cases, parser
regressions, HIR lowering, lints, and backend-specific unit tests. The e2e task
invokes the CLI, generates code for each backend, and compiles or type-checks
the results (Rust, C++, Python, Protocol Buffers, JSON/Schema, IDL, XML).

## Test layers

- **Unit tests** – live next to the relevant code (e.g. `crates/ic-preproc`,
  `crates/ic-hir-xform`). Add new cases when touching low-level functionality.
- **Snapshot tests** – some crates (parsers, lints) use snapshot comparisons.
  Regenerate snapshots consciously and review the diffs; do not blindly accept
  updates.
- **Corpus-based tests** – `tests/idl/` contains IDL corpora used across
  integration and e2e tests. Keep new fixtures small but representative.
- **End-to-end tests** – `cargo xtask e2e` runs in parallel and will skip a
  backend if the required toolchain is missing. Install `clang++`, `python3`,
  and `protoc` locally to run everything.

## Adding tests

1. Prefer the narrowest layer that covers the bug or feature.
2. Add focused IDL snippets under `tests/idl/` for cross-backend regressions.
3. When creating new transforms or lints, extend the existing snapshot tests or
   add targeted cases rather than large fixtures.
4. Update documentation or release notes if behaviour changes in a user-visible
   way.

## Continuous integration

The GitLab pipelines run the same commands listed above on Linux and Windows
containers defined in `ci/`. Use them as references for necessary system
packages and environment configuration.
