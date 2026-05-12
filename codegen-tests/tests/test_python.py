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

import subprocess
from pathlib import Path

import pytest

from conftest import run_codegen


@pytest.mark.parametrize("extra_args", [
    pytest.param([], id="default"),
    pytest.param(["--no-rename"], id="no-rename"),
    pytest.param(["--py-typed"], id="py-typed"),
])
def test_python(
    idl_file: Path,
    idl_compiler: Path,
    output_dir: Path,
    extra_args: list[str],
) -> None:
    py_files = run_codegen(
        idl_compiler, idl_file, output_dir, "python-out", extra_args
    )
    if not py_files:
        return

    result = subprocess.run(
        ["uvx", "ruff", "check", str(output_dir)],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert result.returncode == 0, f"ruff failed:\n{result.stdout}\n{result.stderr}"

    result = subprocess.run(
        [
            "uvx",
            "ty",
            "check",
            f"--extra-search-path={output_dir.parent}",
            str(output_dir),
        ],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert result.returncode == 0, f"ty failed:\n{result.stdout}\n{result.stderr}"
