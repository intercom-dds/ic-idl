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
import typing
from collections.abc import Iterable
from dataclasses import dataclass
from decimal import Decimal
from enum import Flag

from intercom_cts.deserialize import Deserializer, Unmarshal, unmarshal
from intercom_cts.type_info import (
    BuiltinTypes,
    K,
    MemberFlag,
    MemberInfo,
    T,
    TypeInfo,
    V,
)

from .value import JsonValue

N = typing.TypeVar("N", int, float, Decimal)


def _check_char(value: JsonValue, type_info: TypeInfo[str]) -> str:
    if not isinstance(value, str):
        raise TypeError(f"Char must be a str, got: '{value}'")

    if len(value) != 1:
        raise ValueError(f"{type_info.name} must be a single character, got: '{value}'")

    if type_info.max is not None and ord(value) > ord(type_info.max):
        raise ValueError(f"Character '{value}' is out of range for {type_info.name}")
    return value


def _check_numeric(value: JsonValue, type_info: TypeInfo[N]) -> N:
    if (
        value is None
        or isinstance(value, bool)
        or not isinstance(value, (str, type_info.ty))
    ):
        raise TypeError(
            f"Expected numeric type {type_info.ty.__name__}, got: {type(value).__name__}"
        )

    value: N = type_info.ty(value) if isinstance(value, str) else value

    if (
        type_info.max is not None
        and value > type_info.max
        or type_info.min is not None
        and value < type_info.min
    ):
        raise ValueError(f"Value {value} is out of range for {type_info.name}")
    return value


@dataclass(frozen=True, slots=True)
class JsonDeserializer(Deserializer):
    value: JsonValue

    def decode_any(self) -> typing.Any:
        return self.value

    def decode_none(self) -> None:
        return None

    def decode_bool(self) -> bool:
        if isinstance(self.value, bool):
            return self.value
        if isinstance(self.value, str):
            lower = self.value.lower()
            if lower == "true":
                return True
            if lower == "false":
                return False
        raise TypeError(f"Expected bool, got {type(self.value).__name__}")

    def decode_char(self) -> str:
        return _check_char(self.value, BuiltinTypes.CHAR8)

    def decode_wchar(self) -> str:
        return _check_char(self.value, BuiltinTypes.CHAR16)

    def decode_i8(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.I8)

    def decode_u8(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.U8)

    def decode_i16(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.I16)

    def decode_u16(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.U16)

    def decode_i32(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.I32)

    def decode_u32(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.U32)

    def decode_i64(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.I64)

    def decode_u64(self) -> int:
        return _check_numeric(self.value, BuiltinTypes.U64)

    def decode_f32(self) -> float:
        return _check_numeric(self.value, BuiltinTypes.F32)

    def decode_f64(self) -> float:
        return _check_numeric(self.value, BuiltinTypes.F64)

    def decode_f128(self) -> Decimal:
        return _check_numeric(self.value, BuiltinTypes.F128)

    def decode_string(self, bound: int) -> str:
        if not isinstance(self.value, str):
            raise TypeError(f"Expected string, got {type(self.value).__name__}")

        if bound != 0 and len(self.value) > bound:
            raise ValueError(
                f"Expected string length not exceed bound {bound}, got {len(self.value)}"
            )

        return self.value

    def decode_wstring(self, bound: int) -> str:
        return self.decode_string(bound)

    def decode_struct(
        self,
        info: TypeInfo,
        fields: Iterable[tuple[MemberInfo, Unmarshal[JsonValue]]],
    ) -> object:
        if not isinstance(self.value, dict):
            raise TypeError(f"Expected dict, got: '{type(self.value).__name__}")

        instance = info.ty()

        for member_info, member_unmarshal in fields:
            try:
                member_value = self.value.get(member_info.name)
                if member_value != None:
                    member_info.descriptor.__set__(
                        instance, member_unmarshal(JsonDeserializer(member_value))
                    )
                elif MemberFlag.IS_OPTIONAL in member_info.flags:
                    member_info.descriptor.__set__(instance, None)
            except Exception as e:
                e.add_note(f'Error decoding member "{member_info.name}"')
                raise

        return instance

    def decode_array(
        self, info: TypeInfo, elem_decode: Unmarshal[T], length: int
    ) -> list[T]:
        if not isinstance(self.value, list):
            raise TypeError(f"Expected array, got {type(self.value).__name__}")

        if len(self.value) != length:
            raise ValueError(
                f"Expected array length to be {length}, got {len(self.value)}"
            )

        return [elem_decode(JsonDeserializer(item)) for item in self.value]

    def decode_sequence(
        self, info: TypeInfo, elem_decode: Unmarshal[T], bound: int
    ) -> list[T]:
        if not isinstance(self.value, list):
            raise TypeError(f"Expected array, got {type(self.value).__name__}")

        if bound != 0 and len(self.value) > bound:
            raise ValueError(
                f"Expected sequence length not exceed bound {bound}, got {len(self.value)}"
            )

        return [elem_decode(JsonDeserializer(item)) for item in self.value]

    def decode_map(
        self,
        info: TypeInfo,
        key_decode: Unmarshal[K],
        elem_decode: Unmarshal[V],
        bound: int,
    ) -> dict[K, V]:
        if not isinstance(self.value, dict):
            raise TypeError(f"Expected map object, got {type(self.value).__name__}")

        if bound != 0 and len(self.value) > bound:
            raise ValueError(
                f"Expected map length not exceed bound {bound}, got {len(self.value)}"
            )

        out: dict[K, V] = {}
        for key, item in self.value.items():
            decoded_key = key_decode(JsonDeserializer(key))
            decoded_value = elem_decode(JsonDeserializer(value=item))
            out[decoded_key] = decoded_value
        return out

    def decode_union(
        self,
        info: TypeInfo,
        discriminator_decode: Unmarshal[K],
        variants: list[MemberInfo],
    ) -> object:
        if not isinstance(self.value, dict):
            raise TypeError(f"Expected union object, got {type(self.value).__name__}")

        discriminator_value = self.value.get("$discriminator")
        if discriminator_value is None:
            raise TypeError("Union object is missing '$discriminator'")
        discriminator = discriminator_decode(JsonDeserializer(discriminator_value))

        variant = next(
            (v for v in variants if v.key and discriminator in v.key),
            next((v for v in variants if not v.key), None),
        )
        if variant is None:
            raise ValueError(f"No union variant for discriminator {discriminator}")

        instance = info.ty()
        json_value = self.value.get(variant.name)
        value = (
            None
            if json_value is None and MemberFlag.IS_OPTIONAL in variant.flags
            else unmarshal(JsonDeserializer(json_value), variant.type_info)
        )

        variant.descriptor.__set__(instance, value)
        instance._discriminator = discriminator

        return instance

    def decode_enum(
        self,
        info: TypeInfo,
        value_decode: Unmarshal[V],
        members: list[MemberInfo],
    ) -> object:
        if isinstance(self.value, str):
            for member in members:
                if member.name == self.value:
                    return member.key
            raise ValueError(f"Unknown enum value '{self.value}'")

        if isinstance(self.value, int) and not isinstance(self.value, bool):
            return info.ty(self.value)

        raise TypeError(
            f"Expected enum string or numeric value, got {type(self.value).__name__}"
        )

    def decode_bitmask(
        self,
        info: TypeInfo,
        value_decode: Unmarshal[V],
        members: list[MemberInfo],
    ) -> Flag:
        if isinstance(self.value, str):
            value = self.value.strip()
            if value == "":
                return info.ty(0)

            flags = info.ty(0)
            for flag_name in value.split("|"):
                flag_name = flag_name.strip()
                if not flag_name:
                    continue
                member = next((m for m in members if m.name == flag_name), None)
                if member is None:
                    raise ValueError(f"Unknown bitmask flag '{flag_name}'")
                flags |= member.key
            return flags

        if isinstance(self.value, int):
            return info.ty(value_decode(self))

        raise TypeError(
            f"Expected bitmask string or numeric value, got {type(self.value).__name__}"
        )


def from_value(value: JsonValue, info: TypeInfo) -> object:
    deserializer = JsonDeserializer(value)
    return unmarshal(deserializer, info)


def from_str(text: str, info: TypeInfo) -> object:
    return from_value(json.loads(text), info)
