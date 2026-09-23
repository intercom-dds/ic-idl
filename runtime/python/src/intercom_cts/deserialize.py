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

import typing
from abc import ABC, abstractmethod
from collections.abc import Iterable

from .type_info import K, MemberInfo, T, TypeInfo, TypeKind, V

__all__ = ["Deserializer", "Unmarshal", "UnmarshalField", "unmarshal"]

Unmarshal = typing.Callable[["Deserializer"], T]
UnmarshalField = tuple[MemberInfo, typing.Any, Unmarshal[typing.Any]]


class Deserializer(ABC):
    """Base class for implementing deserialization for a format."""

    @abstractmethod
    def decode_none(self) -> None:
        """Deserialize a `None` value."""

    @abstractmethod
    def decode_bool(self) -> bool:
        """Deserialize a `bool` value."""

    @abstractmethod
    def decode_char(self) -> str:
        """Deserialize a `char` value."""

    @abstractmethod
    def decode_wchar(self) -> str:
        """Deserialize a `wchar` value from a single-character `str`."""

    @abstractmethod
    def decode_i8(self) -> int:
        """Deserialize a signed 8-bit integer."""

    @abstractmethod
    def decode_u8(self) -> int:
        """Deserialize an unsigned 8-bit integer."""

    @abstractmethod
    def decode_i16(self) -> int:
        """Deserialize a signed 16-bit integer."""

    @abstractmethod
    def decode_u16(self) -> int:
        """Deserialize an unsigned 16-bit integer."""

    @abstractmethod
    def decode_i32(self) -> int:
        """Deserialize a signed 32-bit integer."""

    @abstractmethod
    def decode_u32(self) -> int:
        """Deserialize an unsigned 32-bit integer."""

    @abstractmethod
    def decode_i64(self) -> int:
        """Deserialize a signed 64-bit integer."""

    @abstractmethod
    def decode_u64(self) -> int:
        """Deserialize an unsigned 64-bit integer."""

    @abstractmethod
    def decode_f32(self) -> float:
        """Deserialize a 32-bit floating point."""

    @abstractmethod
    def decode_f64(self) -> float:
        """Deserialize a 64-bit floating point."""

    @abstractmethod
    def decode_string(self) -> str:
        """Deserialize a string."""

    @abstractmethod
    def decode_wstring(self) -> str:
        """Deserialize a wide string."""

    @abstractmethod
    def decode_struct(
        self,
        info: TypeInfo,
        fields: Iterable[tuple[MemberInfo, Unmarshal[typing.Any]]],
    ) -> typing.Any:
        """Deserialize a struct with fields."""

    @abstractmethod
    def decode_array(self, info: TypeInfo, elem_decode: Unmarshal[T]) -> list[T]:
        """Deserialize an array of elements."""

    @abstractmethod
    def decode_sequence(self, info: TypeInfo, elem_decode: Unmarshal[T]) -> list[T]:
        """Deserialize a sequence of elements."""

    @abstractmethod
    def decode_map(
        self,
        info: TypeInfo,
        key_decode: Unmarshal[K],
        elem_decode: Unmarshal[V],
    ) -> dict[K, V]:
        """Deserialize a map."""

    @abstractmethod
    def decode_union(
        self,
        info: TypeInfo,
        discriminator_decode: Unmarshal[K],
        variants: list[MemberInfo],
    ) -> typing.Any:
        """Deserialize a union."""

    @abstractmethod
    def decode_enum(
        self,
        info: TypeInfo,
        value_decode: Unmarshal[V],
        members: list[MemberInfo],
    ) -> typing.Any:
        """Deserialize an enumeration."""

    @abstractmethod
    def decode_bitmask(
        self,
        info: TypeInfo,
        value_decode: Unmarshal[V],
        members: list[MemberInfo],
    ) -> typing.Any:
        """Deserialize a bitmask."""


_unmarshaller_cache: dict[int, Unmarshal] = {}


def _unmarshaller_for(info: TypeInfo) -> Unmarshal:
    existing = _unmarshaller_cache.get(id(info))
    if existing is not None:
        return existing
    resolved: Unmarshal | None = None

    def recursive_unmarshaller(deserializer: Deserializer) -> typing.Any:
        if resolved is None:
            raise RuntimeError("Unmarshaller used before it was initialized")
        return resolved(deserializer)

    _unmarshaller_cache[id(info)] = recursive_unmarshaller

    if info.kind == TypeKind.NONE:
        resolved = lambda d: d.decode_none()
    elif info.kind == TypeKind.BOOL:
        resolved = lambda d: d.decode_bool()
    elif info.kind == TypeKind.I8:
        resolved = lambda d: d.decode_i8()
    elif info.kind == TypeKind.U8:
        resolved = lambda d: d.decode_u8()
    elif info.kind == TypeKind.I16:
        resolved = lambda d: d.decode_i16()
    elif info.kind == TypeKind.U16:
        resolved = lambda d: d.decode_u16()
    elif info.kind == TypeKind.I32:
        resolved = lambda d: d.decode_i32()
    elif info.kind == TypeKind.U32:
        resolved = lambda d: d.decode_u32()
    elif info.kind == TypeKind.I64:
        resolved = lambda d: d.decode_i64()
    elif info.kind == TypeKind.U64:
        resolved = lambda d: d.decode_u64()
    elif info.kind == TypeKind.F32:
        resolved = lambda d: d.decode_f32()
    elif info.kind == TypeKind.F64:
        resolved = lambda d: d.decode_f64()
    elif info.kind == TypeKind.CHAR8:
        resolved = lambda d: d.decode_char()
    elif info.kind == TypeKind.CHAR16:
        resolved = lambda d: d.decode_wchar()
    elif info.kind == TypeKind.STRING8:
        resolved = lambda d: d.decode_string()
    elif info.kind == TypeKind.STRING16:
        resolved = lambda d: d.decode_wstring()
    elif info.kind == TypeKind.ARRAY:
        elem_decode = _unmarshaller_for(info.element_info)
        resolved = lambda d: d.decode_array(info, elem_decode)
    elif info.kind == TypeKind.SEQUENCE:
        elem_decode = _unmarshaller_for(info.element_info)
        resolved = lambda d: d.decode_sequence(info, elem_decode)
    elif info.kind == TypeKind.MAP:
        key_decode = _unmarshaller_for(info.key_info)
        elem_decode = _unmarshaller_for(info.element_info)
        resolved = lambda d: d.decode_map(info, key_decode, elem_decode)
    elif info.kind == TypeKind.STRUCT:
        members: list[MemberInfo] = info.ty.__member_info__()
        field_decoders = [(m, _unmarshaller_for(m.type_info)) for m in members]

        def _struct(d: Deserializer):
            return d.decode_struct(info, field_decoders)

        resolved = _struct
    elif info.kind == TypeKind.ENUM:
        members: list[MemberInfo] = info.ty.__member_info__()
        key_decode = _unmarshaller_for(info.element_info)

        def _enum(d: Deserializer):
            return d.decode_enum(info, key_decode, members)

        resolved = _enum
    elif info.kind == TypeKind.BITMASK:
        members: list[MemberInfo] = info.ty.__member_info__()
        key_decode = _unmarshaller_for(info.element_info)
        resolved = lambda d: d.decode_bitmask(info, key_decode, members)
    elif info.kind == TypeKind.UNION:
        members: list[MemberInfo] = info.ty.__member_info__()
        discriminator_decode = _unmarshaller_for(info.key_info)

        def _union(d: Deserializer):
            return d.decode_union(
                info,
                discriminator_decode,
                members,
            )

        resolved = _union
    else:
        raise TypeError(f"Unsupported unmarshal of TypeKind {info.kind}")

    _unmarshaller_cache[id(info)] = resolved
    return resolved


def unmarshal(deserializer: Deserializer, info: TypeInfo) -> typing.Any:
    """Deserialize a value which type implements TypeDescriptor on the deserializer."""

    return _unmarshaller_for(info)(deserializer)
