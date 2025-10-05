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

# Contributing

We welcome bug reports, feature proposals, and patches. This page summarises the
workflow and coding standards used in the repository.

## Getting started

1. Fork the repository and create a feature branch.
2. Install the toolchain listed in the [installation guide](../getting-started/installation.md).
3. Run the full test suite (`cargo nextest run --workspace --all-targets` and
   `cargo xtask e2e`) before opening a pull request.

## Code style

- **Rust** – edition 2024, `rustfmt` (nightly) with the configuration in
  `.rustfmt.toml`, and Clippy with `-D warnings`. Only the CLI crate is allowed
  to write to stdout/stderr.
- **C++** – format with `.clang-format` (4‑space indent, 100‑column width) and
  run the checks provided by `.clang-tidy`.
- **Python** – generated code is black-compatible; hand-written helpers should
  follow standard PEP 8.

Use the provided helper tasks:

```bash
cargo +nightly fmt
cargo clippy --workspace --all-targets -- -Dwarnings
cargo xtask deny      # license/dep checks
cargo xtask ipr --fix # ensure BSD headers are present
```

## Licensing

Every source file carries a BSD 3-Clause header. `cargo xtask ipr --fix` updates
missing or incorrect headers automatically. Keep contributions under the same
license unless explicitly discussed with the maintainers.

## Commit and PR guidelines

- Keep commits focused; each should compile and pass tests.
- Reference related issues in the commit message or pull request description.
- Include tests for new behaviour or bug fixes (unit tests, integration tests,
  or e2e scenarios as appropriate).
- Update the documentation (the `book/` or `docs/` directories) when changing user-facing behaviour.

## Code review

All changes are reviewed via pull requests. Expect feedback on correctness,
error handling, and documentation. The maintainers will rerun the CI pipeline,
which mirrors the commands listed above.

## Reporting issues

File issues on the project tracker with:

- A description of the problem or enhancement you are proposing.
- Steps to reproduce (when applicable) and any relevant IDL snippets.
- Information about your platform, compiler versions, and toolchain.

Security-sensitive reports should be sent privately (see `SECURITY.md` in the
repository if present, or contact the maintainers directly).
