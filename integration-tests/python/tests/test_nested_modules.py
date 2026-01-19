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


def test_top_level_types_exist(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert hasattr(mod, "TopLevelStruct")
    assert hasattr(mod, "TopLevelEnum")


def test_nested_module_level1_exists(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert hasattr(mod, "level1")
    assert hasattr(mod.level1, "Level1Struct")
    assert hasattr(mod.level1, "Level1Enum")


def test_nested_module_level2_exists(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert hasattr(mod.level1, "level2")
    assert hasattr(mod.level1.level2, "Level2Struct")


def test_nested_module_level3_exists(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert hasattr(mod.level1.level2, "level3")
    assert hasattr(mod.level1.level2.level3, "Level3Struct")
    assert hasattr(mod.level1.level2.level3, "DEEP_CONST")


def test_sibling_module_exists(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert hasattr(mod, "sibling")
    assert hasattr(mod.sibling, "SiblingStruct")
    assert hasattr(mod.sibling, "CrossRef")


def test_top_level_struct_instantiation(
    generated_modules: dict[str, ModuleType],
) -> None:
    mod = generated_modules["nested_module_types"]
    s = mod.TopLevelStruct(value=42)
    assert s.value == 42


def test_level1_struct_with_parent_ref(
    generated_modules: dict[str, ModuleType],
) -> None:
    mod = generated_modules["nested_module_types"]
    parent = mod.TopLevelStruct(value=1)
    s = mod.level1.Level1Struct(data=10, parent_ref=parent)
    assert s.data == 10
    assert s.parent_ref.value == 1


def test_level2_struct_with_refs(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    top = mod.TopLevelStruct(value=1)
    l1 = mod.level1.Level1Struct(data=2, parent_ref=top)
    l2 = mod.level1.level2.Level2Struct(name="test", level1_ref=l1, top_ref=top)
    assert l2.name == "test"
    assert l2.level1_ref.data == 2
    assert l2.top_ref.value == 1


def test_level3_struct_with_all_refs(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    top = mod.TopLevelStruct(value=1)
    l1 = mod.level1.Level1Struct(data=2, parent_ref=top)
    l2 = mod.level1.level2.Level2Struct(name="l2", level1_ref=l1, top_ref=top)
    l3 = mod.level1.level2.level3.Level3Struct(
        id=100,
        level2_ref=l2,
        level1_ref=l1,
        top_ref=top,
    )
    assert l3.id == 100
    assert l3.level2_ref.name == "l2"
    assert l3.level1_ref.data == 2
    assert l3.top_ref.value == 1


def test_deep_constant(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert mod.level1.level2.level3.DEEP_CONST == 42


def test_sibling_cross_ref_struct(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    top = mod.TopLevelStruct(value=1)
    l1 = mod.level1.Level1Struct(data=2, parent_ref=top)
    l2 = mod.level1.level2.Level2Struct(name="l2", level1_ref=l1, top_ref=top)
    l3 = mod.level1.level2.level3.Level3Struct(
        id=3,
        level2_ref=l2,
        level1_ref=l1,
        top_ref=top,
    )
    cross = mod.sibling.CrossRef(from_level1=l1, from_level2=l2, from_level3=l3)
    assert cross.from_level1.data == 2
    assert cross.from_level2.name == "l2"
    assert cross.from_level3.id == 3


def test_top_using_nested_struct(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    top = mod.TopLevelStruct(value=1)
    l1 = mod.level1.Level1Struct(data=2, parent_ref=top)
    l2 = mod.level1.level2.Level2Struct(name="l2", level1_ref=l1, top_ref=top)
    l3 = mod.level1.level2.level3.Level3Struct(
        id=3,
        level2_ref=l2,
        level1_ref=l1,
        top_ref=top,
    )
    sib = mod.sibling.SiblingStruct(id=4)
    using = mod.TopUsingNested(l1=l1, l2=l2, l3=l3, sib=sib)
    assert using.l1.data == 2
    assert using.l2.name == "l2"
    assert using.l3.id == 3
    assert using.sib.id == 4


def test_level1_enum(generated_modules: dict[str, ModuleType]) -> None:
    mod = generated_modules["nested_module_types"]
    assert mod.level1.Level1Enum.A.value == 0
    assert mod.level1.Level1Enum.B.value == 1
    assert mod.level1.Level1Enum.C.value == 2
