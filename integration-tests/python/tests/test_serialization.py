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

import json
from types import ModuleType

from intercom_cts.json import JsonSerializer, from_str, to_str, to_value
from intercom_cts.type_info import type_info

MAX_SAFE_INTEGER = (1 << 53) - 1


def test_serilization_json_primitives(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    p = dt.PrimitiveDefaults()
    json_str = to_str(p)
    data = from_str(json_str, type_info(dt.PrimitiveDefaults))
    assert data == p


def test_serilization_json_array(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    a = dt.ArrayDefaults()
    json_str = to_str(a)
    data = from_str(json_str, type_info(dt.ArrayDefaults))
    assert data == a


def test_serilization_json_sequence(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    s = dt.SequenceDefaults()
    json_str = to_str(s)
    data = from_str(json_str, type_info(dt.SequenceDefaults))
    assert data == s


def test_serilization_json_map(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    m = dt.MapDefaults()
    json_str = to_str(m)
    data = from_str(json_str, type_info(dt.MapDefaults))
    assert data == m


def test_serilization_json_enum(generated_modules: dict[str, ModuleType]) -> None:
    dt = generated_modules["default_types"]
    e = dt.EnumDefaults()
    json_str = to_str(e)
    data = from_str(json_str, type_info(dt.EnumDefaults))
    assert data == e


def test_serilization_json_bitmask(generated_modules: dict[str, ModuleType]) -> None:
    bt = generated_modules["bitmask_types"]

    p = bt.Permissions.none()
    json_str = to_str(p)
    data = from_str(json_str, type_info(bt.Permissions))
    assert data == p

    p = bt.Permissions.all()
    json_str = to_str(p)
    data = from_str(json_str, type_info(bt.Permissions))
    assert data == p


def test_serilization_json_union(generated_modules: dict[str, ModuleType]) -> None:
    ut = generated_modules["union_types"]
    u = ut.IntOrString()

    u.int_val = 42
    json_str = to_str(u)
    data = from_str(json_str, type_info(ut.IntOrString))
    assert data == u

    u.str_val = "hello"
    json_str = to_str(u)
    data = from_str(json_str, type_info(ut.IntOrString))
    assert data == u


def test_serilization_json_optional_struct_member(
    generated_modules: dict[str, ModuleType],
) -> None:
    at = generated_modules["annotation_types"]

    ot = at.OptionalTypes()
    json_str = to_str(ot)
    data = from_str(json_str, type_info(at.OptionalTypes))
    assert data == ot

    ot = at.OptionalTypes()
    ot.maybe_char = "A"
    json_str = to_str(ot)
    data = from_str(json_str, type_info(at.OptionalTypes))
    assert data == ot


def test_serilization_json_optional_union_variant(
    generated_modules: dict[str, ModuleType],
) -> None:
    at = generated_modules["annotation_types"]
    u = at.OptionalUnion()

    u.number = 123
    json_str = to_str(u)
    data = from_str(json_str, type_info(at.OptionalUnion))
    assert data == u

    u.number = None
    json_str = to_str(u)
    data = from_str(json_str, type_info(at.OptionalUnion))
    assert data == u


def test_serilization_json_circular_type(
    generated_modules: dict[str, ModuleType],
) -> None:
    ct = generated_modules["circular_types"]
    p = ct.TreeNode()
    json_str = to_str(p)
    data = from_str(json_str, type_info(ct.TreeNode))
    assert data == p


def test_serialization_json_numbers_large_integers() -> None:
    safe = MAX_SAFE_INTEGER
    unsafe = MAX_SAFE_INTEGER + 1

    def encode(value: int, *, large_integers_as_strings: bool = False) -> str:
        serializer = JsonSerializer(large_integers_as_strings=large_integers_as_strings)
        return json.dumps(serializer.encode_i64(value))

    assert '"' not in encode(safe)
    assert '"' not in encode(unsafe)

    assert '"' not in encode(safe, large_integers_as_strings=True)
    assert '"' in encode(unsafe, large_integers_as_strings=True)


def test_serialization_json_numbers_large_integers_structure(
    generated_modules: dict[str, ModuleType],
) -> None:
    serialization = generated_modules["serialization_types"]
    safe = serialization.JsonSafeNumbers(
        a=MAX_SAFE_INTEGER,
        b=MAX_SAFE_INTEGER,
        c=-MAX_SAFE_INTEGER,
        d={MAX_SAFE_INTEGER: -MAX_SAFE_INTEGER},
    )
    unsafe = serialization.JsonSafeNumbers(
        a=MAX_SAFE_INTEGER + 1,
        b=MAX_SAFE_INTEGER + 1,
        c=-MAX_SAFE_INTEGER - 1,
        d={MAX_SAFE_INTEGER + 1: -MAX_SAFE_INTEGER - 1},
    )

    assert to_value(safe) == {
        "a": 9007199254740991,
        "b": 9007199254740991,
        "c": -9007199254740991,
        "d": {"9007199254740991": -9007199254740991},
    }
    assert to_value(unsafe) == {
        "a": 9007199254740992,
        "b": 9007199254740992,
        "c": -9007199254740992,
        "d": {"9007199254740992": -9007199254740992},
    }

    assert to_value(safe, large_integers_as_strings=True) == {
        "a": 9007199254740991,
        "b": 9007199254740991,
        "c": -9007199254740991,
        "d": {"9007199254740991": -9007199254740991},
    }
    assert to_value(unsafe, large_integers_as_strings=True) == {
        "a": "9007199254740992",
        "b": "9007199254740992",
        "c": "-9007199254740992",
        "d": {"9007199254740992": "-9007199254740992"},
    }
