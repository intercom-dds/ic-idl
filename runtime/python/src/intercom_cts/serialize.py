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
from enum import Flag

from .type_info import K, MemberInfo, T, TypeDescriptor, TypeInfo, TypeKind, V

__all__ = ["Marshal", "MarshalField", "Serializer", "marshal"]

Marshal = typing.Callable[["Serializer", T], typing.Any]
MarshalField = tuple[MemberInfo, typing.Any, Marshal[typing.Any]]


class Serializer(ABC):
    """Base class for implementing serialization for a format."""

    def encode_none(self, value: None) -> typing.Any:
        """Serialize a `None` value"""

    @abstractmethod
    def encode_bool(self, value: bool) -> typing.Any:
        """Serialize a `bool` value."""

    @abstractmethod
    def encode_char(self, value: str) -> typing.Any:
        """Serialize a `char` value."""

    @abstractmethod
    def encode_wchar(self, value: str) -> typing.Any:
        """Serialize a `wchar` value from a single character `str`."""

    @abstractmethod
    def encode_i8(self, value: int) -> typing.Any:
        """Serialize a signed 8-bit integer."""

    @abstractmethod
    def encode_u8(self, value: int) -> typing.Any:
        """Serialize a unsigned 8-bit integer."""

    @abstractmethod
    def encode_i16(self, value: int) -> typing.Any:
        """Serialize a signed 16-bit integer."""

    @abstractmethod
    def encode_u16(self, value: int) -> typing.Any:
        """Serialize a unsigned 16-bit integer."""

    @abstractmethod
    def encode_i32(self, value: int) -> typing.Any:
        """Serialize a signed 32-bit integer."""

    @abstractmethod
    def encode_u32(self, value: int) -> typing.Any:
        """Serialize a unsigned 32-bit integer."""

    @abstractmethod
    def encode_i64(self, value: int) -> typing.Any:
        """Serialize a signed 64-bit integer."""

    @abstractmethod
    def encode_u64(self, value: int) -> typing.Any:
        """Serialize a unsigned 64-bit integer."""

    @abstractmethod
    def encode_f32(self, value: float) -> typing.Any:
        """Serialize a 32-bit floating point."""

    @abstractmethod
    def encode_f64(self, value: float) -> typing.Any:
        """Serialize a 64-bit floating point."""

    @abstractmethod
    def encode_string(self, value: str) -> typing.Any:
        """Serialize a string."""

    @abstractmethod
    def encode_wstring(self, value: str) -> typing.Any:
        """Serialize as a wide string."""

    @abstractmethod
    def encode_struct(
        self, info: TypeInfo, fields: Iterable[MarshalField]
    ) -> typing.Any:
        """Serialize a struct with fields."""

    @abstractmethod
    def encode_array(self, values: list[T], elem_marshal: Marshal[T]) -> typing.Any:
        """Serialize an array of elements."""

    @abstractmethod
    def encode_sequence(self, values: list[T], elem_marshal: Marshal[T]) -> typing.Any:
        """Serialize a sequence of elements."""

    @abstractmethod
    def encode_map(
        self, dict: dict[K, V], key_marshal: Marshal[K], elem_marshal: Marshal[V]
    ) -> typing.Any:
        """Serialize a map."""

    @abstractmethod
    def encode_union(
        self,
        info: TypeInfo,
        discriminator: K,
        discriminator_marshal: Marshal[K],
        variant: V,
        variant_marshal: Marshal[V],
        variant_info: MemberInfo,
    ) -> typing.Any:
        """Serialize an union."""

    @abstractmethod
    def encode_enum(
        self,
        info: TypeInfo,
        value: V,
        value_marshal: Marshal[V],
        value_info: MemberInfo,
    ) -> typing.Any:
        """Serialize an enumeration."""

    @abstractmethod
    def encode_bitmask(
        self,
        info: TypeInfo,
        value: V,
        value_marshal: Marshal[V],
        members: list[MemberInfo],
    ) -> typing.Any:
        """Serialize a bitmask."""


_marshaller_cache: dict[int, Marshal] = {}


def _marshaller_for(info: TypeInfo) -> Marshal:
    existing = _marshaller_cache.get(id(info))
    if existing is not None:
        return existing
    resolved: Marshal | None = None

    def recursive_marshaller(serializer: Serializer, value: typing.Any) -> Marshal:
        if resolved is None:
            raise RuntimeError("Marshaller used before it was initialized")
        return resolved(serializer, value)

    _marshaller_cache[id(info)] = recursive_marshaller

    if info.kind == TypeKind.NONE:
        resolved = lambda s, v: s.encode_none(v)
    elif info.kind == TypeKind.BOOL:
        resolved = lambda s, v: s.encode_bool(v)
    elif info.kind == TypeKind.I8:
        resolved = lambda s, v: s.encode_i8(v)
    elif info.kind == TypeKind.U8:
        resolved = lambda s, v: s.encode_u8(v)
    elif info.kind == TypeKind.I16:
        resolved = lambda s, v: s.encode_i16(v)
    elif info.kind == TypeKind.U16:
        resolved = lambda s, v: s.encode_u16(v)
    elif info.kind == TypeKind.I32:
        resolved = lambda s, v: s.encode_i32(v)
    elif info.kind == TypeKind.U32:
        resolved = lambda s, v: s.encode_u32(v)
    elif info.kind == TypeKind.I64:
        resolved = lambda s, v: s.encode_i64(v)
    elif info.kind == TypeKind.U64:
        resolved = lambda s, v: s.encode_u64(v)
    elif info.kind == TypeKind.F32:
        resolved = lambda s, v: s.encode_f32(v)
    elif info.kind == TypeKind.F64:
        resolved = lambda s, v: s.encode_f64(v)
    elif info.kind == TypeKind.CHAR8:
        resolved = lambda s, v: s.encode_char(v)
    elif info.kind == TypeKind.CHAR16:
        resolved = lambda s, v: s.encode_wchar(v)
    elif info.kind == TypeKind.STRING8:
        resolved = lambda s, v: s.encode_string(v)
    elif info.kind == TypeKind.STRING16:
        resolved = lambda s, v: s.encode_wstring(v)
    elif info.kind == TypeKind.ARRAY:
        elem_marshal = _marshaller_for(info.element_info)
        resolved = lambda s, v: s.encode_array(v, elem_marshal)
    elif info.kind == TypeKind.SEQUENCE:
        elem_marshal = _marshaller_for(info.element_info)
        resolved = lambda s, v: s.encode_sequence(v, elem_marshal)
    elif info.kind == TypeKind.MAP:
        key_marshal = _marshaller_for(info.key_info)
        elem_marshal = _marshaller_for(info.element_info)
        resolved = lambda s, v: s.encode_map(v, key_marshal, elem_marshal)
    elif info.kind == TypeKind.STRUCT:
        members: list[MemberInfo] = info.ty.__member_info__()
        fields = [
            (member, member.descriptor.__get__, _marshaller_for(member.type_info))
            for member in members
        ]

        def _struct(s: Serializer, v: TypeDescriptor):
            return s.encode_struct(
                info,
                [(f, get(v), marshaller) for (f, get, marshaller) in fields],
            )

        resolved = _struct
    elif info.kind == TypeKind.ENUM:
        members: list[MemberInfo] = info.ty.__member_info__()
        members_by_value = {member.key: member for member in members}
        elem_marshal = _marshaller_for(info.element_info)
        resolved = lambda s, v: s.encode_enum(
            info, v.value, elem_marshal, members_by_value[v]
        )
    elif info.kind == TypeKind.BITMASK:
        members: list[MemberInfo] = info.ty.__member_info__()
        elem_marshal = _marshaller_for(info.element_info)

        def _bitmask(s: Serializer, v: Flag):
            return s.encode_bitmask(info, v.value, elem_marshal, members)

        resolved = _bitmask
    elif info.kind == TypeKind.UNION:
        discriminator_marshal = _marshaller_for(info.key_info)
        members: list[MemberInfo] = info.ty.__member_info__()
        variants: dict[typing.Any, tuple[MemberInfo, typing.Any, Marshal]] = {}
        default_variant: tuple[MemberInfo, typing.Any, Marshal] | None = None
        for member in members:
            variant = (
                member,
                member.descriptor.__get__,
                _marshaller_for(member.type_info),
            )
            if member.key:
                for discriminator in member.key:
                    variants[discriminator] = variant
            elif default_variant is None:
                default_variant = variant

        def _union(s: Serializer, v):
            variant = variants.get(v.discriminator, default_variant)

            if variant is None:
                raise TypeError(f"Unresolvable variant for Union {v}")

            variant_member, variant_get, variant_marshal = variant
            return s.encode_union(
                info,
                v.discriminator,
                discriminator_marshal,
                variant_get(v),
                variant_marshal,
                variant_member,
            )

        resolved = _union
    else:
        raise TypeError(f"Unsupported marshal of TypeKind {info.kind}")

    _marshaller_cache[id(info)] = resolved
    return resolved


_marshaller_by_ty: dict[type, Marshal] = {}


def marshal(serializer: Serializer, value: TypeDescriptor) -> typing.Any:
    """Marshal a value which type implements TypeDescriptor on the serializer."""

    ty = type(value)
    marshaller = _marshaller_by_ty.get(ty)

    if marshaller is None:
        marshaller = _marshaller_for(value.__type_info__())
        _marshaller_by_ty[ty] = marshaller

    return marshaller(serializer, value)
