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


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(
        "--idl-compiler",
        action="store",
        default="../target/debug/ic-idl",
        help="Path to ic-idl compiler binary",
    )
    parser.addoption(
        "--corpus",
        action="store",
        default="corpus",
        help="Path to IDL test corpus directory",
    )
    parser.addoption(
        "--java-compiler",
        action="store",
        default="javac",
        help="Path to Java compiler (javac)",
    )
    parser.addoption(
        "--dotnet",
        action="store",
        default="dotnet",
        help="Path to .NET SDK (dotnet)",
    )
    parser.addoption(
        "--protoc",
        action="store",
        default="protoc",
        help="Path to Protocol Buffers compiler (protoc)",
    )
    parser.addoption(
        "--tsc",
        action="store",
        default="tsc",
        help="Path to TypeScript compiler (tsc)",
    )
    parser.addoption(
        "--strict-skip",
        action="store_true",
        default=False,
        help="Treat skipped tests as failures",
    )


@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_makereport(item: pytest.Item, call: pytest.CallInfo):
    outcome = yield
    report = outcome.get_result()
    if (
        report.when == "setup"
        and report.skipped
        and item.config.getoption("--strict-skip")
    ):
        report.outcome = "failed"
        report.longrepr = f"skipped (treated as failure): {report.longrepr}"


@pytest.fixture(scope="session")
def idl_compiler(request: pytest.FixtureRequest) -> Path:
    path = Path(request.config.getoption("--idl-compiler"))
    if not path.exists():
        pytest.fail(f"ic-idl not found at {path}. Run 'cargo build' first.")
    return path


def get_corpus_files(corpus_dir: Path) -> list[Path]:
    return sorted(corpus_dir.glob("**/*.idl"))


def pytest_generate_tests(metafunc: pytest.Metafunc) -> None:
    if "idl_file" in metafunc.fixturenames:
        corpus_path = Path(metafunc.config.getoption("--corpus"))
        files = get_corpus_files(corpus_path)
        metafunc.parametrize("idl_file", files, ids=lambda p: p.stem)


def make_output_dir(request: pytest.FixtureRequest, lang: str) -> Path:
    test_name: str = request.node.name
    if "[" in test_name:
        test_name = test_name.split("[")[1].rstrip("]")
    test_name = test_name.replace("/", "_").replace("\\", "_").replace("..", "_")
    out = Path("..") / "target" / "e2e-tests" / lang / test_name
    out.mkdir(parents=True, exist_ok=True)
    return out


def run_codegen(
    idl_compiler: Path,
    idl_file: Path,
    output_dir: Path,
    output_flag: str,
    extra_args: list[str] | None = None,
) -> list[Path]:
    """
    Run ic-idl codegen in two passes:
    1. With -l to get list of files that will be generated
    2. Without -l to actually generate the files

    Returns list of generated file paths.
    """
    extra_args = extra_args or []
    base_args = [str(idl_compiler), f"--{output_flag}={output_dir}"] + extra_args

    result = subprocess.run(
        base_args + ["-l", str(idl_file)],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert result.returncode == 0, f"codegen -l failed:\n{result.stderr}"

    expected_files: list[Path] = []
    for line in result.stdout.splitlines():
        if line.startswith("gen:"):
            expected_files.append(Path(line[4:]))

    result = subprocess.run(
        base_args + [str(idl_file)],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert result.returncode == 0, f"codegen failed:\n{result.stderr}"

    return expected_files


@pytest.fixture(scope="session")
def corpus_dir(request: pytest.FixtureRequest) -> Path:
    path = Path(request.config.getoption("--corpus"))
    if not path.exists():
        pytest.fail(f"Corpus directory not found at {path}")
    return path


@pytest.fixture
def output_dir(request: pytest.FixtureRequest) -> Path:
    return make_output_dir(request, "python")
