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

from intercom_cts.serialize import Marshal, MarshalField, Serializer, marshal
from intercom_cts.type_info import K, MemberInfo, T, TypeDescriptor, TypeInfo, V

from .value import JsonValue


class JsonSerializer(Serializer):
    def encode_none(self, value: None) -> JsonValue:
        return None

    def encode_bool(self, value: bool) -> JsonValue:
        return value

    def encode_char(self, value: str) -> JsonValue:
        return value

    def encode_wchar(self, value: str) -> JsonValue:
        return value

    def encode_i8(self, value: int) -> JsonValue:
        return value

    def encode_u8(self, value: int) -> JsonValue:
        return value

    def encode_i16(self, value: int) -> JsonValue:
        return value

    def encode_u16(self, value: int) -> JsonValue:
        return value

    def encode_i32(self, value: int) -> JsonValue:
        return value

    def encode_u32(self, value: int) -> JsonValue:
        return value

    def encode_i64(self, value: int) -> JsonValue:
        return value

    def encode_u64(self, value: int) -> JsonValue:
        return value

    def encode_f32(self, value: float) -> JsonValue:
        return self.encode_f64(value)

    def encode_f64(self, value: float) -> JsonValue:
        return value

    def encode_string(self, value: str) -> JsonValue:
        return value

    def encode_wstring(self, value: str) -> JsonValue:
        return value

    def encode_array(self, values: list[T], elem_marshal: Marshal[T]) -> JsonValue:
        return [elem_marshal(self, value) for value in values]

    def encode_sequence(self, values: list[T], elem_marshal: Marshal[T]) -> JsonValue:
        return [elem_marshal(self, value) for value in values]

    def encode_map(
        self, dict: dict[K, V], key_marshal: Marshal[K], elem_marshal: Marshal[V]
    ) -> JsonValue:
        obj = {}

        for key, value in dict.items():
            obj[key_marshal(self, key)] = elem_marshal(self, value)

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
            variant_info.name: variant_marshal(self, variant),
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
            if value is None:
                obj[member_info.name] = self.encode_none(None)
            else:
                obj[member_info.name] = value_marshal(self, value)
        return obj


def to_value(value: TypeDescriptor) -> JsonValue:
    serializer = JsonSerializer()
    return marshal(serializer, value)


def to_str(value: TypeDescriptor, pretty=False) -> str:
    return json.dumps(to_value(value), indent=4 if pretty else None)
