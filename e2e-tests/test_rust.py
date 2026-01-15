# Copyright 2026 KONGSBERG
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

import shutil
import subprocess
from pathlib import Path

import pytest

from conftest import make_output_dir, run_codegen

CARGO_TOML_TEMPLATE = """[package]
name = "generated"
version = "0.0.0"
edition = "2024"

[workspace]

[dependencies]
intercom-cts = {{ path = "{cts_path}" }}
"""


@pytest.fixture(scope="session")
def cargo(request: pytest.FixtureRequest) -> str:
    path = request.config.getoption("--cargo")
    if not shutil.which(path):
        pytest.skip(f"cargo not found: {path}")
    return path


@pytest.fixture(scope="session")
def cts_path() -> Path:
    root = Path(__file__).parent.parent
    return (root / "library" / "rust" / "intercom-cts").resolve()


@pytest.fixture
def rust_output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "rust")


def test_rust(
    idl_file: Path,
    idl_compiler: Path,
    cargo: str,
    rust_output_dir: Path,
    cts_path: Path,
) -> None:
    src_dir = rust_output_dir / "src"
    src_dir.mkdir(exist_ok=True)

    rs_files = run_codegen(idl_compiler, idl_file, src_dir, "rust-out")
    if not rs_files:
        return

    cargo_toml = rust_output_dir / "Cargo.toml"
    cargo_toml.write_text(CARGO_TOML_TEMPLATE.format(cts_path=cts_path))

    result = subprocess.run(
        [cargo, "check", "--quiet", "--target-dir=target"],
        cwd=rust_output_dir,
        capture_output=True,
        text=True,
        timeout=120,
    )
    assert result.returncode == 0, (
        f"cargo check failed:\n{result.stdout}\n{result.stderr}"
    )
