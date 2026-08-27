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


@pytest.fixture(scope="session")
def tsc(request: pytest.FixtureRequest) -> str:
    path = request.config.getoption("--tsc")
    if not shutil.which(path):
        pytest.skip(f"TypeScript compiler not found: {path}")
    return path


@pytest.fixture
def typescript_output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "typescript")


RESOLUTION_MODES = [("ESNext", "bundler"), ("nodenext", "nodenext")]


def test_typescript(
    idl_file: Path, idl_compiler: Path, tsc: str, typescript_output_dir: Path
) -> None:
    ts_files = run_codegen(
        idl_compiler, idl_file, typescript_output_dir, "typescript-out"
    )
    if not ts_files:
        return

    (typescript_output_dir / "package.json").write_text('{"type": "module"}')

    for module, resolution in RESOLUTION_MODES:
        result = subprocess.run(
            [
                tsc,
                "--noEmit",
                "--strict",
                "--skipLibCheck",
                "--module",
                module,
                "--moduleResolution",
                resolution,
                "--target",
                "ESNext",
                *[str(f) for f in ts_files],
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=60,
        )
        assert result.returncode == 0, (
            f"tsc failed ({resolution}):\n{result.stdout}\n{result.stderr}"
        )
