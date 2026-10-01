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
from dataclasses import dataclass
from enum import Enum, Flag, auto
from types import MemberDescriptorType

__all__ = [
    "BuiltinTypes",
    "K",
    "MemberFlag",
    "MemberInfo",
    "T",
    "TypeFlag",
    "TypeInfo",
    "TypeKind",
    "V",
    "member_info",
    "type_info",
]

T = typing.TypeVar("T")
K = typing.TypeVar("K")
V = typing.TypeVar("V")


class MemberFlag(Flag):
    NONE = 0
    TRY_CONSTRUCT1 = 0x01
    TRY_CONSTRUCT2 = 0x02
    IS_EXTERNAL = 0x04
    IS_OPTIONAL = 0x08
    IS_MUST_UNDERSTAND = 0x10
    IS_KEY = 0x20
    IS_DEFAULT = 0x40


class TypeFlag(Flag):
    NONE = 0
    IS_FINAL = 0x01
    IS_APPENDABLE = 0x02
    IS_MUTABLE = 0x04
    IS_NESTED = 0x08
    IS_AUTOID_HASH = 0x10
    IS_KEYED = 0x20


class TypeKind(Enum):
    NONE = auto()
    BOOL = auto()
    I8 = auto()
    U8 = auto()
    I16 = auto()
    U16 = auto()
    I32 = auto()
    U32 = auto()
    I64 = auto()
    U64 = auto()
    F32 = auto()
    F64 = auto()
    CHAR8 = auto()
    CHAR16 = auto()
    ALIAS = auto()
    STRUCT = auto()
    UNION = auto()
    BITMASK = auto()
    ENUM = auto()
    STRING8 = auto()
    STRING16 = auto()
    ANNOTATION = auto()
    ARRAY = auto()
    MAP = auto()
    SEQUENCE = auto()


@dataclass(frozen=True, slots=True)
class TypeInfo[T]:
    name: str
    ty: T
    flags: TypeFlag
    kind: TypeKind
    key_info: typing.Any = None
    element_info: typing.Any = None
    length: int = 0

    def is_final(self) -> bool:
        return TypeFlag.IS_FINAL in self.flags

    def is_appendable(self) -> bool:
        return TypeFlag.IS_APPENDABLE in self.flags

    def is_mutable(self) -> bool:
        return TypeFlag.IS_MUTABLE in self.flags

    def is_primitive(self) -> bool:
        return self.kind in [
            TypeKind.BOOL,
            TypeKind.I8,
            TypeKind.U8,
            TypeKind.I16,
            TypeKind.U16,
            TypeKind.I32,
            TypeKind.U32,
            TypeKind.I64,
            TypeKind.U64,
            TypeKind.F32,
            TypeKind.F64,
            TypeKind.CHAR8,
            TypeKind.CHAR16,
        ]


@dataclass(frozen=True, slots=True)
class MemberInfo[T]:
    name: str
    descriptor: MemberDescriptorType
    member_id: int
    flags: MemberFlag
    type_info: TypeInfo
    key: typing.Any = None


class TypeDescriptor(typing.Protocol):
    @classmethod
    def __type_info__(self) -> TypeInfo[typing.Self]:
        pass

    @classmethod
    def __member_info__(self) -> list[MemberInfo[typing.Self]]:
        pass


def type_info(ty: TypeDescriptor | type[TypeDescriptor]) -> TypeInfo:
    return ty.__type_info__()


def member_info(ty: TypeDescriptor | type[TypeDescriptor]) -> list[MemberInfo]:
    return ty.__member_info__()


class BuiltinTypes:
    NONE = TypeInfo(
        name="bool",
        ty=bool,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.NONE,
        key_info=None,
        element_info=None,
    )
    BOOL = TypeInfo(
        name="bool",
        ty=bool,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.BOOL,
        key_info=None,
        element_info=None,
    )
    I8 = TypeInfo(
        name="i8",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I8,
        key_info=None,
        element_info=None,
    )
    U8 = TypeInfo(
        name="u8",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U8,
        key_info=None,
        element_info=None,
    )
    I16 = TypeInfo(
        name="i16",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I16,
        key_info=None,
        element_info=None,
    )
    U16 = TypeInfo(
        name="u16",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U16,
        key_info=None,
        element_info=None,
    )
    I32 = TypeInfo(
        name="i32",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I32,
        key_info=None,
        element_info=None,
    )
    U32 = TypeInfo(
        name="u32",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U32,
        key_info=None,
        element_info=None,
    )
    I64 = TypeInfo(
        name="i64",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I64,
        key_info=None,
        element_info=None,
    )
    U64 = TypeInfo(
        name="u64",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U64,
        key_info=None,
        element_info=None,
    )
    F32 = TypeInfo(
        name="f32",
        ty=float,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.F32,
        key_info=None,
        element_info=None,
    )
    F64 = TypeInfo(
        name="f64",
        ty=float,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.F64,
        key_info=None,
        element_info=None,
    )
    CHAR8 = TypeInfo(
        name="char8",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.CHAR8,
        key_info=None,
        element_info=None,
    )
    CHAR16 = TypeInfo(
        name="char16",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.CHAR16,
        key_info=None,
        element_info=None,
    )
    STRING8 = TypeInfo(
        name="string8",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.STRING8,
        key_info=None,
        element_info=None,
    )
    STRING16 = TypeInfo(
        name="string16",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.STRING16,
        key_info=None,
        element_info=None,
    )


def array_of(elem: type[TypeDescriptor], length: int) -> TypeInfo:
    return TypeInfo(
        name="array",
        ty=list,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.ARRAY,
        key_info=None,
        element_info=type_info(elem),
        length=length,
    )


def sequence_of(elem: type[TypeDescriptor], bound: int | None = None) -> TypeInfo:
    return TypeInfo(
        name="sequence",
        ty=list,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.SEQUENCE,
        key_info=None,
        element_info=type_info(elem),
        length=bound or 0,
    )


def map_of(
    key: type[TypeDescriptor], elem: type[TypeDescriptor], bound: int | None = None
) -> TypeInfo:
    return TypeInfo(
        name="map",
        ty=list,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.SEQUENCE,
        key_info=type_info(key),
        element_info=type_info(elem),
        length=bound or 0,
    )
