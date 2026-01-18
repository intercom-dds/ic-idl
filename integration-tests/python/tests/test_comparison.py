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


def test_struct_equality(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["struct_types"]
    p1 = core.Point(x=10, y=20)
    p2 = core.Point(x=10, y=20)
    p3 = core.Point(x=10, y=30)
    assert p1 == p2
    assert p1 != p3


def test_struct_ordering(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["struct_types"]
    p1 = core.Point(x=0, y=0)
    p2 = core.Point(x=1, y=0)
    p3 = core.Point(x=0, y=1)
    assert p1 < p2
    assert p1 < p3
    assert p3 < p2


def test_struct_sorting(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["struct_types"]
    points = [
        core.Point(x=2, y=2),
        core.Point(x=0, y=0),
        core.Point(x=1, y=1),
    ]
    sorted_points = sorted(points)
    assert sorted_points[0] == core.Point(x=0, y=0)
    assert sorted_points[1] == core.Point(x=1, y=1)
    assert sorted_points[2] == core.Point(x=2, y=2)


def test_struct_not_hashable(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["struct_types"]
    p1 = core.Point(x=10, y=20)
    with pytest.raises(TypeError, match="unhashable"):
        hash(p1)


def test_union_equality(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    u1 = union.IntOrString()
    u1.int_val = 42
    u2 = union.IntOrString()
    u2.int_val = 42
    u3 = union.IntOrString()
    u3.str_val = "hello"

    assert u1 == u2
    assert u1 != u3


def test_exception_equality(generated_modules: dict[str, ModuleType]) -> None:
    exc = generated_modules["exception_types"]
    e1 = exc.SimpleError(error_code=404, message="Not found")
    e2 = exc.SimpleError(error_code=404, message="Not found")
    e3 = exc.SimpleError(error_code=500, message="Server error")

    assert e1 == e2
    assert e1 != e3
