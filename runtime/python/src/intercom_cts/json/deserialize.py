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
from collections.abc import Iterable
from dataclasses import dataclass
from enum import Flag

from intercom_cts.deserialize import Deserializer, Unmarshal, unmarshal
from intercom_cts.type_info import K, MemberFlag, MemberInfo, T, TypeInfo, V

from .value import JsonValue


@dataclass(frozen=True, slots=True)
class JsonDeserializer(Deserializer):
    value: JsonValue

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
        if not isinstance(self.value, str):
            raise TypeError(f"Char must be a str, got: '{self.value}'")
        if len(self.value) != 1:
            raise TypeError(
                f"Char must not contain multiple chracters, got: '{self.value}'"
            )

        return self.value

    def decode_wchar(self) -> str:
        return self.decode_char()

    def decode_i8(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_u8(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_i16(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_u16(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_i32(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_u32(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_i64(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_u64(self) -> int:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return int(self.value)

    def decode_f32(self) -> float:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return float(self.value)

    def decode_f64(self) -> float:
        if not isinstance(self.value, (int, str)):
            raise TypeError(f"Expected integral or numeric string, got: '{self.value}")

        return float(self.value)

    def decode_string(self) -> str:
        if isinstance(self.value, str):
            return self.value
        raise TypeError(f"Expected string, got {type(self.value).__name__}")

    def decode_wstring(self) -> str:
        return self.decode_string()

    def decode_struct(
        self,
        info: TypeInfo,
        fields: Iterable[tuple[MemberInfo, Unmarshal[JsonValue]]],
    ) -> object:
        if not isinstance(self.value, dict):
            raise TypeError(f"Expected dict, got: '{type(self.value).__name__}")

        instance = info.ty()

        for member_info, member_unmarshal in fields:
            member_value = self.value.get(member_info.name)
            if member_value != None:
                member_info.descriptor.__set__(
                    instance, member_unmarshal(JsonDeserializer(member_value))
                )
            elif member_info.flags & MemberFlag.IS_OPTIONAL:
                member_info.descriptor.__set__(instance, None)

        return instance

    def decode_array(self, info: TypeInfo, elem_decode: Unmarshal[T]) -> list[T]:
        if not isinstance(self.value, list):
            raise TypeError(f"Expected array, got {type(self.value).__name__}")

        return [elem_decode(JsonDeserializer(item)) for item in self.value]

    def decode_sequence(self, info: TypeInfo, elem_decode: Unmarshal[T]) -> list[T]:
        return self.decode_array(info, elem_decode)

    def decode_map(
        self,
        info: TypeInfo,
        key_decode: Unmarshal[K],
        elem_decode: Unmarshal[V],
    ) -> dict[K, V]:
        if not isinstance(self.value, dict):
            raise TypeError(f"Expected map object, got {type(self.value).__name__}")

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
            (
                variant
                for variant in variants
                if variant.key and discriminator in variant.key
            ),
            None,
        )

        if variant is None:
            default_variant = next(
                (member for member in variants if not member.key), None
            )
            if default_variant is None:
                raise ValueError(
                    f"Expected union variant, got {list(self.value.keys())}"
                )
        else:
            instance = info.ty()
            decoded_value = unmarshal(
                JsonDeserializer(self.value[variant.name]), variant.type_info
            )
            variant.descriptor.__set__(instance, decoded_value)

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

        if isinstance(self.value, int):
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
            return info.ty(int(self.value))

        raise TypeError(
            f"Expected bitmask string or numeric value, got {type(self.value).__name__}"
        )


def from_value(value: JsonValue, info: TypeInfo) -> object:
    deserializer = JsonDeserializer(value)
    return unmarshal(deserializer, info)


def from_str(text: str, info: TypeInfo) -> object:
    return from_value(json.loads(text), info)
