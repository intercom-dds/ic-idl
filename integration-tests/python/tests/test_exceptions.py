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

from types import ModuleType

import pytest


def test_exception_inherits_from_exception(
    generated_modules: dict[str, ModuleType],
) -> None:
    exc = generated_modules["exception_types"]
    assert issubclass(exc.SimpleError, Exception)


def test_exception_instantiation(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]
    e = exc.SimpleError(error_code=404, message="Not found")
    assert e.error_code == 404
    assert e.message == "Not found"


def test_exception_raise_and_catch(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]

    with pytest.raises(exc.SimpleError) as exc_info:
        raise exc.SimpleError(error_code=500, message="Internal error")

    assert exc_info.value.error_code == 500
    assert exc_info.value.message == "Internal error"


def test_exception_catch_as_base(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]

    with pytest.raises(exc.SimpleError) as exc_info:
        raise exc.SimpleError(error_code=400, message="Bad request")

    assert isinstance(exc_info.value, exc.SimpleError)
    assert issubclass(type(exc_info.value), Exception)


def test_empty_exception(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]
    e = exc.EmptyError()
    assert isinstance(e, Exception)


def test_detailed_exception_fields(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]
    e = exc.DetailedError(
        code=123,
        message="Something went wrong",
        details="Additional context here",
        recoverable=True,
    )
    assert e.code == 123
    assert e.message == "Something went wrong"
    assert e.details == "Additional context here"
    assert e.recoverable is True


def test_validation_error(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]
    e = exc.ValidationError(
        field_name="email",
        error_message="Invalid email format",
        position=10,
    )
    assert e.field_name == "email"
    assert e.error_message == "Invalid email format"
    assert e.position == 10
