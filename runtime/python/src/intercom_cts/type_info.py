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
from decimal import Decimal
from enum import Enum, Flag, auto
from types import NoneType

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
    F128 = auto()
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
    ANY = auto()


@dataclass(frozen=True, slots=True)
class TypeInfo(typing.Generic[T]):
    name: str
    ty: type[T]
    flags: TypeFlag
    kind: TypeKind
    key_info: typing.Any = None
    element_info: typing.Any = None
    length: int = 0
    max: T | None = None
    min: T | None = None

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
            TypeKind.F128,
            TypeKind.CHAR8,
            TypeKind.CHAR16,
        ]


class AttributeDescriptor:
    __slots__ = ("name",)

    def __init__(self, name: str) -> None:
        self.name = name

    def __get__(self, instance: typing.Any, owner: type | None = None) -> typing.Any:
        return getattr(instance, self.name)

    def __set__(self, instance: typing.Any, value: typing.Any) -> None:
        setattr(instance, self.name, value)


class MemberDescriptor(typing.Protocol):
    def __get__(
        self, instance: typing.Any, owner: type | None = None, /
    ) -> typing.Any: ...
    def __set__(self, instance: typing.Any, value: typing.Any, /) -> None: ...


@dataclass(frozen=True, slots=True)
class MemberInfo(typing.Generic[T]):
    name: str
    descriptor: AttributeDescriptor | MemberDescriptor
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
    ANY = TypeInfo[typing.Any](
        name="any",
        ty=typing.Any,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.ANY,
    )
    NONE = TypeInfo[None](
        name="none",
        ty=NoneType,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.NONE,
    )
    BOOL = TypeInfo[bool](
        name="bool",
        ty=bool,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.BOOL,
    )
    I8 = TypeInfo[int](
        name="i8",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I8,
        min=-(1 << 7),
        max=(1 << 7) - 1,
    )
    U8 = TypeInfo[int](
        name="u8",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U8,
        min=0,
        max=(1 << 8) - 1,
    )
    I16 = TypeInfo[int](
        name="i16",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I16,
        min=-(1 << 15),
        max=(1 << 15) - 1,
    )
    U16 = TypeInfo[int](
        name="u16",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U16,
        min=0,
        max=(1 << 16) - 1,
    )
    I32 = TypeInfo[int](
        name="i32",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I32,
        min=-(1 << 31),
        max=(1 << 31) - 1,
    )
    U32 = TypeInfo[int](
        name="u32",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U32,
        min=0,
        max=(1 << 32) - 1,
    )
    I64 = TypeInfo[int](
        name="i64",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.I64,
        min=-(1 << 63),
        max=(1 << 63) - 1,
    )
    U64 = TypeInfo[int](
        name="u64",
        ty=int,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.U64,
        min=0,
        max=(1 << 64) - 1,
    )
    F32 = TypeInfo[float](
        name="f32",
        ty=float,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.F32,
        min=-3.4028234663852886e38,
        max=3.4028234663852886e38,
    )
    F64 = TypeInfo[float](
        name="f64",
        ty=float,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.F64,
        min=-1.7976931348623157e308,
        max=1.7976931348623157e308,
    )
    F128 = TypeInfo[Decimal](
        name="f128",
        ty=Decimal,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.F128,
        min=Decimal(-((1 << 113) - 1) << (16383 - 112)),
        max=Decimal(((1 << 113) - 1) << (16383 - 112)),
    )
    CHAR8 = TypeInfo[str](
        name="char8",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.CHAR8,
        length=1,
        min=chr(0),
        max=chr(0xFF),
    )
    CHAR16 = TypeInfo[str](
        name="char16",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.CHAR16,
        length=1,
        min=chr(0),
        max=chr(0xFFFF),
    )
    STRING8 = TypeInfo[str](
        name="string8",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.STRING8,
    )
    STRING16 = TypeInfo[str](
        name="string16",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.STRING16,
    )


def bounded_string(wide: bool, bound: int) -> TypeInfo:
    return TypeInfo(
        name="string16" if wide else "string8",
        ty=str,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.STRING16 if wide else TypeKind.STRING8,
        length=bound,
    )


def array_of(elem: type[TypeDescriptor], length: int) -> TypeInfo:
    return TypeInfo(
        name="array",
        ty=list,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.ARRAY,
        element_info=type_info(elem),
        length=length,
    )


def sequence_of(elem: type[TypeDescriptor], bound: int | None = None) -> TypeInfo:
    return TypeInfo(
        name="sequence",
        ty=list,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.SEQUENCE,
        element_info=type_info(elem),
        length=bound or 0,
    )


def map_of(
    key: type[TypeDescriptor], elem: type[TypeDescriptor], bound: int | None = None
) -> TypeInfo:
    return TypeInfo(
        name="map",
        ty=dict,
        flags=TypeFlag.IS_FINAL,
        kind=TypeKind.MAP,
        key_info=type_info(key),
        element_info=type_info(elem),
        length=bound or 0,
    )
