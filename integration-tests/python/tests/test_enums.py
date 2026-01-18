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

import enum
from types import ModuleType


def test_enum_members_exist(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert hasattr(core.Color, "RED")
    assert hasattr(core.Color, "GREEN")
    assert hasattr(core.Color, "BLUE")


def test_enum_is_enum_type(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert issubclass(core.Color, enum.Enum)


def test_enum_auto_values(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.Color.RED.value == 0
    assert core.Color.GREEN.value == 1
    assert core.Color.BLUE.value == 2


def test_enum_explicit_values(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.Status.OK.value == 0
    assert core.Status.WARNING.value == 100
    assert core.Status.ERROR.value == 200


def test_enum_iteration(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    colors = list(core.Color)
    assert len(colors) == 3
    assert core.Color.RED in colors
    assert core.Color.GREEN in colors
    assert core.Color.BLUE in colors


def test_enum_comparison(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.Color.RED == core.Color.RED
    assert core.Color.RED != core.Color.BLUE


def test_enum_by_value(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.Color(0) == core.Color.RED
    assert core.Status(100) == core.Status.WARNING


def test_enum_by_name(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.Color["RED"] == core.Color.RED
    assert core.Status["ERROR"] == core.Status.ERROR


def test_enum_name_property(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.Color.RED.name == "RED"
    assert core.Status.WARNING.name == "WARNING"


def test_enum_gapped_values(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.GappedEnum.FIRST.value == 0
    assert core.GappedEnum.SECOND.value == 5
    assert core.GappedEnum.THIRD.value == 10
    assert core.GappedEnum.FOURTH.value == 100


def test_enum_negative_values(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.NegativeEnum.NEG_TWO.value == -2
    assert core.NegativeEnum.NEG_ONE.value == -1
    assert core.NegativeEnum.ZERO.value == 0
    assert core.NegativeEnum.POS_ONE.value == 1


def test_enum_const_from_enum_value(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.ENUM_CONST == core.Status.WARNING


def test_enum_mixed_explicit_auto(generated_modules: dict[str, ModuleType]) -> None:
    core = generated_modules["enum_types"]
    assert core.MixedEnum.AUTO_FIRST.value == 0
    assert core.MixedEnum.EXPLICIT_TEN.value == 10
    assert core.MixedEnum.AUTO_ELEVEN.value == 11
    assert core.MixedEnum.EXPLICIT_HUNDRED.value == 100
    assert core.MixedEnum.AUTO_HUNDRED_ONE.value == 101
