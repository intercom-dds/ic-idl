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


def test_two_level_seq(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.TwoLevelSeq()
    assert t.matrix == []
    t.matrix = [[1, 2], [3, 4, 5]]
    assert t.matrix[0] == [1, 2]
    assert t.matrix[1][2] == 5


def test_three_level_seq(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.ThreeLevelSeq()
    assert t.cube == []
    t.cube = [[[1, 2], [3]], [[4, 5, 6]]]
    assert t.cube[0][0][1] == 2
    assert t.cube[1][0][2] == 6


def test_four_level_deep(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.FourLevelDeep()
    assert t.hypercube == []
    t.hypercube = [[[[1]]]]
    assert t.hypercube[0][0][0][0] == 1


def test_map_of_seq(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.MapOfSeq()
    assert t.indexed_lists == {}
    t.indexed_lists = {"nums": [1, 2, 3], "more": [4, 5]}
    assert t.indexed_lists["nums"] == [1, 2, 3]
    assert t.indexed_lists["more"][1] == 5


def test_seq_of_map(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.SeqOfMap()
    assert t.list_of_dicts == []
    t.list_of_dicts = [{"a": 1}, {"b": 2, "c": 3}]
    assert t.list_of_dicts[0]["a"] == 1
    assert t.list_of_dicts[1]["c"] == 3


def test_map_of_map(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.MapOfMap()
    assert t.nested_dict == {}
    t.nested_dict = {"outer1": {"inner": 42}, "outer2": {"k": 1}}
    assert t.nested_dict["outer1"]["inner"] == 42


def test_map_seq_map(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.MapSeqMap()
    assert t.complex_structure == {}
    t.complex_structure = {"key": [{"a": 1}, {"b": 2}]}
    assert t.complex_structure["key"][0]["a"] == 1
    assert t.complex_structure["key"][1]["b"] == 2


def test_seq_map_seq(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.SeqMapSeq()
    assert t.inverse_structure == []
    t.inverse_structure = [{"x": [1, 2]}, {"y": [3, 4, 5]}]
    assert t.inverse_structure[0]["x"] == [1, 2]
    assert t.inverse_structure[1]["y"][2] == 5


def test_point_struct(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    p = dg.Point(x=10, y=20)
    assert p.x == 10
    assert p.y == 20


def test_seq_of_points(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    p1 = dg.Point(x=1, y=2)
    p2 = dg.Point(x=3, y=4)
    s = dg.SeqOfPoints(points=[p1, p2])
    assert len(s.points) == 2
    assert s.points[0].x == 1
    assert s.points[1].y == 4


def test_map_of_points(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    origin = dg.Point(x=0, y=0)
    corner = dg.Point(x=100, y=100)
    m = dg.MapOfPoints(named_points={"origin": origin, "corner": corner})
    assert m.named_points["origin"].x == 0
    assert m.named_points["corner"].y == 100


def test_seq_of_seq_of_points(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    row1 = [dg.Point(x=0, y=0), dg.Point(x=1, y=0)]
    row2 = [dg.Point(x=0, y=1), dg.Point(x=1, y=1)]
    m = dg.SeqOfSeqOfPoints(point_matrix=[row1, row2])
    assert m.point_matrix[0][0].x == 0
    assert m.point_matrix[1][1].x == 1
    assert m.point_matrix[1][1].y == 1


def test_map_of_seq_of_points(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    line1 = [dg.Point(x=0, y=0), dg.Point(x=10, y=10)]
    line2 = [dg.Point(x=5, y=5)]
    m = dg.MapOfSeqOfPoints(point_lists={"line1": line1, "line2": line2})
    assert len(m.point_lists["line1"]) == 2
    assert m.point_lists["line1"][1].x == 10
    assert m.point_lists["line2"][0].x == 5


def test_typedef_aliases_exist(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    assert hasattr(dg, "IntList")
    assert hasattr(dg, "IntMatrix")
    assert hasattr(dg, "NamedMatrices")


def test_using_typedef_chain(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.UsingTypedefChain()
    assert t.data == {}
    t.data = {"matrix1": [[1, 2], [3, 4]], "matrix2": [[5]]}
    assert t.data["matrix1"][0][1] == 2
    assert t.data["matrix2"][0][0] == 5


def test_array_of_seq(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.ArrayOfSeq()
    assert t.items == []
    t.items = [[1, 2], [3], [4, 5, 6]]
    assert len(t.items) == 3
    assert t.items[2][2] == 6


def test_three_ints_typedef(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    assert hasattr(dg, "ThreeInts")


def test_seq_of_array(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.SeqOfArray()
    assert t.fixed_triples == []
    t.fixed_triples = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    assert len(t.fixed_triples) == 3
    assert t.fixed_triples[1] == [4, 5, 6]
    assert t.fixed_triples[2][2] == 9


def test_map_of_array(generated_modules: dict[str, ModuleType]) -> None:
    dg = generated_modules["deep_generic_types"]
    t = dg.MapOfArray()
    assert t.named_triples == {}
    t.named_triples = {"first": [1, 2, 3], "second": [10, 20, 30]}
    assert t.named_triples["first"] == [1, 2, 3]
    assert t.named_triples["second"][1] == 20
