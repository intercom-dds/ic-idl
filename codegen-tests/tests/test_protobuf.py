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

import os
import shutil
import subprocess
from pathlib import Path

import pytest

from conftest import make_output_dir, run_codegen


@pytest.fixture(scope="session")
def protoc(request: pytest.FixtureRequest) -> str:
    path = request.config.getoption("--protoc")
    if not shutil.which(path):
        pytest.skip(f"protoc not found: {path}")
    return path


@pytest.fixture
def protobuf_output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "protobuf")


def test_protobuf(
    idl_file: Path,
    idl_compiler: Path,
    protoc: str,
    protobuf_output_dir: Path,
) -> None:
    proto_files = run_codegen(idl_compiler, idl_file, protobuf_output_dir, "proto-out")
    if not proto_files:
        return

    result = subprocess.run(
        [
            protoc,
            f"--proto_path={protobuf_output_dir}",
            f"--descriptor_set_out={os.devnull}",
        ]
        + [str(f) for f in proto_files],
        check=False,
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert result.returncode == 0, f"protoc failed:\n{result.stderr}"
