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


class CxxCompiler(NamedTuple):
    path: str
    kind: str


def detect_cxx_compiler(requested: str | None) -> CxxCompiler | None:
    """Detect available C++ compiler, preferring the requested one if specified."""
    if requested:
        path = shutil.which(requested)
        if path:
            kind = _detect_compiler_kind(requested)
            return CxxCompiler(path, kind)
        return None

    if platform.system() == "Windows":
        candidates = ["cl.exe", "clang++.exe"]
    else:
        candidates = ["g++", "clang++"]

    for candidate in candidates:
        path = shutil.which(candidate)
        if path:
            kind = _detect_compiler_kind(candidate)
            return CxxCompiler(path, kind)

    return None


def _detect_compiler_kind(name: str) -> str:
    """Determine compiler kind from executable name."""
    name_lower = name.lower()
    if "cl" in name_lower and "clang" not in name_lower:
        return "msvc"
    if "clang" in name_lower:
        return "clang"
    return "gcc"


def get_warning_flags(kind: str) -> list[str]:
    """Get warning flags appropriate for the compiler kind."""
    if kind == "msvc":
        return [
            "/W4",
            "/WX",
            "/permissive-",
            "/Zc:__cplusplus",
            "/std:c++17",
            "/EHsc",
        ]
    else:
        return [
            "-Wall",
            "-Wextra",
            "-Wpedantic",
            "-Werror",
            "-Wconversion",
            "-Wshadow",
            "-Wnon-virtual-dtor",
            "-Wold-style-cast",
            "-Woverloaded-virtual",
            "-Wno-switch-bool",
            "-std=c++17",
        ]


def get_syntax_check_flags(kind: str) -> list[str]:
    """Get flags for syntax-only checking (no linking)."""
    if kind == "msvc":
        return ["/Zs"]
    else:
        return ["-fsyntax-only"]


@pytest.fixture(scope="session")
def cxx_compiler(request: pytest.FixtureRequest) -> CxxCompiler:
    requested = request.config.getoption("--cpp-compiler")
    compiler = detect_cxx_compiler(requested)
    if compiler is None:
        pytest.skip("No C++ compiler found")
    assert compiler is not None
    return compiler


@pytest.fixture(scope="session")
def cpp_include_path() -> Path:
    root = Path(__file__).parent.parent.parent
    return (root / "runtime" / "cpp" / "include").resolve()


@pytest.fixture
def cpp_output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "cpp")


def test_cpp(
    idl_file: Path,
    idl_compiler: Path,
    cxx_compiler: CxxCompiler,
    cpp_output_dir: Path,
    cpp_include_path: Path,
) -> None:
    generated_files = run_codegen(idl_compiler, idl_file, cpp_output_dir, "cpp-out")
    if not generated_files:
        return

    source_files = [f for f in generated_files if f.suffix == ".cpp"]
    if not source_files:
        return

    warning_flags = get_warning_flags(cxx_compiler.kind)
    syntax_flags = get_syntax_check_flags(cxx_compiler.kind)

    if cxx_compiler.kind == "msvc":
        include_flags = [f"/I{cpp_include_path}", f"/I{cpp_output_dir}"]
    else:
        include_flags = [f"-I{cpp_include_path}", f"-I{cpp_output_dir}"]

    for source in source_files:
        cmd = [
            cxx_compiler.path,
            *warning_flags,
            *syntax_flags,
            *include_flags,
            str(source),
        ]

        result = subprocess.run(
            cmd,
            check=False,
            capture_output=True,
            text=True,
            timeout=60,
        )

        assert result.returncode == 0, (
            f"C++ compilation failed for {source}:\n"
            f"Command: {' '.join(cmd)}\n"
            f"stdout: {result.stdout}\n"
            f"stderr: {result.stderr}"
        )
