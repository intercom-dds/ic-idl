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


def test_module_a_exists(generated_modules: dict[str, ModuleType]) -> None:
    assert "module_a" in generated_modules


def test_module_b_exists(generated_modules: dict[str, ModuleType]) -> None:
    assert "module_b" in generated_modules


def test_module_a_first_opening(generated_modules: dict[str, ModuleType]) -> None:
    ma = generated_modules["module_a"]
    assert hasattr(ma, "StructA1")
    assert hasattr(ma, "CONST_A1")
    assert hasattr(ma, "EnumA")
    assert ma.CONST_A1 == 100


def test_module_a_second_opening(generated_modules: dict[str, ModuleType]) -> None:
    ma = generated_modules["module_a"]
    assert hasattr(ma, "StructA2")
    assert hasattr(ma, "CONST_A2")
    assert hasattr(ma, "EnumA2")
    assert ma.CONST_A2 == 101


def test_module_a_third_opening(generated_modules: dict[str, ModuleType]) -> None:
    ma = generated_modules["module_a"]
    assert hasattr(ma, "StructA3")
    assert hasattr(ma, "CONST_A3")
    assert ma.CONST_A3 == 102


def test_module_b_both_openings(generated_modules: dict[str, ModuleType]) -> None:
    mb = generated_modules["module_b"]
    assert hasattr(mb, "StructB1")
    assert hasattr(mb, "CONST_B1")
    assert hasattr(mb, "StructB2")
    assert hasattr(mb, "CONST_B2")
    assert mb.CONST_B1 == 200
    assert mb.CONST_B2 == 201


def test_reopened_module_types_can_reference_earlier(
    generated_modules: dict[str, ModuleType],
) -> None:
    ma = generated_modules["module_a"]
    a1 = ma.StructA1(value=10)
    a2 = ma.StructA2(data=3.14, ref_to_a1=a1)
    assert a2.ref_to_a1.value == 10


def test_reopened_module_chain(generated_modules: dict[str, ModuleType]) -> None:
    ma = generated_modules["module_a"]
    a1 = ma.StructA1(value=1)
    a2 = ma.StructA2(data=2.0, ref_to_a1=a1)
    a3 = ma.StructA3(flag=True, a1=a1, a2=a2)
    assert a3.flag is True
    assert a3.a1.value == 1
    assert a3.a2.data == 2.0


def test_constants_only_module(generated_modules: dict[str, ModuleType]) -> None:
    co = generated_modules["constants_only"]
    assert co.C1 == 1
    assert co.C2 == 2
    assert co.C3 == 3


def test_enums_only_module(generated_modules: dict[str, ModuleType]) -> None:
    eo = generated_modules["enums_only"]
    assert hasattr(eo, "Color")
    assert hasattr(eo, "Size")
    assert eo.Color.RED.value == 0
    assert eo.Size.LARGE.value == 2
