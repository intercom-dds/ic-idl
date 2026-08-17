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

import platform
import shutil
import subprocess
from pathlib import Path
from typing import NamedTuple

import pytest

from conftest import make_output_dir, run_codegen


class CCompiler(NamedTuple):
    path: str
    kind: str


def detect_c_compiler(requested: str | None) -> CCompiler | None:
    if requested:
        path = shutil.which(requested)
        if path:
            kind = _detect_compiler_kind(requested)
            return CCompiler(path, kind)

    if platform.system() == "Windows":
        candidates = ["cl.exe", "clang.exe"]
    else:
        candidates = ["cc", "gcc", "clang"]

    for candidate in candidates:
        path = shutil.which(candidate)
        if path:
            kind = _detect_compiler_kind(candidate)
            return CCompiler(path, kind)

    return None


def _detect_compiler_kind(name: str) -> str:
    name_lower = Path(name).name.lower()
    if "cl" in name_lower and "clang" not in name_lower:
        return "msvc"
    if "clang" in name_lower:
        return "clang"
    return "gcc"


def get_warning_flags(kind: str) -> list[str]:
    if kind == "msvc":
        return [
            "/W4",
            "/WX",
            "/permissive-",
            "/std:c11",
            "/TC",
        ]
    else:
        return [
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
        ]


def get_syntax_check_flags(kind: str) -> list[str]:
    if kind == "msvc":
        return ["/Zs"]
    else:
        return ["-fsyntax-only"]


@pytest.fixture(scope="session")
def c_compiler(request: pytest.FixtureRequest) -> CCompiler:
    requested = request.config.getoption("--c-compiler")
    compiler = detect_c_compiler(requested)
    if compiler is None:
        pytest.skip("No C compiler found")
    assert compiler is not None
    return compiler


@pytest.fixture(scope="session")
def c_include_path() -> Path:
    root = Path(__file__).parent.parent.parent
    return (root / "runtime" / "c" / "include").resolve()


@pytest.fixture
def c_output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "c")


def test_c(
    idl_file: Path,
    idl_compiler: Path,
    c_compiler: CCompiler,
    c_output_dir: Path,
    c_include_path: Path,
) -> None:
    generated_files = run_codegen(idl_compiler, idl_file, c_output_dir, "c-out")
    if not generated_files:
        return

    headers = [f for f in generated_files if f.suffix == ".h"]
    if not headers:
        return

    check_file = c_output_dir / "check.c"
    check_file.write_text("".join(f'#include "{header.name}"\n' for header in headers))

    warning_flags = get_warning_flags(c_compiler.kind)
    syntax_flags = get_syntax_check_flags(c_compiler.kind)

    if c_compiler.kind == "msvc":
        include_flags = [f"/I{c_include_path}", f"/I{c_output_dir}"]
    else:
        include_flags = [f"-I{c_include_path}", f"-I{c_output_dir}"]

    cmd = [
        c_compiler.path,
        *warning_flags,
        *syntax_flags,
        *include_flags,
        str(check_file),
    ]

    result = subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        timeout=60,
    )

    assert result.returncode == 0, (
        f"C compilation failed for {check_file}:\n"
        f"Command: {' '.join(cmd)}\n"
        f"stdout: {result.stdout}\n"
        f"stderr: {result.stderr}"
    )
