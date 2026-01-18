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


def test_bounded_string_typedef_maps_to_str(
    generated_modules: dict[str, ModuleType],
) -> None:
    bt = generated_modules["bounded_types"]
    assert bt.ShortString == "str"
    assert bt.MediumString == "str"
    assert bt.LongString == "str"


def test_bounded_sequence_typedef_maps_to_list(
    generated_modules: dict[str, ModuleType],
) -> None:
    bt = generated_modules["bounded_types"]
    assert bt.SmallIntList == "list[int]"
    assert bt.StringList100 == "list[str]"
    assert bt.LargeDoubleList == "list[float]"


def test_bounded_fields_struct(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    s = bt.BoundedFields(
        name="test",
        description="A longer description",
        values=[1, 2, 3],
        tags=["a", "b"],
    )
    assert s.name == "test"
    assert s.description == "A longer description"
    assert s.values == [1, 2, 3]
    assert s.tags == ["a", "b"]


def test_bounded_fields_annotations(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    annotations = bt.BoundedFields.__annotations__
    assert annotations["name"] == "str"
    assert annotations["description"] == "str"
    assert annotations["values"] == "list[int]"
    assert annotations["tags"] == "list[str]"


def test_nested_bounded_struct(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    s = bt.NestedBounded(
        matrix=[[1, 2], [3, 4]],
        indexed_lists={"a": [1, 2, 3], "b": [4, 5, 6]},
    )
    assert s.matrix == [[1, 2], [3, 4]]
    assert s.indexed_lists["a"] == [1, 2, 3]


def test_nested_bounded_annotations(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    annotations = bt.NestedBounded.__annotations__
    assert annotations["matrix"] == "list[list[int]]"
    assert annotations["indexed_lists"] == "dict[str, list[int]]"


def test_typedef_chain_with_bounds(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    assert bt.Name == "str"
    assert bt.NameList == "list[Name]"
    assert bt.NameMap == "dict[Name, NameList]"


def test_mixed_bounds_struct(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    s = bt.MixedBounds(
        bounded_string="bounded",
        unbounded_string="unbounded" * 100,
        bounded_seq=[1, 2, 3],
        unbounded_seq=list(range(1000)),
    )
    assert s.bounded_string == "bounded"
    assert len(s.unbounded_string) == 900
    assert s.bounded_seq == [1, 2, 3]
    assert len(s.unbounded_seq) == 1000


def test_mixed_bounds_annotations(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bounded_types"]
    annotations = bt.MixedBounds.__annotations__
    assert annotations["bounded_string"] == "str"
    assert annotations["unbounded_string"] == "str"
    assert annotations["bounded_seq"] == "list[int]"
    assert annotations["unbounded_seq"] == "list[int]"


def test_bounds_not_enforced_at_runtime(
    generated_modules: dict[str, ModuleType],
) -> None:
    bt = generated_modules["bounded_types"]
    s = bt.BoundedFields(
        name="x" * 1000,
        description="y" * 10000,
        values=list(range(1000)),
        tags=["tag"] * 500,
    )
    assert len(s.name) == 1000
    assert len(s.description) == 10000
    assert len(s.values) == 1000
    assert len(s.tags) == 500
