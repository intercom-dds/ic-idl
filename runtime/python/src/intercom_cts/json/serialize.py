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
import math
import typing
from collections.abc import Iterable
from decimal import Decimal

from intercom_cts.serialize import Marshal, MarshalField, Serializer, marshal
from intercom_cts.type_info import (
    BuiltinTypes,
    K,
    MemberFlag,
    MemberInfo,
    T,
    TypeDescriptor,
    TypeInfo,
    V,
)

from .value import JsonValue

_MAX_SAFE_INTEGER = (1 << 53) - 1

N = typing.TypeVar("N", int, float, Decimal)


def _check_numeric(value: N, type_info: TypeInfo[N]) -> N:
    if not isinstance(value, type_info.ty) or isinstance(value, bool):
        raise TypeError(
            f"Expected numeric type {type_info.ty.__name__}, got: {type(value).__name__}"
        )

    if (
        type_info.max is not None
        and value > type_info.max
        or type_info.min is not None
        and value < type_info.min
    ):
        raise ValueError(f"Value {value} is out of range for {type_info.name}")
    return value


def _check_char(value: str, type_info: TypeInfo[str]) -> str:
    if not isinstance(value, str):
        raise TypeError(f"Expected str type, got: {type(value).__name__}")

    if len(value) != 1:
        raise ValueError(f"{type_info.name} must be a single character, got: '{value}'")

    if type_info.max is not None and ord(value) > ord(type_info.max):
        raise ValueError(f"Character '{value}' is out of range for {type_info.name}")
    return value


class JsonSerializer(Serializer):
    def __init__(self, large_integers_as_strings: bool = False):
        self.large_integers_as_strings = large_integers_as_strings

    def encode_any(self, value: typing.Any) -> JsonValue:
        return value

    def encode_none(self, value: None) -> JsonValue:
        return None

    def encode_bool(self, value: bool) -> JsonValue:
        return bool(value)

    def encode_char(self, value: str) -> JsonValue:
        return _check_char(value, BuiltinTypes.CHAR8)

    def encode_wchar(self, value: str) -> JsonValue:
        return _check_char(value, BuiltinTypes.CHAR16)

    def encode_i8(self, value: int) -> JsonValue:
        return _check_numeric(value, BuiltinTypes.I8)

    def encode_u8(self, value: int) -> JsonValue:
        return _check_numeric(value, BuiltinTypes.U8)

    def encode_i16(self, value: int) -> JsonValue:
        return _check_numeric(value, BuiltinTypes.I16)

    def encode_u16(self, value: int) -> JsonValue:
        return _check_numeric(value, BuiltinTypes.U16)

    def encode_i32(self, value: int) -> JsonValue:
        return _check_numeric(value, BuiltinTypes.I32)

    def encode_u32(self, value: int) -> JsonValue:
        return _check_numeric(value, BuiltinTypes.U32)

    def encode_i64(self, value: int) -> JsonValue:
        _check_numeric(value, BuiltinTypes.I64)
        if self.large_integers_as_strings and (
            value > _MAX_SAFE_INTEGER or value < -_MAX_SAFE_INTEGER
        ):
            return str(value)
        else:
            return value

    def encode_u64(self, value: int) -> JsonValue:
        _check_numeric(value, BuiltinTypes.U64)
        if self.large_integers_as_strings and value > _MAX_SAFE_INTEGER:
            return str(value)
        else:
            return value

    def encode_f32(self, value: float) -> JsonValue:
        if not math.isfinite(value):
            return None

        _check_numeric(value, BuiltinTypes.F32)

        return value

    def encode_f64(self, value: float) -> JsonValue:
        if not math.isfinite(value):
            return None

        _check_numeric(value, BuiltinTypes.F64)

        return value

    def encode_f128(self, value: Decimal) -> JsonValue:
        if not value.is_finite():
            return None

        _check_numeric(value, BuiltinTypes.F128)

        return str(value)

    def encode_string(self, value: str, bound: int) -> JsonValue:
        if not isinstance(value, str):
            raise TypeError(f"Expected string type, got {type(value).__name__}")

        if bound != 0 and len(value) > bound:
            raise ValueError(
                f"String length exceeds bound {bound}, length {len(value)}"
            )

        return value

    def encode_wstring(self, value: str, bound: int) -> JsonValue:
        return self.encode_string(value, bound)

    def encode_array(
        self, values: list[T], elem_marshal: Marshal[T], length: int
    ) -> JsonValue:
        if not isinstance(values, list):
            raise TypeError(f"Expected list type, got {type(values).__name__}")

        if len(values) != length:
            raise ValueError(f"Expected array length to be {length}, got {len(values)}")

        return [elem_marshal(self, value) for value in values]

    def encode_sequence(
        self, values: list[T], elem_marshal: Marshal[T], bound: int
    ) -> JsonValue:
        if not isinstance(values, list):
            raise TypeError(f"Expected list type, got {type(values).__name__}")

        if bound != 0 and len(values) > bound:
            raise ValueError(
                f"Sequence length exceeds bound {bound}, length {len(values)}"
            )

        return [elem_marshal(self, value) for value in values]

    def encode_map(
        self,
        map: dict[K, V],
        key_marshal: Marshal[K],
        elem_marshal: Marshal[V],
        bound: int,
    ) -> JsonValue:
        obj = {}

        if not isinstance(map, dict):
            raise TypeError(f"Expected dict type, got {type(map).__name__}")

        if bound != 0 and len(map) > bound:
            raise ValueError(f"Map length exceeds bound {bound}, length {len(map)}")

        for key, value in map.items():
            obj[
                str(key_marshal(self, key)).lower()
                if isinstance(key, bool)
                else str(key_marshal(self, key))
            ] = elem_marshal(self, value)

        return obj

    def encode_union(
        self,
        info: TypeInfo,
        discriminator: K,
        discriminator_marshal: Marshal[K],
        variant: V,
        variant_marshal: Marshal[V],
        variant_info: MemberInfo,
    ) -> JsonValue:
        return {
            "$discriminator": discriminator_marshal(self, discriminator),
            variant_info.name: None
            if variant is None and MemberFlag.IS_OPTIONAL in variant_info.flags
            else variant_marshal(self, variant),
        }

    def encode_enum(
        self,
        info: TypeInfo,
        value: V,
        value_marshal: Marshal[V],
        value_info: MemberInfo,
    ) -> JsonValue:
        return value_info.name

    def encode_bitmask(
        self,
        info: TypeInfo,
        value: V,
        value_marshal: Marshal[V],
        members: list[MemberInfo],
    ) -> JsonValue:
        return "|".join([m.name for m in members if bool(m.key.value & value)])

    def encode_struct(
        self, info: TypeInfo, fields: Iterable[MarshalField]
    ) -> JsonValue:
        obj = {}
        for member_info, value, value_marshal in fields:
            try:
                if value is None:
                    if MemberFlag.IS_OPTIONAL in member_info.flags:
                        obj[member_info.name] = None
                    else:
                        raise ValueError("Got a None value for a non-optional member")
                else:
                    obj[member_info.name] = value_marshal(self, value)
            except Exception as e:
                e.add_note(f'Error encoding member "{member_info.name}"')
                raise

        return obj


def to_value(
    value: TypeDescriptor, large_integers_as_strings: bool = False
) -> JsonValue:
    serializer = JsonSerializer(large_integers_as_strings)
    return marshal(serializer, value)


def to_str(
    value: TypeDescriptor, pretty: bool = False, large_integers_as_strings: bool = False
) -> str:
    return json.dumps(
        to_value(value, large_integers_as_strings), indent=4 if pretty else None
    )
