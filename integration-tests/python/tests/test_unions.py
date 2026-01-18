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


def test_union_int_variant(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    u = union.IntOrString()
    u.int_val = 42
    assert u.discriminator == 1
    assert u.int_val == 42


def test_union_string_variant(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    u = union.IntOrString()
    u.str_val = "hello"
    assert u.discriminator == 2
    assert u.str_val == "hello"


def test_union_wrong_variant_raises(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    u = union.IntOrString()
    u.int_val = 42
    with pytest.raises(ValueError, match="str_val not selected"):
        _ = u.str_val


def test_union_enum_discriminator(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    tv = union.TypedValue()
    tv.int_value = 100
    assert tv.discriminator == union.ValueKind.INT_KIND
    assert tv.int_value == 100


def test_union_enum_string_variant(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    tv = union.TypedValue()
    tv.string_value = "test"
    assert tv.discriminator == union.ValueKind.STRING_KIND
    assert tv.string_value == "test"


def test_union_bool_discriminator(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    bs = union.BoolSwitch()
    bs.true_val = 999
    assert bs.discriminator is True
    assert bs.true_val == 999

    bs.false_val = "negative"
    assert bs.discriminator is False
    assert bs.false_val == "negative"


def test_union_multi_case(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    mc = union.MultiCase()
    mc.small_val = 5
    assert mc.discriminator == 1
    assert mc.small_val == 5


def test_union_default_method(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    u = union.IntOrString()
    u.int_val = 42
    u.default()
    assert u._value is None  # noqa: SLF001


def test_union_discriminator_property(generated_modules: dict[str, ModuleType]) -> None:
    union = generated_modules["union_types"]
    u = union.IntOrString()
    assert hasattr(u, "discriminator")
