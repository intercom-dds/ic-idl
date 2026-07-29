// Copyright 2023 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{MemberFlag, TypeFlag};

#[must_use]
#[derive(Debug, Copy, Clone)]
pub struct TypeInfo<'a> {
    pub name: &'a str,
    pub flags: TypeFlag,
    pub kind: TypeKind,
    pub key_info: Option<&'a TypeInfo<'a>>,
    pub element_info: Option<&'a TypeInfo<'a>>,
}

impl TypeInfo<'_> {
    /// Check if this type is final
    #[must_use]
    pub fn is_final(&self) -> bool {
        self.flags.contains(TypeFlag::IS_FINAL)
    }

    /// Check if this type is appendable
    #[must_use]
    pub fn is_appendable(&self) -> bool {
        self.flags.contains(TypeFlag::IS_APPENDABLE)
    }

    /// Check if this type is mutable
    #[must_use]
    pub fn is_mutable(&self) -> bool {
        self.flags.contains(TypeFlag::IS_MUTABLE)
    }

    /// Check if this type is a primitive type
    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        use TypeKind::{Bool, Char8, Char16, F32, F64, I8, I16, I32, I64, U8, U16, U32, U64};
        matches!(
            self.kind,
            Bool | I8 | U8 | I16 | U16 | I32 | U32 | I64 | U64 | F32 | F64 | Char8 | Char16
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemberInfo<'a> {
    pub name: &'a str,
    pub member_id: u32,
    pub flags: MemberFlag,
    pub type_info: &'a TypeInfo<'a>,
}

const DISC_TYPE_INFO: TypeInfo<'static> = TypeInfo {
    name: "discriminator",
    flags: TypeFlag::IS_FINAL,
    kind: TypeKind::I32,
    key_info: None,
    element_info: None,
};

#[doc(hidden)]
pub const DISC_INFO: MemberInfo<'static> = MemberInfo {
    name: "$discriminator",
    member_id: 0,
    flags: MemberFlag::IS_MUST_UNDERSTAND,
    type_info: &DISC_TYPE_INFO,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKind {
    None,
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Char8,
    Char16,
    Alias,
    Struct,
    Union,
    Bitmask,
    Enum,
    String8,
    String16,
    Annotation,
    Array,
    Map,
    Sequence,
}

/// Provides type metadata for static types.
///
/// This trait is used to construct references to `TypeInfo` in `MemberInfo`
/// definitions for static types. It allows generic code to access type
/// information at compile time.
pub trait TypeDescriptor {
    /// Type information for this type.
    const TYPE_INFO: &'static TypeInfo<'static>;

    /// Member information for this type.
    const MEMBER_INFO: &'static [MemberInfo<'static>];
}

pub const fn type_info<T: TypeDescriptor>() -> &'static TypeInfo<'static> {
    T::TYPE_INFO
}

#[must_use]
pub const fn member_info<T: TypeDescriptor>() -> &'static [MemberInfo<'static>] {
    T::MEMBER_INFO
}

impl TypeDescriptor for () {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "null",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::None,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for bool {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "bool",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::Bool,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for u8 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "u8",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::U8,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for u16 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "u16",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::U16,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for u32 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "u32",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::U32,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for u64 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "u64",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::U64,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for i8 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "i8",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::I8,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for i16 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "i16",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::I16,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for i32 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "i32",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::I32,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for i64 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "i64",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::I64,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for f32 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "f32",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::F32,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for f64 {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "f64",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::F64,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for String {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "string",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::String8,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for str {
    const TYPE_INFO: &'static TypeInfo<'static> = String::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor> TypeDescriptor for Vec<T> {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "sequence",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::Sequence,
        key_info: None,
        element_info: Some(T::TYPE_INFO),
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<K: TypeDescriptor, V: TypeDescriptor> TypeDescriptor for BTreeMap<K, V> {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "map",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::Map,
        key_info: Some(K::TYPE_INFO),
        element_info: Some(V::TYPE_INFO),
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor> TypeDescriptor for Option<T> {
    const TYPE_INFO: &'static TypeInfo<'static> = T::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor, const N: usize> TypeDescriptor for [T; N] {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "array",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::Array,
        key_info: None,
        element_info: Some(T::TYPE_INFO),
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for char {
    const TYPE_INFO: &'static TypeInfo<'static> = &TypeInfo {
        name: "char",
        flags: TypeFlag::IS_FINAL,
        kind: TypeKind::Char8,
        key_info: None,
        element_info: None,
    };
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for isize {
    const TYPE_INFO: &'static TypeInfo<'static> = i64::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl TypeDescriptor for usize {
    const TYPE_INFO: &'static TypeInfo<'static> = u64::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor> TypeDescriptor for Box<T> {
    const TYPE_INFO: &'static TypeInfo<'static> = T::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor> TypeDescriptor for &T {
    const TYPE_INFO: &'static TypeInfo<'static> = T::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor> TypeDescriptor for [T] {
    const TYPE_INFO: &'static TypeInfo<'static> = <Vec<T>>::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor> TypeDescriptor for BTreeSet<T> {
    const TYPE_INFO: &'static TypeInfo<'static> = <Vec<T>>::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<T: TypeDescriptor, H> TypeDescriptor for HashSet<T, H> {
    const TYPE_INFO: &'static TypeInfo<'static> = <Vec<T>>::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}

impl<K: TypeDescriptor, V: TypeDescriptor, H> TypeDescriptor for HashMap<K, V, H> {
    const TYPE_INFO: &'static TypeInfo<'static> = <BTreeMap<K, V>>::TYPE_INFO;
    const MEMBER_INFO: &'static [MemberInfo<'static>] = &[];
}
