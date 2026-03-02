// @generated
// Copyright 2025 KONGSBERG
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

pub type TypeKind = u8;

pub const TK_NONE: crate::types::xtypes::TypeKind = 0;

pub const TK_BOOLEAN: crate::types::xtypes::TypeKind = 1;

pub const TK_BYTE: crate::types::xtypes::TypeKind = 2;

pub const TK_INT16: crate::types::xtypes::TypeKind = 3;

pub const TK_INT32: crate::types::xtypes::TypeKind = 4;

pub const TK_INT64: crate::types::xtypes::TypeKind = 5;

pub const TK_UINT16: crate::types::xtypes::TypeKind = 6;

pub const TK_UINT32: crate::types::xtypes::TypeKind = 7;

pub const TK_UINT64: crate::types::xtypes::TypeKind = 8;

pub const TK_FLOAT32: crate::types::xtypes::TypeKind = 9;

pub const TK_FLOAT64: crate::types::xtypes::TypeKind = 10;

pub const TK_FLOAT128: crate::types::xtypes::TypeKind = 11;

pub const TK_INT8: crate::types::xtypes::TypeKind = 12;

pub const TK_UINT8: crate::types::xtypes::TypeKind = 13;

pub const TK_CHAR8: crate::types::xtypes::TypeKind = 16;

pub const TK_CHAR16: crate::types::xtypes::TypeKind = 17;

pub const TK_STRING8: crate::types::xtypes::TypeKind = 32;

pub const TK_STRING16: crate::types::xtypes::TypeKind = 33;

pub const TK_ALIAS: crate::types::xtypes::TypeKind = 48;

pub const TK_ENUM: crate::types::xtypes::TypeKind = 64;

pub const TK_BITMASK: crate::types::xtypes::TypeKind = 65;

pub const TK_ANNOTATION: crate::types::xtypes::TypeKind = 80;

pub const TK_STRUCTURE: crate::types::xtypes::TypeKind = 81;

pub const TK_UNION: crate::types::xtypes::TypeKind = 82;

pub const TK_BITSET: crate::types::xtypes::TypeKind = 83;

pub const TK_SEQUENCE: crate::types::xtypes::TypeKind = 96;

pub const TK_ARRAY: crate::types::xtypes::TypeKind = 97;

pub const TK_MAP: crate::types::xtypes::TypeKind = 98;

pub type EquivalenceKind = u8;

pub const EK_MINIMAL: u8 = 241;

pub const EK_COMPLETE: u8 = 242;

pub const EK_BOTH: u8 = 243;

pub type TypeIdentiferKind = u8;

pub const TI_STRING8_SMALL: u8 = 112;

pub const TI_STRING8_LARGE: u8 = 113;

pub const TI_STRING16_SMALL: u8 = 114;

pub const TI_STRING16_LARGE: u8 = 115;

pub const TI_PLAIN_SEQUENCE_SMALL: u8 = 128;

pub const TI_PLAIN_SEQUENCE_LARGE: u8 = 129;

pub const TI_PLAIN_ARRAY_SMALL: u8 = 144;

pub const TI_PLAIN_ARRAY_LARGE: u8 = 145;

pub const TI_PLAIN_MAP_SMALL: u8 = 160;

pub const TI_PLAIN_MAP_LARGE: u8 = 161;

pub const TI_STRONGLY_CONNECTED_COMPONENT: u8 = 176;

pub const MEMBER_NAME_MAX_LENGTH: i32 = 256;

pub type MemberName = ::std::string::String;

pub const TYPE_NAME_MAX_LENGTH: i32 = 256;

pub type QualifiedTypeName = ::std::string::String;

pub type PrimitiveTypeId = u8;

pub type EquivalenceHash = [u8; 14];

pub type NameHash = [u8; 4];

pub type LBound = u32;

pub type LBoundSeq = ::std::vec::Vec<crate::types::xtypes::LBound>;

pub const INVALID_LBOUND: crate::types::xtypes::LBound = 0;

pub type SBound = u8;

pub type SBoundSeq = ::std::vec::Vec<crate::types::xtypes::SBound>;

pub const INVALID_SBOUND: crate::types::xtypes::SBound = 0;

::intercom_cts::bitmask! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub MemberFlag: u32 {
        TRY_CONSTRUCT1 = 1,
        TRY_CONSTRUCT2 = 2,
        IS_EXTERNAL = 4,
        IS_OPTIONAL = 8,
        IS_MUST_UNDERSTAND = 16,
        IS_KEY = 32,
        IS_DEFAULT = 64,
    }
}

impl MemberFlag {
    #[must_use]
    pub const fn new() -> Self {
        Self::nil()
    }
}

impl ::std::default::Default for MemberFlag {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MemberFlag {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MemberFlag",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,
        kind: ::intercom_cts::TypeKind::Bitmask,
        key_info: None,
        element_info: Some(::intercom_cts::type_info::<u32>()),
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "TRY_CONSTRUCT1",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "TRY_CONSTRUCT2",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_EXTERNAL",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_OPTIONAL",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_MUST_UNDERSTAND",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_KEY",
            member_id: 5,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_DEFAULT",
            member_id: 6,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlag>(),
        },
    ];
};

::intercom_cts::bitmask! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub MemberFlagExtended: u32 {
        IS_DISCRIMINATOR = 65_536,
        IS_AIR_DUMMY = 131_072,
        IS_XRI_SEQUENCE = 262_144,
        IS_ELEMENT_SIZE = 524_288,
        HAS_DYNAMIC_ELEMENT_SIZE = 1_048_576,
        IS_VENDOR_EXTENSION = 2_097_152,
        IS_IMPLICIT_KEY = 4_194_304,
        IS_INTEGER_RANGE_VALUE = 8_388_608,
        IS_USE_DEFAULT_TRY_CONSTRUCT = 16_777_216,
        IS_DISCARD_TRY_CONSTRUCT = 33_554_432,
        IS_TRIM_TRY_CONSTRUCT = 67_108_864,
    }
}

impl MemberFlagExtended {
    #[must_use]
    pub const fn new() -> Self {
        Self::nil()
    }
}

impl ::std::default::Default for MemberFlagExtended {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MemberFlagExtended {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MemberFlagExtended",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,
        kind: ::intercom_cts::TypeKind::Bitmask,
        key_info: None,
        element_info: Some(::intercom_cts::type_info::<u32>()),
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "IS_DISCRIMINATOR",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_AIR_DUMMY",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_XRI_SEQUENCE",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_ELEMENT_SIZE",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "HAS_DYNAMIC_ELEMENT_SIZE",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_VENDOR_EXTENSION",
            member_id: 5,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_IMPLICIT_KEY",
            member_id: 6,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_INTEGER_RANGE_VALUE",
            member_id: 7,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_USE_DEFAULT_TRY_CONSTRUCT",
            member_id: 8,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_DISCARD_TRY_CONSTRUCT",
            member_id: 9,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_TRIM_TRY_CONSTRUCT",
            member_id: 10,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberFlagExtended>(),
        },
    ];
};

pub type CollectionElementFlag = crate::types::xtypes::MemberFlag;

pub type StructMemberFlag = crate::types::xtypes::MemberFlag;

pub type UnionMemberFlag = crate::types::xtypes::MemberFlag;

pub type UnionDiscriminatorFlag = crate::types::xtypes::MemberFlag;

pub type EnumeratedLiteralFlag = crate::types::xtypes::MemberFlag;

pub type AnnotationParameterFlag = crate::types::xtypes::MemberFlag;

pub type AliasMemberFlag = crate::types::xtypes::MemberFlag;

pub type BitflagFlag = crate::types::xtypes::MemberFlag;

pub type BitsetMemberFlag = crate::types::xtypes::MemberFlag;

pub const MEMBER_FLAG_MINIMAL_MASK: u16 = 63;

::intercom_cts::bitmask! {
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub TypeFlag: u32 {
        IS_FINAL = 1,
        IS_APPENDABLE = 2,
        IS_MUTABLE = 4,
        IS_NESTED = 8,
        IS_AUTOID_HASH = 16,
    }
}

impl TypeFlag {
    #[must_use]
    pub const fn new() -> Self {
        Self::nil()
    }
}

impl ::std::default::Default for TypeFlag {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeFlag {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeFlag",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,
        kind: ::intercom_cts::TypeKind::Bitmask,
        key_info: None,
        element_info: Some(::intercom_cts::type_info::<u32>()),
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "IS_FINAL",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_APPENDABLE",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_MUTABLE",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_NESTED",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "IS_AUTOID_HASH",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeFlag>(),
        },
    ];
};

pub type StructTypeFlag = crate::types::xtypes::TypeFlag;

pub type UnionTypeFlag = crate::types::xtypes::TypeFlag;

pub type CollectionTypeFlag = crate::types::xtypes::TypeFlag;

pub type AnnotationTypeFlag = crate::types::xtypes::TypeFlag;

pub type AliasTypeFlag = crate::types::xtypes::TypeFlag;

pub type EnumTypeFlag = crate::types::xtypes::TypeFlag;

pub type BitmaskTypeFlag = crate::types::xtypes::TypeFlag;

pub type BitsetTypeFlag = crate::types::xtypes::TypeFlag;

pub const TYPE_FLAG_MINIMAL_MASK: u16 = 7;

pub type MemberId = u32;

pub const MEMBER_ID_INVALID: crate::types::xtypes::MemberId = 268_435_455;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TypeObjectHashId {
    EkComplete(crate::types::xtypes::EquivalenceHash),
    EkMinimal(crate::types::xtypes::EquivalenceHash),
    Null,
}

impl TypeObjectHashId {
    #[must_use]
    pub fn new() -> Self {
        Self::Null
    }

    #[must_use]
    pub const fn disc(&self) -> u8 {
        match self {
            Self::EkComplete(_) => crate::types::xtypes::EK_COMPLETE,
            Self::EkMinimal(_) => crate::types::xtypes::EK_MINIMAL,
            Self::Null => 0,
        }
    }
}

impl From<u8> for TypeObjectHashId {
    fn from(disc: u8) -> Self {
        match disc {
            crate::types::xtypes::EK_COMPLETE => {
                Self::EkComplete(<crate::types::xtypes::EquivalenceHash>::default())
            }
            crate::types::xtypes::EK_MINIMAL => {
                Self::EkMinimal(<crate::types::xtypes::EquivalenceHash>::default())
            }
            _ => Self::default(),
        }
    }
}

impl ::std::default::Default for TypeObjectHashId {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeObjectHashId {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeObjectHashId",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Union,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "hash",
        member_id: 1,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::EquivalenceHash>(),
    }];

    impl ::intercom_cts::Marshal for TypeObjectHashId {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::UnionSerializer as _;

            let mut state = ar.encode_union(&TYPE_INFO)?;
            state.encode_discriminant(&self.disc())?;
            match self {
                Self::EkComplete(v) | Self::EkMinimal(v) => {
                    state.encode_variant(&MEMBER_INFO[0], v)
                }
                _ => state.encode_null(),
            }
        }
    }

    impl ::intercom_cts::Unmarshal for TypeObjectHashId {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::UnionDeserializer as _;

            let mut state = ar.decode_union(&TYPE_INFO)?;
            let mut disc = u8::default();
            state.decode_discriminant(&mut disc)?;
            *self = match disc {
                crate::types::xtypes::EK_COMPLETE | crate::types::xtypes::EK_MINIMAL => {
                    let mut value = <crate::types::xtypes::EquivalenceHash>::default();
                    state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                    Self::EkComplete(value)
                }
                _ => Self::Null,
            };
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StringSTypeDefn {
    pub bound: crate::types::xtypes::SBound,
}

impl StringSTypeDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bound: <crate::types::xtypes::SBound>::default(),
        }
    }
}

impl ::std::default::Default for StringSTypeDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for StringSTypeDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::StringSTypeDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "bound",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::SBound>(),
    }];

    impl ::intercom_cts::Marshal for StringSTypeDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bound)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for StringSTypeDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bound)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StringLTypeDefn {
    pub bound: crate::types::xtypes::LBound,
}

impl StringLTypeDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bound: <crate::types::xtypes::LBound>::default(),
        }
    }
}

impl ::std::default::Default for StringLTypeDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for StringLTypeDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::StringLTypeDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "bound",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::LBound>(),
    }];

    impl ::intercom_cts::Marshal for StringLTypeDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bound)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for StringLTypeDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bound)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainCollectionHeader {
    pub equiv_kind: crate::types::xtypes::EquivalenceKind,
    pub element_flags: crate::types::xtypes::CollectionElementFlag,
}

impl PlainCollectionHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            equiv_kind: <crate::types::xtypes::EquivalenceKind>::default(),
            element_flags: <crate::types::xtypes::CollectionElementFlag>::default(),
        }
    }
}

impl ::std::default::Default for PlainCollectionHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainCollectionHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainCollectionHeader",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "equiv_kind",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::EquivalenceKind>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_flags",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionElementFlag>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainCollectionHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.equiv_kind)?;
            state.encode_field(&MEMBER_INFO[1], &self.element_flags)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainCollectionHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.equiv_kind)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.element_flags)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainSequenceSElemDefn {
    pub header: crate::types::xtypes::PlainCollectionHeader,
    pub bound: crate::types::xtypes::SBound,
    pub element_identifier: Box<crate::types::xtypes::TypeIdentifier>,
}

impl PlainSequenceSElemDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: <crate::types::xtypes::PlainCollectionHeader>::default(),
            bound: <crate::types::xtypes::SBound>::default(),
            element_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for PlainSequenceSElemDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainSequenceSElemDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainSequenceSElemDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bound",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::SBound>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_identifier",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainSequenceSElemDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.header)?;
            state.encode_field(&MEMBER_INFO[1], &self.bound)?;
            state.encode_field(&MEMBER_INFO[2], &self.element_identifier)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainSequenceSElemDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.bound)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element_identifier)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainSequenceLElemDefn {
    pub header: crate::types::xtypes::PlainCollectionHeader,
    pub bound: crate::types::xtypes::LBound,
    pub element_identifier: Box<crate::types::xtypes::TypeIdentifier>,
}

impl PlainSequenceLElemDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: <crate::types::xtypes::PlainCollectionHeader>::default(),
            bound: <crate::types::xtypes::LBound>::default(),
            element_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for PlainSequenceLElemDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainSequenceLElemDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainSequenceLElemDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bound",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::LBound>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_identifier",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainSequenceLElemDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.header)?;
            state.encode_field(&MEMBER_INFO[1], &self.bound)?;
            state.encode_field(&MEMBER_INFO[2], &self.element_identifier)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainSequenceLElemDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.bound)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element_identifier)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainArraySElemDefn {
    pub header: crate::types::xtypes::PlainCollectionHeader,
    pub array_bound_seq: crate::types::xtypes::SBoundSeq,
    pub element_identifier: Box<crate::types::xtypes::TypeIdentifier>,
}

impl PlainArraySElemDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: <crate::types::xtypes::PlainCollectionHeader>::default(),
            array_bound_seq: <crate::types::xtypes::SBoundSeq>::default(),
            element_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for PlainArraySElemDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainArraySElemDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainArraySElemDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "array_bound_seq",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::SBoundSeq>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_identifier",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainArraySElemDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.header)?;
            state.encode_field(&MEMBER_INFO[1], &self.array_bound_seq)?;
            state.encode_field(&MEMBER_INFO[2], &self.element_identifier)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainArraySElemDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.array_bound_seq)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element_identifier)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainArrayLElemDefn {
    pub header: crate::types::xtypes::PlainCollectionHeader,
    pub array_bound_seq: crate::types::xtypes::LBoundSeq,
    pub element_identifier: Box<crate::types::xtypes::TypeIdentifier>,
}

impl PlainArrayLElemDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: <crate::types::xtypes::PlainCollectionHeader>::default(),
            array_bound_seq: <crate::types::xtypes::LBoundSeq>::default(),
            element_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for PlainArrayLElemDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainArrayLElemDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainArrayLElemDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "array_bound_seq",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::LBoundSeq>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_identifier",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainArrayLElemDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.header)?;
            state.encode_field(&MEMBER_INFO[1], &self.array_bound_seq)?;
            state.encode_field(&MEMBER_INFO[2], &self.element_identifier)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainArrayLElemDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.array_bound_seq)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element_identifier)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainMapSTypeDefn {
    pub header: crate::types::xtypes::PlainCollectionHeader,
    pub bound: crate::types::xtypes::SBound,
    pub element_identifier: Box<crate::types::xtypes::TypeIdentifier>,
    pub key_flags: crate::types::xtypes::CollectionElementFlag,
    pub key_identifier: Box<crate::types::xtypes::TypeIdentifier>,
}

impl PlainMapSTypeDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: <crate::types::xtypes::PlainCollectionHeader>::default(),
            bound: <crate::types::xtypes::SBound>::default(),
            element_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
            key_flags: <crate::types::xtypes::CollectionElementFlag>::default(),
            key_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for PlainMapSTypeDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainMapSTypeDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainMapSTypeDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bound",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::SBound>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_identifier",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "key_flags",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionElementFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "key_identifier",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainMapSTypeDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.header)?;
            state.encode_field(&MEMBER_INFO[1], &self.bound)?;
            state.encode_field(&MEMBER_INFO[2], &self.element_identifier)?;
            state.encode_field(&MEMBER_INFO[3], &self.key_flags)?;
            state.encode_field(&MEMBER_INFO[4], &self.key_identifier)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainMapSTypeDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.bound)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element_identifier)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.key_flags)?;
            state.decode_field(&MEMBER_INFO[4], &mut self.key_identifier)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlainMapLTypeDefn {
    pub header: crate::types::xtypes::PlainCollectionHeader,
    pub bound: crate::types::xtypes::LBound,
    pub element_identifier: Box<crate::types::xtypes::TypeIdentifier>,
    pub key_flags: crate::types::xtypes::CollectionElementFlag,
    pub key_identifier: Box<crate::types::xtypes::TypeIdentifier>,
}

impl PlainMapLTypeDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: <crate::types::xtypes::PlainCollectionHeader>::default(),
            bound: <crate::types::xtypes::LBound>::default(),
            element_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
            key_flags: <crate::types::xtypes::CollectionElementFlag>::default(),
            key_identifier: Box::<crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for PlainMapLTypeDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for PlainMapLTypeDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::PlainMapLTypeDefn",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bound",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::LBound>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element_identifier",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "key_flags",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionElementFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "key_identifier",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for PlainMapLTypeDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.header)?;
            state.encode_field(&MEMBER_INFO[1], &self.bound)?;
            state.encode_field(&MEMBER_INFO[2], &self.element_identifier)?;
            state.encode_field(&MEMBER_INFO[3], &self.key_flags)?;
            state.encode_field(&MEMBER_INFO[4], &self.key_identifier)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for PlainMapLTypeDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.bound)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element_identifier)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.key_flags)?;
            state.decode_field(&MEMBER_INFO[4], &mut self.key_identifier)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StronglyConnectedComponentId {
    pub sc_component_id: crate::types::xtypes::TypeObjectHashId,
    pub scc_length: i32,
    pub scc_index: i32,
}

impl StronglyConnectedComponentId {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sc_component_id: <crate::types::xtypes::TypeObjectHashId>::default(),
            scc_length: 0,
            scc_index: 0,
        }
    }
}

impl ::std::default::Default for StronglyConnectedComponentId {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for StronglyConnectedComponentId {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::StronglyConnectedComponentId",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "sc_component_id",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeObjectHashId>(),
        },
        ::intercom_cts::MemberInfo {
            name: "scc_length",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "scc_index",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i32>(),
        },
    ];

    impl ::intercom_cts::Marshal for StronglyConnectedComponentId {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.sc_component_id)?;
            state.encode_field(&MEMBER_INFO[1], &self.scc_length)?;
            state.encode_field(&MEMBER_INFO[2], &self.scc_index)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for StronglyConnectedComponentId {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.sc_component_id)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.scc_length)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.scc_index)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExtendedTypeDefn {}

impl ExtendedTypeDefn {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for ExtendedTypeDefn {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for ExtendedTypeDefn {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::ExtendedTypeDefn",
        flags: ::intercom_cts::TypeFlag::IS_MUTABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for ExtendedTypeDefn {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for ExtendedTypeDefn {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Empty {}

impl Empty {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for Empty {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::Empty",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for Empty {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for Empty {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TypeIdentifier {
    TkNone(crate::types::xtypes::Empty),
    TkBoolean(crate::types::xtypes::Empty),
    TkByte(crate::types::xtypes::Empty),
    TkInt8(crate::types::xtypes::Empty),
    TkInt16(crate::types::xtypes::Empty),
    TkInt32(crate::types::xtypes::Empty),
    TkInt64(crate::types::xtypes::Empty),
    TkUint8(crate::types::xtypes::Empty),
    TkUint16(crate::types::xtypes::Empty),
    TkUint32(crate::types::xtypes::Empty),
    TkUint64(crate::types::xtypes::Empty),
    TkFloat32(crate::types::xtypes::Empty),
    TkFloat64(crate::types::xtypes::Empty),
    TkFloat128(crate::types::xtypes::Empty),
    TkChar8(crate::types::xtypes::Empty),
    TkChar16(crate::types::xtypes::Empty),
    TiString8Small(crate::types::xtypes::StringSTypeDefn),
    TiString16Small(crate::types::xtypes::StringSTypeDefn),
    TiString8Large(crate::types::xtypes::StringLTypeDefn),
    TiString16Large(crate::types::xtypes::StringLTypeDefn),
    SeqSdefn(crate::types::xtypes::PlainSequenceSElemDefn),
    SeqLdefn(crate::types::xtypes::PlainSequenceLElemDefn),
    ArraySdefn(crate::types::xtypes::PlainArraySElemDefn),
    ArrayLdefn(crate::types::xtypes::PlainArrayLElemDefn),
    MapSdefn(crate::types::xtypes::PlainMapSTypeDefn),
    MapLdefn(crate::types::xtypes::PlainMapLTypeDefn),
    ScComponentId(crate::types::xtypes::StronglyConnectedComponentId),
    EkComplete(crate::types::xtypes::EquivalenceHash),
    EkMinimal(crate::types::xtypes::EquivalenceHash),
    ExtendedDefn(crate::types::xtypes::ExtendedTypeDefn),
}

impl TypeIdentifier {
    #[must_use]
    pub fn new() -> Self {
        Self::TkNone(Empty::default())
    }

    #[must_use]
    pub const fn disc(&self) -> u8 {
        match self {
            Self::TkNone(_) => crate::types::xtypes::TK_NONE,
            Self::TkBoolean(_) => crate::types::xtypes::TK_BOOLEAN,
            Self::TkByte(_) => crate::types::xtypes::TK_BYTE,
            Self::TkInt8(_) => crate::types::xtypes::TK_INT8,
            Self::TkInt16(_) => crate::types::xtypes::TK_INT16,
            Self::TkInt32(_) => crate::types::xtypes::TK_INT32,
            Self::TkInt64(_) => crate::types::xtypes::TK_INT64,
            Self::TkUint8(_) => crate::types::xtypes::TK_UINT8,
            Self::TkUint16(_) => crate::types::xtypes::TK_UINT16,
            Self::TkUint32(_) => crate::types::xtypes::TK_UINT32,
            Self::TkUint64(_) => crate::types::xtypes::TK_UINT64,
            Self::TkFloat32(_) => crate::types::xtypes::TK_FLOAT32,
            Self::TkFloat64(_) => crate::types::xtypes::TK_FLOAT64,
            Self::TkFloat128(_) => crate::types::xtypes::TK_FLOAT128,
            Self::TkChar8(_) => crate::types::xtypes::TK_CHAR8,
            Self::TkChar16(_) => crate::types::xtypes::TK_CHAR16,
            Self::TiString8Small(_) => crate::types::xtypes::TI_STRING8_SMALL,
            Self::TiString16Small(_) => crate::types::xtypes::TI_STRING16_SMALL,
            Self::TiString8Large(_) => crate::types::xtypes::TI_STRING8_LARGE,
            Self::TiString16Large(_) => crate::types::xtypes::TI_STRING16_LARGE,
            Self::SeqSdefn(_) => crate::types::xtypes::TI_PLAIN_SEQUENCE_SMALL,
            Self::SeqLdefn(_) => crate::types::xtypes::TI_PLAIN_SEQUENCE_LARGE,
            Self::ArraySdefn(_) => crate::types::xtypes::TI_PLAIN_ARRAY_SMALL,
            Self::ArrayLdefn(_) => crate::types::xtypes::TI_PLAIN_ARRAY_LARGE,
            Self::MapSdefn(_) => crate::types::xtypes::TI_PLAIN_MAP_SMALL,
            Self::MapLdefn(_) => crate::types::xtypes::TI_PLAIN_MAP_LARGE,
            Self::ScComponentId(_) => crate::types::xtypes::TI_STRONGLY_CONNECTED_COMPONENT,
            Self::EkComplete(_) => crate::types::xtypes::EK_COMPLETE,
            Self::EkMinimal(_) => crate::types::xtypes::EK_MINIMAL,
            Self::ExtendedDefn(_) => 0,
        }
    }
}

impl From<u8> for TypeIdentifier {
    fn from(disc: u8) -> Self {
        match disc {
            crate::types::xtypes::TK_NONE => Self::TkNone(<crate::types::xtypes::Empty>::default()),
            crate::types::xtypes::TK_BOOLEAN => {
                Self::TkBoolean(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_BYTE => Self::TkByte(<crate::types::xtypes::Empty>::default()),
            crate::types::xtypes::TK_INT8 => Self::TkInt8(<crate::types::xtypes::Empty>::default()),
            crate::types::xtypes::TK_INT16 => {
                Self::TkInt16(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_INT32 => {
                Self::TkInt32(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_INT64 => {
                Self::TkInt64(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_UINT8 => {
                Self::TkUint8(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_UINT16 => {
                Self::TkUint16(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_UINT32 => {
                Self::TkUint32(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_UINT64 => {
                Self::TkUint64(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_FLOAT32 => {
                Self::TkFloat32(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_FLOAT64 => {
                Self::TkFloat64(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_FLOAT128 => {
                Self::TkFloat128(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_CHAR8 => {
                Self::TkChar8(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TK_CHAR16 => {
                Self::TkChar16(<crate::types::xtypes::Empty>::default())
            }
            crate::types::xtypes::TI_STRING8_SMALL => {
                Self::TiString8Small(<crate::types::xtypes::StringSTypeDefn>::default())
            }
            crate::types::xtypes::TI_STRING16_SMALL => {
                Self::TiString16Small(<crate::types::xtypes::StringSTypeDefn>::default())
            }
            crate::types::xtypes::TI_STRING8_LARGE => {
                Self::TiString8Large(<crate::types::xtypes::StringLTypeDefn>::default())
            }
            crate::types::xtypes::TI_STRING16_LARGE => {
                Self::TiString16Large(<crate::types::xtypes::StringLTypeDefn>::default())
            }
            crate::types::xtypes::TI_PLAIN_SEQUENCE_SMALL => {
                Self::SeqSdefn(<crate::types::xtypes::PlainSequenceSElemDefn>::default())
            }
            crate::types::xtypes::TI_PLAIN_SEQUENCE_LARGE => {
                Self::SeqLdefn(<crate::types::xtypes::PlainSequenceLElemDefn>::default())
            }
            crate::types::xtypes::TI_PLAIN_ARRAY_SMALL => {
                Self::ArraySdefn(<crate::types::xtypes::PlainArraySElemDefn>::default())
            }
            crate::types::xtypes::TI_PLAIN_ARRAY_LARGE => {
                Self::ArrayLdefn(<crate::types::xtypes::PlainArrayLElemDefn>::default())
            }
            crate::types::xtypes::TI_PLAIN_MAP_SMALL => {
                Self::MapSdefn(<crate::types::xtypes::PlainMapSTypeDefn>::default())
            }
            crate::types::xtypes::TI_PLAIN_MAP_LARGE => {
                Self::MapLdefn(<crate::types::xtypes::PlainMapLTypeDefn>::default())
            }
            crate::types::xtypes::TI_STRONGLY_CONNECTED_COMPONENT => {
                Self::ScComponentId(<crate::types::xtypes::StronglyConnectedComponentId>::default())
            }
            crate::types::xtypes::EK_COMPLETE => {
                Self::EkComplete(<crate::types::xtypes::EquivalenceHash>::default())
            }
            crate::types::xtypes::EK_MINIMAL => {
                Self::EkMinimal(<crate::types::xtypes::EquivalenceHash>::default())
            }
            _ => Self::default(),
        }
    }
}

impl ::std::default::Default for TypeIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeIdentifier {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeIdentifier",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Union,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "primitive",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::Empty>(),
        },
        ::intercom_cts::MemberInfo {
            name: "string_sdefn",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::StringSTypeDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "string_ldefn",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::StringLTypeDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "seq_sdefn",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainSequenceSElemDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "seq_ldefn",
            member_id: 5,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainSequenceLElemDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "array_sdefn",
            member_id: 6,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainArraySElemDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "array_ldefn",
            member_id: 7,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainArrayLElemDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "map_sdefn",
            member_id: 8,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainMapSTypeDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "map_ldefn",
            member_id: 9,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::PlainMapLTypeDefn>(),
        },
        ::intercom_cts::MemberInfo {
            name: "sc_component_id",
            member_id: 10,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::StronglyConnectedComponentId,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "equivalence_hash",
            member_id: 11,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::EquivalenceHash>(),
        },
        ::intercom_cts::MemberInfo {
            name: "extended_defn",
            member_id: 12,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::ExtendedTypeDefn>(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeIdentifier {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::UnionSerializer as _;

            let mut state = ar.encode_union(&TYPE_INFO)?;
            state.encode_discriminant(&self.disc())?;
            match self {
                Self::TkNone(v)
                | Self::TkBoolean(v)
                | Self::TkByte(v)
                | Self::TkInt8(v)
                | Self::TkInt16(v)
                | Self::TkInt32(v)
                | Self::TkInt64(v)
                | Self::TkUint8(v)
                | Self::TkUint16(v)
                | Self::TkUint32(v)
                | Self::TkUint64(v)
                | Self::TkFloat32(v)
                | Self::TkFloat64(v)
                | Self::TkFloat128(v)
                | Self::TkChar8(v)
                | Self::TkChar16(v) => state.encode_variant(&MEMBER_INFO[0], v),
                Self::TiString8Small(v) | Self::TiString16Small(v) => {
                    state.encode_variant(&MEMBER_INFO[1], v)
                }
                Self::TiString8Large(v) | Self::TiString16Large(v) => {
                    state.encode_variant(&MEMBER_INFO[2], v)
                }
                Self::SeqSdefn(v) => state.encode_variant(&MEMBER_INFO[3], v),
                Self::SeqLdefn(v) => state.encode_variant(&MEMBER_INFO[4], v),
                Self::ArraySdefn(v) => state.encode_variant(&MEMBER_INFO[5], v),
                Self::ArrayLdefn(v) => state.encode_variant(&MEMBER_INFO[6], v),
                Self::MapSdefn(v) => state.encode_variant(&MEMBER_INFO[7], v),
                Self::MapLdefn(v) => state.encode_variant(&MEMBER_INFO[8], v),
                Self::ScComponentId(v) => state.encode_variant(&MEMBER_INFO[9], v),
                Self::EkComplete(v) | Self::EkMinimal(v) => {
                    state.encode_variant(&MEMBER_INFO[10], v)
                }
                _ => state.encode_null(),
            }
        }
    }

    impl ::intercom_cts::Unmarshal for TypeIdentifier {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::UnionDeserializer as _;

            let mut state = ar.decode_union(&TYPE_INFO)?;
            let mut disc = u8::default();
            state.decode_discriminant(&mut disc)?;
            *self = match disc {
                crate::types::xtypes::TK_NONE
                | crate::types::xtypes::TK_BOOLEAN
                | crate::types::xtypes::TK_BYTE
                | crate::types::xtypes::TK_INT8
                | crate::types::xtypes::TK_INT16
                | crate::types::xtypes::TK_INT32
                | crate::types::xtypes::TK_INT64
                | crate::types::xtypes::TK_UINT8
                | crate::types::xtypes::TK_UINT16
                | crate::types::xtypes::TK_UINT32
                | crate::types::xtypes::TK_UINT64
                | crate::types::xtypes::TK_FLOAT32
                | crate::types::xtypes::TK_FLOAT64
                | crate::types::xtypes::TK_FLOAT128
                | crate::types::xtypes::TK_CHAR8
                | crate::types::xtypes::TK_CHAR16 => {
                    let mut value = <crate::types::xtypes::Empty>::default();
                    state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                    Self::TkNone(value)
                }
                crate::types::xtypes::TI_STRING8_SMALL
                | crate::types::xtypes::TI_STRING16_SMALL => {
                    let mut value = <crate::types::xtypes::StringSTypeDefn>::default();
                    state.decode_variant(&MEMBER_INFO[1], &mut value)?;
                    Self::TiString8Small(value)
                }
                crate::types::xtypes::TI_STRING8_LARGE
                | crate::types::xtypes::TI_STRING16_LARGE => {
                    let mut value = <crate::types::xtypes::StringLTypeDefn>::default();
                    state.decode_variant(&MEMBER_INFO[2], &mut value)?;
                    Self::TiString8Large(value)
                }
                crate::types::xtypes::TI_PLAIN_SEQUENCE_SMALL => {
                    let mut value = <crate::types::xtypes::PlainSequenceSElemDefn>::default();
                    state.decode_variant(&MEMBER_INFO[3], &mut value)?;
                    Self::SeqSdefn(value)
                }
                crate::types::xtypes::TI_PLAIN_SEQUENCE_LARGE => {
                    let mut value = <crate::types::xtypes::PlainSequenceLElemDefn>::default();
                    state.decode_variant(&MEMBER_INFO[4], &mut value)?;
                    Self::SeqLdefn(value)
                }
                crate::types::xtypes::TI_PLAIN_ARRAY_SMALL => {
                    let mut value = <crate::types::xtypes::PlainArraySElemDefn>::default();
                    state.decode_variant(&MEMBER_INFO[5], &mut value)?;
                    Self::ArraySdefn(value)
                }
                crate::types::xtypes::TI_PLAIN_ARRAY_LARGE => {
                    let mut value = <crate::types::xtypes::PlainArrayLElemDefn>::default();
                    state.decode_variant(&MEMBER_INFO[6], &mut value)?;
                    Self::ArrayLdefn(value)
                }
                crate::types::xtypes::TI_PLAIN_MAP_SMALL => {
                    let mut value = <crate::types::xtypes::PlainMapSTypeDefn>::default();
                    state.decode_variant(&MEMBER_INFO[7], &mut value)?;
                    Self::MapSdefn(value)
                }
                crate::types::xtypes::TI_PLAIN_MAP_LARGE => {
                    let mut value = <crate::types::xtypes::PlainMapLTypeDefn>::default();
                    state.decode_variant(&MEMBER_INFO[8], &mut value)?;
                    Self::MapLdefn(value)
                }
                crate::types::xtypes::TI_STRONGLY_CONNECTED_COMPONENT => {
                    let mut value = <crate::types::xtypes::StronglyConnectedComponentId>::default();
                    state.decode_variant(&MEMBER_INFO[9], &mut value)?;
                    Self::ScComponentId(value)
                }
                crate::types::xtypes::EK_COMPLETE | crate::types::xtypes::EK_MINIMAL => {
                    let mut value = <crate::types::xtypes::EquivalenceHash>::default();
                    state.decode_variant(&MEMBER_INFO[10], &mut value)?;
                    Self::EkComplete(value)
                }
                _ => {
                    let mut value = <crate::types::xtypes::ExtendedTypeDefn>::default();
                    state.decode_variant(&MEMBER_INFO[11], &mut value)?;
                    Self::ExtendedDefn(value)
                }
            };
            Ok(())
        }
    }
};

pub type TypeIdentifierSeq = ::std::vec::Vec<crate::types::xtypes::TypeIdentifier>;

pub const ANNOTATION_STR_VALUE_MAX_LEN: u32 = 128;

pub const ANNOTATION_OCTETSEC_VALUE_MAX_LEN: u32 = 128;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExtendedAnnotationParameterValue {}

impl ExtendedAnnotationParameterValue {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for ExtendedAnnotationParameterValue {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for ExtendedAnnotationParameterValue {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::ExtendedAnnotationParameterValue",
        flags: ::intercom_cts::TypeFlag::IS_MUTABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for ExtendedAnnotationParameterValue {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for ExtendedAnnotationParameterValue {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum AnnotationParameterValue {
    BooleanValue(bool),
    TkByte(u8),
    TkInt8(u8),
    TkUint8(u8),
    Int16Value(i16),
    Uint16Value(u16),
    Int32Value(i32),
    Uint32Value(u32),
    Int64Value(i64),
    Uint64Value(u64),
    Float32Value(f32),
    Float64Value(f64),
    Float128Value([u8; 16]),
    CharValue(char),
    WcharValue(char),
    EnumeratedValue(i32),
    String8Value(::std::string::String),
    String16Value(::std::string::String),
    ExtendedValue(crate::types::xtypes::ExtendedAnnotationParameterValue),
}

impl AnnotationParameterValue {
    #[must_use]
    pub fn new() -> Self {
        Self::ExtendedValue(<crate::types::xtypes::ExtendedAnnotationParameterValue>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> u8 {
        match self {
            Self::BooleanValue(_) => crate::types::xtypes::TK_BOOLEAN,
            Self::TkByte(_) => crate::types::xtypes::TK_BYTE,
            Self::TkInt8(_) => crate::types::xtypes::TK_INT8,
            Self::TkUint8(_) => crate::types::xtypes::TK_UINT8,
            Self::Int16Value(_) => crate::types::xtypes::TK_INT16,
            Self::Uint16Value(_) => crate::types::xtypes::TK_UINT16,
            Self::Int32Value(_) => crate::types::xtypes::TK_INT32,
            Self::Uint32Value(_) => crate::types::xtypes::TK_UINT32,
            Self::Int64Value(_) => crate::types::xtypes::TK_INT64,
            Self::Uint64Value(_) => crate::types::xtypes::TK_UINT64,
            Self::Float32Value(_) => crate::types::xtypes::TK_FLOAT32,
            Self::Float64Value(_) => crate::types::xtypes::TK_FLOAT64,
            Self::Float128Value(_) => crate::types::xtypes::TK_FLOAT128,
            Self::CharValue(_) => crate::types::xtypes::TK_CHAR8,
            Self::WcharValue(_) => crate::types::xtypes::TK_CHAR16,
            Self::EnumeratedValue(_) => crate::types::xtypes::TK_ENUM,
            Self::String8Value(_) => crate::types::xtypes::TK_STRING8,
            Self::String16Value(_) => crate::types::xtypes::TK_STRING16,
            Self::ExtendedValue(_) => 0,
        }
    }
}

impl From<u8> for AnnotationParameterValue {
    fn from(disc: u8) -> Self {
        match disc {
            crate::types::xtypes::TK_BOOLEAN => Self::BooleanValue(false),
            crate::types::xtypes::TK_BYTE => Self::TkByte(0),
            crate::types::xtypes::TK_INT8 => Self::TkInt8(0),
            crate::types::xtypes::TK_UINT8 => Self::TkUint8(0),
            crate::types::xtypes::TK_INT16 => Self::Int16Value(0),
            crate::types::xtypes::TK_UINT16 => Self::Uint16Value(0),
            crate::types::xtypes::TK_INT32 => Self::Int32Value(0),
            crate::types::xtypes::TK_UINT32 => Self::Uint32Value(0),
            crate::types::xtypes::TK_INT64 => Self::Int64Value(0),
            crate::types::xtypes::TK_UINT64 => Self::Uint64Value(0),
            crate::types::xtypes::TK_FLOAT32 => Self::Float32Value(0_f32),
            crate::types::xtypes::TK_FLOAT64 => Self::Float64Value(0_f64),
            crate::types::xtypes::TK_FLOAT128 => Self::Float128Value(<[u8; 16]>::default()),
            crate::types::xtypes::TK_CHAR8 => Self::CharValue('\0'),
            crate::types::xtypes::TK_CHAR16 => Self::WcharValue('\0'),
            crate::types::xtypes::TK_ENUM => Self::EnumeratedValue(0),
            crate::types::xtypes::TK_STRING8 => {
                Self::String8Value(<::std::string::String>::default())
            }
            crate::types::xtypes::TK_STRING16 => {
                Self::String16Value(<::std::string::String>::default())
            }
            _ => Self::default(),
        }
    }
}

impl ::std::default::Default for AnnotationParameterValue {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for AnnotationParameterValue {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::AnnotationParameterValue",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Union,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "boolean_value",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<bool>(),
        },
        ::intercom_cts::MemberInfo {
            name: "byte_value",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u8>(),
        },
        ::intercom_cts::MemberInfo {
            name: "int16_value",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i16>(),
        },
        ::intercom_cts::MemberInfo {
            name: "uint_16_value",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u16>(),
        },
        ::intercom_cts::MemberInfo {
            name: "int32_value",
            member_id: 5,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "uint32_value",
            member_id: 6,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "int64_value",
            member_id: 7,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i64>(),
        },
        ::intercom_cts::MemberInfo {
            name: "uint64_value",
            member_id: 8,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u64>(),
        },
        ::intercom_cts::MemberInfo {
            name: "float32_value",
            member_id: 9,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<f32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "float64_value",
            member_id: 10,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<f64>(),
        },
        ::intercom_cts::MemberInfo {
            name: "float128_value",
            member_id: 11,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<[u8; 16]>(),
        },
        ::intercom_cts::MemberInfo {
            name: "char_value",
            member_id: 12,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<char>(),
        },
        ::intercom_cts::MemberInfo {
            name: "wchar_value",
            member_id: 13,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<char>(),
        },
        ::intercom_cts::MemberInfo {
            name: "enumerated_value",
            member_id: 14,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "string8_value",
            member_id: 15,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
        ::intercom_cts::MemberInfo {
            name: "string16_value",
            member_id: 16,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
        ::intercom_cts::MemberInfo {
            name: "extended_value",
            member_id: 17,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::ExtendedAnnotationParameterValue,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for AnnotationParameterValue {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::UnionSerializer as _;

            let mut state = ar.encode_union(&TYPE_INFO)?;
            state.encode_discriminant(&self.disc())?;
            match self {
                Self::BooleanValue(v) => state.encode_variant(&MEMBER_INFO[0], v),
                Self::TkByte(v) | Self::TkInt8(v) | Self::TkUint8(v) => {
                    state.encode_variant(&MEMBER_INFO[1], v)
                }
                Self::Int16Value(v) => state.encode_variant(&MEMBER_INFO[2], v),
                Self::Uint16Value(v) => state.encode_variant(&MEMBER_INFO[3], v),
                Self::Int32Value(v) => state.encode_variant(&MEMBER_INFO[4], v),
                Self::Uint32Value(v) => state.encode_variant(&MEMBER_INFO[5], v),
                Self::Int64Value(v) => state.encode_variant(&MEMBER_INFO[6], v),
                Self::Uint64Value(v) => state.encode_variant(&MEMBER_INFO[7], v),
                Self::Float32Value(v) => state.encode_variant(&MEMBER_INFO[8], v),
                Self::Float64Value(v) => state.encode_variant(&MEMBER_INFO[9], v),
                Self::Float128Value(v) => state.encode_variant(&MEMBER_INFO[10], v),
                Self::CharValue(v) => state.encode_variant(&MEMBER_INFO[11], v),
                Self::WcharValue(v) => state.encode_variant(&MEMBER_INFO[12], v),
                Self::EnumeratedValue(v) => state.encode_variant(&MEMBER_INFO[13], v),
                Self::String8Value(v) => state.encode_variant(&MEMBER_INFO[14], v),
                Self::String16Value(v) => state.encode_variant(&MEMBER_INFO[15], v),
                _ => state.encode_null(),
            }
        }
    }

    impl ::intercom_cts::Unmarshal for AnnotationParameterValue {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::UnionDeserializer as _;

            let mut state = ar.decode_union(&TYPE_INFO)?;
            let mut disc = u8::default();
            state.decode_discriminant(&mut disc)?;
            *self = match disc {
                crate::types::xtypes::TK_BOOLEAN => {
                    let mut value = false;
                    state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                    Self::BooleanValue(value)
                }
                crate::types::xtypes::TK_BYTE
                | crate::types::xtypes::TK_INT8
                | crate::types::xtypes::TK_UINT8 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[1], &mut value)?;
                    Self::TkByte(value)
                }
                crate::types::xtypes::TK_INT16 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[2], &mut value)?;
                    Self::Int16Value(value)
                }
                crate::types::xtypes::TK_UINT16 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[3], &mut value)?;
                    Self::Uint16Value(value)
                }
                crate::types::xtypes::TK_INT32 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[4], &mut value)?;
                    Self::Int32Value(value)
                }
                crate::types::xtypes::TK_UINT32 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[5], &mut value)?;
                    Self::Uint32Value(value)
                }
                crate::types::xtypes::TK_INT64 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[6], &mut value)?;
                    Self::Int64Value(value)
                }
                crate::types::xtypes::TK_UINT64 => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[7], &mut value)?;
                    Self::Uint64Value(value)
                }
                crate::types::xtypes::TK_FLOAT32 => {
                    let mut value = 0_f32;
                    state.decode_variant(&MEMBER_INFO[8], &mut value)?;
                    Self::Float32Value(value)
                }
                crate::types::xtypes::TK_FLOAT64 => {
                    let mut value = 0_f64;
                    state.decode_variant(&MEMBER_INFO[9], &mut value)?;
                    Self::Float64Value(value)
                }
                crate::types::xtypes::TK_FLOAT128 => {
                    let mut value = <[u8; 16]>::default();
                    state.decode_variant(&MEMBER_INFO[10], &mut value)?;
                    Self::Float128Value(value)
                }
                crate::types::xtypes::TK_CHAR8 => {
                    let mut value = '\0';
                    state.decode_variant(&MEMBER_INFO[11], &mut value)?;
                    Self::CharValue(value)
                }
                crate::types::xtypes::TK_CHAR16 => {
                    let mut value = '\0';
                    state.decode_variant(&MEMBER_INFO[12], &mut value)?;
                    Self::WcharValue(value)
                }
                crate::types::xtypes::TK_ENUM => {
                    let mut value = 0;
                    state.decode_variant(&MEMBER_INFO[13], &mut value)?;
                    Self::EnumeratedValue(value)
                }
                crate::types::xtypes::TK_STRING8 => {
                    let mut value = <::std::string::String>::default();
                    state.decode_variant(&MEMBER_INFO[14], &mut value)?;
                    Self::String8Value(value)
                }
                crate::types::xtypes::TK_STRING16 => {
                    let mut value = <::std::string::String>::default();
                    state.decode_variant(&MEMBER_INFO[15], &mut value)?;
                    Self::String16Value(value)
                }
                _ => {
                    let mut value =
                        <crate::types::xtypes::ExtendedAnnotationParameterValue>::default();
                    state.decode_variant(&MEMBER_INFO[16], &mut value)?;
                    Self::ExtendedValue(value)
                }
            };
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AppliedAnnotationParameter {
    pub paramname_hash: crate::types::xtypes::NameHash,
    pub value: crate::types::xtypes::AnnotationParameterValue,
}

impl AppliedAnnotationParameter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            paramname_hash: <crate::types::xtypes::NameHash>::default(),
            value: <crate::types::xtypes::AnnotationParameterValue>::default(),
        }
    }
}

impl ::std::default::Default for AppliedAnnotationParameter {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for AppliedAnnotationParameter {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::AppliedAnnotationParameter",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "paramname_hash",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::NameHash>(),
        },
        ::intercom_cts::MemberInfo {
            name: "value",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationParameterValue>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for AppliedAnnotationParameter {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.paramname_hash)?;
            state.encode_field(&MEMBER_INFO[1], &self.value)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for AppliedAnnotationParameter {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.paramname_hash)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.value)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type AppliedAnnotationParameterSeq =
    ::std::vec::Vec<crate::types::xtypes::AppliedAnnotationParameter>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AppliedAnnotation {
    pub annotation_typeid: crate::types::xtypes::TypeIdentifier,
    pub param_seq: ::std::option::Option<crate::types::xtypes::AppliedAnnotationParameterSeq>,
}

impl AppliedAnnotation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            annotation_typeid: <crate::types::xtypes::TypeIdentifier>::default(),
            param_seq: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for AppliedAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for AppliedAnnotation {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::AppliedAnnotation",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "annotation_typeid",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "param_seq",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::AppliedAnnotationParameterSeq,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for AppliedAnnotation {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.annotation_typeid)?;
            state.encode_optional(&MEMBER_INFO[1], &self.param_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for AppliedAnnotation {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.annotation_typeid)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.param_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type AppliedAnnotationSeq = ::std::vec::Vec<crate::types::xtypes::AppliedAnnotation>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AppliedVerbatimAnnotation {
    pub placement: ::std::string::String,
    pub language: ::std::string::String,
    pub text: ::std::string::String,
}

impl AppliedVerbatimAnnotation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            placement: <::std::string::String>::default(),
            language: <::std::string::String>::default(),
            text: <::std::string::String>::default(),
        }
    }
}

impl ::std::default::Default for AppliedVerbatimAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for AppliedVerbatimAnnotation {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::AppliedVerbatimAnnotation",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "placement",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
        ::intercom_cts::MemberInfo {
            name: "language",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
        ::intercom_cts::MemberInfo {
            name: "text",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
    ];

    impl ::intercom_cts::Marshal for AppliedVerbatimAnnotation {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.placement)?;
            state.encode_field(&MEMBER_INFO[1], &self.language)?;
            state.encode_field(&MEMBER_INFO[2], &self.text)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for AppliedVerbatimAnnotation {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.placement)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.language)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.text)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AppliedBuiltinMemberAnnotations {
    pub unit: ::std::option::Option<::std::string::String>,
    pub min: ::std::option::Option<crate::types::xtypes::AnnotationParameterValue>,
    pub max: ::std::option::Option<crate::types::xtypes::AnnotationParameterValue>,
    pub hash_id: ::std::option::Option<::std::string::String>,
}

impl AppliedBuiltinMemberAnnotations {
    #[must_use]
    pub fn new() -> Self {
        Self {
            unit: ::std::option::Option::None,
            min: ::std::option::Option::None,
            max: ::std::option::Option::None,
            hash_id: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for AppliedBuiltinMemberAnnotations {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for AppliedBuiltinMemberAnnotations {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::AppliedBuiltinMemberAnnotations",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "unit",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
        ::intercom_cts::MemberInfo {
            name: "min",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationParameterValue>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "max",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationParameterValue>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "hash_id",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
    ];

    impl ::intercom_cts::Marshal for AppliedBuiltinMemberAnnotations {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_optional(&MEMBER_INFO[0], &self.unit)?;
            state.encode_optional(&MEMBER_INFO[1], &self.min)?;
            state.encode_optional(&MEMBER_INFO[2], &self.max)?;
            state.encode_optional(&MEMBER_INFO[3], &self.hash_id)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for AppliedBuiltinMemberAnnotations {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.unit)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.min)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.max)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.hash_id)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonStructMember {
    pub member_id: crate::types::xtypes::MemberId,
    pub member_flags: crate::types::xtypes::StructMemberFlag,
    pub member_type_id: crate::types::xtypes::TypeIdentifier,
}

impl CommonStructMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            member_id: <crate::types::xtypes::MemberId>::default(),
            member_flags: <crate::types::xtypes::StructMemberFlag>::default(),
            member_type_id: <crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for CommonStructMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonStructMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonStructMember",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "member_id",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberId>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_flags",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::StructMemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_type_id",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonStructMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.member_id)?;
            state.encode_field(&MEMBER_INFO[1], &self.member_flags)?;
            state.encode_field(&MEMBER_INFO[2], &self.member_type_id)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonStructMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.member_id)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.member_flags)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.member_type_id)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteMemberDetail {
    pub name: crate::types::xtypes::MemberName,
    pub ann_builtin: ::std::option::Option<crate::types::xtypes::AppliedBuiltinMemberAnnotations>,
    pub ann_custom: ::std::option::Option<crate::types::xtypes::AppliedAnnotationSeq>,
}

impl CompleteMemberDetail {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: <crate::types::xtypes::MemberName>::default(),
            ann_builtin: ::std::option::Option::None,
            ann_custom: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for CompleteMemberDetail {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteMemberDetail {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteMemberDetail",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "name",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberName>(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_builtin",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::AppliedBuiltinMemberAnnotations,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_custom",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AppliedAnnotationSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteMemberDetail {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.name)?;
            state.encode_optional(&MEMBER_INFO[1], &self.ann_builtin)?;
            state.encode_optional(&MEMBER_INFO[2], &self.ann_custom)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteMemberDetail {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.name)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.ann_builtin)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.ann_custom)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalMemberDetail {
    pub name_hash: crate::types::xtypes::NameHash,
}

impl MinimalMemberDetail {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name_hash: <crate::types::xtypes::NameHash>::default(),
        }
    }
}

impl ::std::default::Default for MinimalMemberDetail {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalMemberDetail {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalMemberDetail",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "name_hash",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::NameHash>(),
    }];

    impl ::intercom_cts::Marshal for MinimalMemberDetail {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.name_hash)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalMemberDetail {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.name_hash)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteStructMember {
    pub common: crate::types::xtypes::CommonStructMember,
    pub detail: crate::types::xtypes::CompleteMemberDetail,
}

impl CompleteStructMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonStructMember>::default(),
            detail: <crate::types::xtypes::CompleteMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteStructMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteStructMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteStructMember",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonStructMember>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteStructMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteStructMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteStructMemberSeq = ::std::vec::Vec<crate::types::xtypes::CompleteStructMember>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalStructMember {
    pub common: crate::types::xtypes::CommonStructMember,
    pub detail: crate::types::xtypes::MinimalMemberDetail,
}

impl MinimalStructMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonStructMember>::default(),
            detail: <crate::types::xtypes::MinimalMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for MinimalStructMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalStructMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalStructMember",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonStructMember>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalStructMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalStructMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type MinimalStructMemberSeq = ::std::vec::Vec<crate::types::xtypes::MinimalStructMember>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AppliedBuiltinTypeAnnotations {
    pub verbatim: ::std::option::Option<crate::types::xtypes::AppliedVerbatimAnnotation>,
}

impl AppliedBuiltinTypeAnnotations {
    #[must_use]
    pub fn new() -> Self {
        Self {
            verbatim: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for AppliedBuiltinTypeAnnotations {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for AppliedBuiltinTypeAnnotations {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::AppliedBuiltinTypeAnnotations",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "verbatim",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::AppliedVerbatimAnnotation>(),
    }];

    impl ::intercom_cts::Marshal for AppliedBuiltinTypeAnnotations {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_optional(&MEMBER_INFO[0], &self.verbatim)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for AppliedBuiltinTypeAnnotations {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.verbatim)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalTypeDetail {}

impl MinimalTypeDetail {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for MinimalTypeDetail {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalTypeDetail {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalTypeDetail",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for MinimalTypeDetail {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalTypeDetail {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteTypeDetail {
    pub ann_builtin: ::std::option::Option<crate::types::xtypes::AppliedBuiltinTypeAnnotations>,
    pub ann_custom: ::std::option::Option<crate::types::xtypes::AppliedAnnotationSeq>,
    pub type_name: crate::types::xtypes::QualifiedTypeName,
}

impl CompleteTypeDetail {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ann_builtin: ::std::option::Option::None,
            ann_custom: ::std::option::Option::None,
            type_name: <crate::types::xtypes::QualifiedTypeName>::default(),
        }
    }
}

impl ::std::default::Default for CompleteTypeDetail {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteTypeDetail {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteTypeDetail",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "ann_builtin",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::AppliedBuiltinTypeAnnotations,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_custom",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AppliedAnnotationSeq>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_name",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::QualifiedTypeName>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteTypeDetail {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_optional(&MEMBER_INFO[0], &self.ann_builtin)?;
            state.encode_optional(&MEMBER_INFO[1], &self.ann_custom)?;
            state.encode_field(&MEMBER_INFO[2], &self.type_name)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteTypeDetail {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.ann_builtin)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.ann_custom)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.type_name)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteStructHeader {
    pub base_type: crate::types::xtypes::TypeIdentifier,
    pub detail: crate::types::xtypes::CompleteTypeDetail,
}

impl CompleteStructHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_type: <crate::types::xtypes::TypeIdentifier>::default(),
            detail: <crate::types::xtypes::CompleteTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteStructHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteStructHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteStructHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "base_type",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteStructHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.base_type)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteStructHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.base_type)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalStructHeader {
    pub base_type: crate::types::xtypes::TypeIdentifier,
    pub detail: crate::types::xtypes::MinimalTypeDetail,
}

impl MinimalStructHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_type: <crate::types::xtypes::TypeIdentifier>::default(),
            detail: <crate::types::xtypes::MinimalTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for MinimalStructHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalStructHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalStructHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "base_type",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalStructHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.base_type)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalStructHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.base_type)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteStructType {
    pub struct_flags: crate::types::xtypes::StructTypeFlag,
    pub header: crate::types::xtypes::CompleteStructHeader,
    pub member_seq: crate::types::xtypes::CompleteStructMemberSeq,
}

impl CompleteStructType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            struct_flags: <crate::types::xtypes::StructTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteStructHeader>::default(),
            member_seq: <crate::types::xtypes::CompleteStructMemberSeq>::default(),
        }
    }
}

impl ::std::default::Default for CompleteStructType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteStructType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteStructType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "struct_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::StructTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteStructHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteStructMemberSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteStructType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.struct_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.member_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteStructType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.struct_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.member_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalStructType {
    pub struct_flags: crate::types::xtypes::StructTypeFlag,
    pub header: crate::types::xtypes::MinimalStructHeader,
    pub member_seq: crate::types::xtypes::MinimalStructMemberSeq,
}

impl MinimalStructType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            struct_flags: <crate::types::xtypes::StructTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalStructHeader>::default(),
            member_seq: <crate::types::xtypes::MinimalStructMemberSeq>::default(),
        }
    }
}

impl ::std::default::Default for MinimalStructType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalStructType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalStructType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "struct_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::StructTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalStructHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalStructMemberSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalStructType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.struct_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.member_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalStructType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.struct_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.member_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type UnionCaseLabelSeq = ::std::vec::Vec<i32>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonUnionMember {
    pub member_id: crate::types::xtypes::MemberId,
    pub member_flags: crate::types::xtypes::UnionMemberFlag,
    pub type_id: crate::types::xtypes::TypeIdentifier,
    pub label_seq: crate::types::xtypes::UnionCaseLabelSeq,
}

impl CommonUnionMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            member_id: <crate::types::xtypes::MemberId>::default(),
            member_flags: <crate::types::xtypes::UnionMemberFlag>::default(),
            type_id: <crate::types::xtypes::TypeIdentifier>::default(),
            label_seq: <crate::types::xtypes::UnionCaseLabelSeq>::default(),
        }
    }
}

impl ::std::default::Default for CommonUnionMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonUnionMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonUnionMember",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "member_id",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberId>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_flags",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::UnionMemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_id",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "label_seq",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::UnionCaseLabelSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonUnionMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.member_id)?;
            state.encode_field(&MEMBER_INFO[1], &self.member_flags)?;
            state.encode_field(&MEMBER_INFO[2], &self.type_id)?;
            state.encode_field(&MEMBER_INFO[3], &self.label_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonUnionMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.member_id)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.member_flags)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.type_id)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.label_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteUnionMember {
    pub common: crate::types::xtypes::CommonUnionMember,
    pub detail: crate::types::xtypes::CompleteMemberDetail,
}

impl CompleteUnionMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonUnionMember>::default(),
            detail: <crate::types::xtypes::CompleteMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteUnionMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteUnionMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteUnionMember",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonUnionMember>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteUnionMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteUnionMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteUnionMemberSeq = ::std::vec::Vec<crate::types::xtypes::CompleteUnionMember>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalUnionMember {
    pub common: crate::types::xtypes::CommonUnionMember,
    pub detail: crate::types::xtypes::MinimalMemberDetail,
}

impl MinimalUnionMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonUnionMember>::default(),
            detail: <crate::types::xtypes::MinimalMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for MinimalUnionMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalUnionMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalUnionMember",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonUnionMember>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalUnionMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalUnionMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type MinimalUnionMemberSeq = ::std::vec::Vec<crate::types::xtypes::MinimalUnionMember>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonDiscriminatorMember {
    pub member_flags: crate::types::xtypes::UnionDiscriminatorFlag,
    pub type_id: crate::types::xtypes::TypeIdentifier,
}

impl CommonDiscriminatorMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            member_flags: <crate::types::xtypes::UnionDiscriminatorFlag>::default(),
            type_id: <crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for CommonDiscriminatorMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonDiscriminatorMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonDiscriminatorMember",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "member_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::UnionDiscriminatorFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_id",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonDiscriminatorMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.member_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.type_id)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonDiscriminatorMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.member_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.type_id)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteDiscriminatorMember {
    pub common: crate::types::xtypes::CommonDiscriminatorMember,
    pub ann_builtin: ::std::option::Option<crate::types::xtypes::AppliedBuiltinTypeAnnotations>,
    pub ann_custom: ::std::option::Option<crate::types::xtypes::AppliedAnnotationSeq>,
}

impl CompleteDiscriminatorMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonDiscriminatorMember>::default(),
            ann_builtin: ::std::option::Option::None,
            ann_custom: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for CompleteDiscriminatorMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteDiscriminatorMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteDiscriminatorMember",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonDiscriminatorMember>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_builtin",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::AppliedBuiltinTypeAnnotations,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_custom",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AppliedAnnotationSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteDiscriminatorMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_optional(&MEMBER_INFO[1], &self.ann_builtin)?;
            state.encode_optional(&MEMBER_INFO[2], &self.ann_custom)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteDiscriminatorMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.ann_builtin)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.ann_custom)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalDiscriminatorMember {
    pub common: crate::types::xtypes::CommonDiscriminatorMember,
}

impl MinimalDiscriminatorMember {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonDiscriminatorMember>::default(),
        }
    }
}

impl ::std::default::Default for MinimalDiscriminatorMember {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalDiscriminatorMember {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalDiscriminatorMember",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "common",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonDiscriminatorMember>(),
    }];

    impl ::intercom_cts::Marshal for MinimalDiscriminatorMember {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalDiscriminatorMember {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteUnionHeader {
    pub base_type: crate::types::xtypes::TypeIdentifier,
    pub detail: crate::types::xtypes::CompleteTypeDetail,
}

impl CompleteUnionHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_type: <crate::types::xtypes::TypeIdentifier>::default(),
            detail: <crate::types::xtypes::CompleteTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteUnionHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteUnionHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteUnionHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "base_type",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteUnionHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.base_type)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteUnionHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.base_type)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalUnionHeader {
    pub base_type: crate::types::xtypes::TypeIdentifier,
    pub detail: crate::types::xtypes::MinimalTypeDetail,
}

impl MinimalUnionHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_type: <crate::types::xtypes::TypeIdentifier>::default(),
            detail: <crate::types::xtypes::MinimalTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for MinimalUnionHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalUnionHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalUnionHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "base_type",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalUnionHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.base_type)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalUnionHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.base_type)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteUnionType {
    pub union_flags: crate::types::xtypes::UnionTypeFlag,
    pub header: crate::types::xtypes::CompleteUnionHeader,
    pub discriminator: crate::types::xtypes::CompleteDiscriminatorMember,
    pub member_seq: crate::types::xtypes::CompleteUnionMemberSeq,
}

impl CompleteUnionType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            union_flags: <crate::types::xtypes::UnionTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteUnionHeader>::default(),
            discriminator: <crate::types::xtypes::CompleteDiscriminatorMember>::default(),
            member_seq: <crate::types::xtypes::CompleteUnionMemberSeq>::default(),
        }
    }
}

impl ::std::default::Default for CompleteUnionType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteUnionType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteUnionType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "union_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::UnionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteUnionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "discriminator",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteDiscriminatorMember>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "member_seq",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteUnionMemberSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteUnionType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.union_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.discriminator)?;
            state.encode_field(&MEMBER_INFO[3], &self.member_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteUnionType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.union_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.discriminator)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.member_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalUnionType {
    pub union_flags: crate::types::xtypes::UnionTypeFlag,
    pub header: crate::types::xtypes::MinimalUnionHeader,
    pub discriminator: crate::types::xtypes::MinimalDiscriminatorMember,
    pub member_seq: crate::types::xtypes::MinimalUnionMemberSeq,
}

impl MinimalUnionType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            union_flags: <crate::types::xtypes::UnionTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalUnionHeader>::default(),
            discriminator: <crate::types::xtypes::MinimalDiscriminatorMember>::default(),
            member_seq: <crate::types::xtypes::MinimalUnionMemberSeq>::default(),
        }
    }
}

impl ::std::default::Default for MinimalUnionType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalUnionType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalUnionType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "union_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::UnionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalUnionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "discriminator",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalDiscriminatorMember>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "member_seq",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalUnionMemberSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalUnionType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.union_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.discriminator)?;
            state.encode_field(&MEMBER_INFO[3], &self.member_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalUnionType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.union_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.discriminator)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.member_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonAnnotationParameter {
    pub member_flags: crate::types::xtypes::AnnotationParameterFlag,
    pub member_type_id: crate::types::xtypes::TypeIdentifier,
}

impl CommonAnnotationParameter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            member_flags: <crate::types::xtypes::AnnotationParameterFlag>::default(),
            member_type_id: <crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for CommonAnnotationParameter {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonAnnotationParameter {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonAnnotationParameter",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "member_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationParameterFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_type_id",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonAnnotationParameter {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.member_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.member_type_id)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonAnnotationParameter {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.member_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.member_type_id)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteAnnotationParameter {
    pub common: crate::types::xtypes::CommonAnnotationParameter,
    pub name: crate::types::xtypes::MemberName,
    pub default_value: crate::types::xtypes::AnnotationParameterValue,
}

impl CompleteAnnotationParameter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonAnnotationParameter>::default(),
            name: <crate::types::xtypes::MemberName>::default(),
            default_value: <crate::types::xtypes::AnnotationParameterValue>::default(),
        }
    }
}

impl ::std::default::Default for CompleteAnnotationParameter {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteAnnotationParameter {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteAnnotationParameter",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonAnnotationParameter>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "name",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MemberName>(),
        },
        ::intercom_cts::MemberInfo {
            name: "default_value",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationParameterValue>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteAnnotationParameter {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.name)?;
            state.encode_field(&MEMBER_INFO[2], &self.default_value)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteAnnotationParameter {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.name)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.default_value)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteAnnotationParameterSeq =
    ::std::vec::Vec<crate::types::xtypes::CompleteAnnotationParameter>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MinimalAnnotationParameter {
    pub common: crate::types::xtypes::CommonAnnotationParameter,
    pub name_hash: crate::types::xtypes::NameHash,
    pub default_value: crate::types::xtypes::AnnotationParameterValue,
}

impl MinimalAnnotationParameter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonAnnotationParameter>::default(),
            name_hash: <crate::types::xtypes::NameHash>::default(),
            default_value: <crate::types::xtypes::AnnotationParameterValue>::default(),
        }
    }
}

impl ::std::default::Default for MinimalAnnotationParameter {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalAnnotationParameter {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalAnnotationParameter",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonAnnotationParameter>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "name_hash",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::NameHash>(),
        },
        ::intercom_cts::MemberInfo {
            name: "default_value",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationParameterValue>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalAnnotationParameter {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.name_hash)?;
            state.encode_field(&MEMBER_INFO[2], &self.default_value)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalAnnotationParameter {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.name_hash)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.default_value)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type MinimalAnnotationParameterSeq =
    ::std::vec::Vec<crate::types::xtypes::MinimalAnnotationParameter>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CompleteAnnotationHeader {
    pub annotation_name: crate::types::xtypes::QualifiedTypeName,
}

impl CompleteAnnotationHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            annotation_name: <crate::types::xtypes::QualifiedTypeName>::default(),
        }
    }
}

impl ::std::default::Default for CompleteAnnotationHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteAnnotationHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteAnnotationHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "annotation_name",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::QualifiedTypeName>(),
    }];

    impl ::intercom_cts::Marshal for CompleteAnnotationHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.annotation_name)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteAnnotationHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.annotation_name)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalAnnotationHeader {}

impl MinimalAnnotationHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for MinimalAnnotationHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalAnnotationHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalAnnotationHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for MinimalAnnotationHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalAnnotationHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteAnnotationType {
    pub annotation_flag: crate::types::xtypes::AnnotationTypeFlag,
    pub header: crate::types::xtypes::CompleteAnnotationHeader,
    pub member_seq: crate::types::xtypes::CompleteAnnotationParameterSeq,
}

impl CompleteAnnotationType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            annotation_flag: <crate::types::xtypes::AnnotationTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteAnnotationHeader>::default(),
            member_seq: <crate::types::xtypes::CompleteAnnotationParameterSeq>::default(),
        }
    }
}

impl ::std::default::Default for CompleteAnnotationType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteAnnotationType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteAnnotationType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "annotation_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteAnnotationHeader>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "member_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::CompleteAnnotationParameterSeq,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteAnnotationType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.annotation_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.member_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteAnnotationType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.annotation_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.member_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MinimalAnnotationType {
    pub annotation_flag: crate::types::xtypes::AnnotationTypeFlag,
    pub header: crate::types::xtypes::MinimalAnnotationHeader,
    pub member_seq: crate::types::xtypes::MinimalAnnotationParameterSeq,
}

impl MinimalAnnotationType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            annotation_flag: <crate::types::xtypes::AnnotationTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalAnnotationHeader>::default(),
            member_seq: <crate::types::xtypes::MinimalAnnotationParameterSeq>::default(),
        }
    }
}

impl ::std::default::Default for MinimalAnnotationType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalAnnotationType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalAnnotationType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "annotation_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AnnotationTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalAnnotationHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "member_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::MinimalAnnotationParameterSeq,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalAnnotationType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.annotation_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.member_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalAnnotationType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.annotation_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.member_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonAliasBody {
    pub related_flags: crate::types::xtypes::AliasMemberFlag,
    pub related_type: crate::types::xtypes::TypeIdentifier,
}

impl CommonAliasBody {
    #[must_use]
    pub fn new() -> Self {
        Self {
            related_flags: <crate::types::xtypes::AliasMemberFlag>::default(),
            related_type: <crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for CommonAliasBody {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonAliasBody {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonAliasBody",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "related_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AliasMemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "related_type",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonAliasBody {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.related_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.related_type)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonAliasBody {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.related_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.related_type)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteAliasBody {
    pub common: crate::types::xtypes::CommonAliasBody,
    pub ann_builtin: ::std::option::Option<crate::types::xtypes::AppliedBuiltinMemberAnnotations>,
    pub ann_custom: ::std::option::Option<crate::types::xtypes::AppliedAnnotationSeq>,
}

impl CompleteAliasBody {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonAliasBody>::default(),
            ann_builtin: ::std::option::Option::None,
            ann_custom: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for CompleteAliasBody {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteAliasBody {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteAliasBody",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonAliasBody>(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_builtin",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::AppliedBuiltinMemberAnnotations,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_custom",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AppliedAnnotationSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteAliasBody {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_optional(&MEMBER_INFO[1], &self.ann_builtin)?;
            state.encode_optional(&MEMBER_INFO[2], &self.ann_custom)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteAliasBody {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.ann_builtin)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.ann_custom)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalAliasBody {
    pub common: crate::types::xtypes::CommonAliasBody,
}

impl MinimalAliasBody {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonAliasBody>::default(),
        }
    }
}

impl ::std::default::Default for MinimalAliasBody {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalAliasBody {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalAliasBody",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "common",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonAliasBody>(),
    }];

    impl ::intercom_cts::Marshal for MinimalAliasBody {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalAliasBody {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteAliasHeader {
    pub detail: crate::types::xtypes::CompleteTypeDetail,
}

impl CompleteAliasHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            detail: <crate::types::xtypes::CompleteTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteAliasHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteAliasHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteAliasHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "detail",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
    }];

    impl ::intercom_cts::Marshal for CompleteAliasHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteAliasHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalAliasHeader {}

impl MinimalAliasHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for MinimalAliasHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalAliasHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalAliasHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for MinimalAliasHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalAliasHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteAliasType {
    pub alias_flags: crate::types::xtypes::AliasTypeFlag,
    pub header: crate::types::xtypes::CompleteAliasHeader,
    pub body: crate::types::xtypes::CompleteAliasBody,
}

impl CompleteAliasType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            alias_flags: <crate::types::xtypes::AliasTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteAliasHeader>::default(),
            body: <crate::types::xtypes::CompleteAliasBody>::default(),
        }
    }
}

impl ::std::default::Default for CompleteAliasType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteAliasType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteAliasType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "alias_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AliasTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteAliasHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "body",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteAliasBody>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteAliasType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.alias_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.body)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteAliasType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.alias_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.body)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalAliasType {
    pub alias_flags: crate::types::xtypes::AliasTypeFlag,
    pub header: crate::types::xtypes::MinimalAliasHeader,
    pub body: crate::types::xtypes::MinimalAliasBody,
}

impl MinimalAliasType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            alias_flags: <crate::types::xtypes::AliasTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalAliasHeader>::default(),
            body: <crate::types::xtypes::MinimalAliasBody>::default(),
        }
    }
}

impl ::std::default::Default for MinimalAliasType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalAliasType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalAliasType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "alias_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AliasTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalAliasHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "body",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalAliasBody>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalAliasType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.alias_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.body)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalAliasType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.alias_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.body)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteElementDetail {
    pub ann_builtin: ::std::option::Option<crate::types::xtypes::AppliedBuiltinMemberAnnotations>,
    pub ann_custom: ::std::option::Option<crate::types::xtypes::AppliedAnnotationSeq>,
}

impl CompleteElementDetail {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ann_builtin: ::std::option::Option::None,
            ann_custom: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for CompleteElementDetail {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteElementDetail {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteElementDetail",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "ann_builtin",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::AppliedBuiltinMemberAnnotations,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "ann_custom",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::AppliedAnnotationSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteElementDetail {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_optional(&MEMBER_INFO[0], &self.ann_builtin)?;
            state.encode_optional(&MEMBER_INFO[1], &self.ann_custom)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteElementDetail {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.ann_builtin)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.ann_custom)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonCollectionElement {
    pub element_flags: crate::types::xtypes::CollectionElementFlag,
    pub type_: crate::types::xtypes::TypeIdentifier,
}

impl CommonCollectionElement {
    #[must_use]
    pub fn new() -> Self {
        Self {
            element_flags: <crate::types::xtypes::CollectionElementFlag>::default(),
            type_: <crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for CommonCollectionElement {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonCollectionElement {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonCollectionElement",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "element_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionElementFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonCollectionElement {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.element_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.type_)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonCollectionElement {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.element_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.type_)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteCollectionElement {
    pub common: crate::types::xtypes::CommonCollectionElement,
    pub detail: crate::types::xtypes::CompleteElementDetail,
}

impl CompleteCollectionElement {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonCollectionElement>::default(),
            detail: <crate::types::xtypes::CompleteElementDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteCollectionElement {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteCollectionElement {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteCollectionElement",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonCollectionElement>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteElementDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteCollectionElement {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteCollectionElement {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalCollectionElement {
    pub common: crate::types::xtypes::CommonCollectionElement,
}

impl MinimalCollectionElement {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for MinimalCollectionElement {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalCollectionElement {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalCollectionElement",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "common",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonCollectionElement>(),
    }];

    impl ::intercom_cts::Marshal for MinimalCollectionElement {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalCollectionElement {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonCollectionHeader {
    pub bound: crate::types::xtypes::LBound,
}

impl CommonCollectionHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bound: <crate::types::xtypes::LBound>::default(),
        }
    }
}

impl ::std::default::Default for CommonCollectionHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonCollectionHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonCollectionHeader",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "bound",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::LBound>(),
    }];

    impl ::intercom_cts::Marshal for CommonCollectionHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bound)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonCollectionHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bound)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteCollectionHeader {
    pub common: crate::types::xtypes::CommonCollectionHeader,
    pub detail: ::std::option::Option<crate::types::xtypes::CompleteTypeDetail>,
}

impl CompleteCollectionHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonCollectionHeader>::default(),
            detail: ::std::option::Option::None,
        }
    }
}

impl ::std::default::Default for CompleteCollectionHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteCollectionHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteCollectionHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::IS_OPTIONAL,
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteCollectionHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_optional(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteCollectionHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalCollectionHeader {
    pub common: crate::types::xtypes::CommonCollectionHeader,
}

impl MinimalCollectionHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonCollectionHeader>::default(),
        }
    }
}

impl ::std::default::Default for MinimalCollectionHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalCollectionHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalCollectionHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "common",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonCollectionHeader>(),
    }];

    impl ::intercom_cts::Marshal for MinimalCollectionHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalCollectionHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteSequenceType {
    pub collection_flag: crate::types::xtypes::CollectionTypeFlag,
    pub header: crate::types::xtypes::CompleteCollectionHeader,
    pub element: crate::types::xtypes::CompleteCollectionElement,
}

impl CompleteSequenceType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_flag: <crate::types::xtypes::CollectionTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteCollectionHeader>::default(),
            element: <crate::types::xtypes::CompleteCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for CompleteSequenceType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteSequenceType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteSequenceType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "collection_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteCollectionHeader>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "element",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteCollectionElement>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteSequenceType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.collection_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.element)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteSequenceType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.collection_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalSequenceType {
    pub collection_flag: crate::types::xtypes::CollectionTypeFlag,
    pub header: crate::types::xtypes::MinimalCollectionHeader,
    pub element: crate::types::xtypes::MinimalCollectionElement,
}

impl MinimalSequenceType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_flag: <crate::types::xtypes::CollectionTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalCollectionHeader>::default(),
            element: <crate::types::xtypes::MinimalCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for MinimalSequenceType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalSequenceType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalSequenceType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "collection_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalCollectionElement>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalSequenceType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.collection_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.element)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalSequenceType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.collection_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonArrayHeader {
    pub bound_seq: crate::types::xtypes::LBoundSeq,
}

impl CommonArrayHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bound_seq: <crate::types::xtypes::LBoundSeq>::default(),
        }
    }
}

impl ::std::default::Default for CommonArrayHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonArrayHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonArrayHeader",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "bound_seq",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::LBoundSeq>(),
    }];

    impl ::intercom_cts::Marshal for CommonArrayHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bound_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonArrayHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bound_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteArrayHeader {
    pub common: crate::types::xtypes::CommonArrayHeader,
    pub detail: crate::types::xtypes::CompleteTypeDetail,
}

impl CompleteArrayHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonArrayHeader>::default(),
            detail: <crate::types::xtypes::CompleteTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteArrayHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteArrayHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteArrayHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonArrayHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteArrayHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteArrayHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalArrayHeader {
    pub common: crate::types::xtypes::CommonArrayHeader,
}

impl MinimalArrayHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonArrayHeader>::default(),
        }
    }
}

impl ::std::default::Default for MinimalArrayHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalArrayHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalArrayHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "common",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonArrayHeader>(),
    }];

    impl ::intercom_cts::Marshal for MinimalArrayHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalArrayHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteArrayType {
    pub collection_flag: crate::types::xtypes::CollectionTypeFlag,
    pub header: crate::types::xtypes::CompleteArrayHeader,
    pub element: crate::types::xtypes::CompleteCollectionElement,
}

impl CompleteArrayType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_flag: <crate::types::xtypes::CollectionTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteArrayHeader>::default(),
            element: <crate::types::xtypes::CompleteCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for CompleteArrayType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteArrayType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteArrayType",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "collection_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteArrayHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteCollectionElement>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteArrayType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.collection_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.element)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteArrayType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.collection_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalArrayType {
    pub collection_flag: crate::types::xtypes::CollectionTypeFlag,
    pub header: crate::types::xtypes::MinimalArrayHeader,
    pub element: crate::types::xtypes::MinimalCollectionElement,
}

impl MinimalArrayType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_flag: <crate::types::xtypes::CollectionTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalArrayHeader>::default(),
            element: <crate::types::xtypes::MinimalCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for MinimalArrayType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalArrayType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalArrayType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "collection_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalArrayHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "element",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalCollectionElement>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalArrayType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.collection_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.element)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalArrayType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.collection_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.element)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteMapType {
    pub collection_flag: crate::types::xtypes::CollectionTypeFlag,
    pub header: crate::types::xtypes::CompleteCollectionHeader,
    pub key: crate::types::xtypes::CompleteCollectionElement,
    pub element: crate::types::xtypes::CompleteCollectionElement,
}

impl CompleteMapType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_flag: <crate::types::xtypes::CollectionTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteCollectionHeader>::default(),
            key: <crate::types::xtypes::CompleteCollectionElement>::default(),
            element: <crate::types::xtypes::CompleteCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for CompleteMapType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteMapType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteMapType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "collection_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteCollectionHeader>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "key",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteCollectionElement>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "element",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteCollectionElement>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteMapType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.collection_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.key)?;
            state.encode_field(&MEMBER_INFO[3], &self.element)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteMapType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.collection_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.key)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.element)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalMapType {
    pub collection_flag: crate::types::xtypes::CollectionTypeFlag,
    pub header: crate::types::xtypes::MinimalCollectionHeader,
    pub key: crate::types::xtypes::MinimalCollectionElement,
    pub element: crate::types::xtypes::MinimalCollectionElement,
}

impl MinimalMapType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_flag: <crate::types::xtypes::CollectionTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalCollectionHeader>::default(),
            key: <crate::types::xtypes::MinimalCollectionElement>::default(),
            element: <crate::types::xtypes::MinimalCollectionElement>::default(),
        }
    }
}

impl ::std::default::Default for MinimalMapType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalMapType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalMapType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "collection_flag",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CollectionTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalCollectionHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "key",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalCollectionElement>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "element",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalCollectionElement>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalMapType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.collection_flag)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.key)?;
            state.encode_field(&MEMBER_INFO[3], &self.element)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalMapType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.collection_flag)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.key)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.element)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type BitBound = u16;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonEnumeratedLiteral {
    pub value: i32,
    pub flags: crate::types::xtypes::EnumeratedLiteralFlag,
}

impl CommonEnumeratedLiteral {
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: 0,
            flags: <crate::types::xtypes::EnumeratedLiteralFlag>::default(),
        }
    }
}

impl ::std::default::Default for CommonEnumeratedLiteral {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonEnumeratedLiteral {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonEnumeratedLiteral",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "value",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "flags",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::EnumeratedLiteralFlag>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonEnumeratedLiteral {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.value)?;
            state.encode_field(&MEMBER_INFO[1], &self.flags)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonEnumeratedLiteral {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.value)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.flags)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteEnumeratedLiteral {
    pub common: crate::types::xtypes::CommonEnumeratedLiteral,
    pub detail: crate::types::xtypes::CompleteMemberDetail,
}

impl CompleteEnumeratedLiteral {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonEnumeratedLiteral>::default(),
            detail: <crate::types::xtypes::CompleteMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteEnumeratedLiteral {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteEnumeratedLiteral {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteEnumeratedLiteral",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonEnumeratedLiteral>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteEnumeratedLiteral {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteEnumeratedLiteral {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteEnumeratedLiteralSeq =
    ::std::vec::Vec<crate::types::xtypes::CompleteEnumeratedLiteral>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalEnumeratedLiteral {
    pub common: crate::types::xtypes::CommonEnumeratedLiteral,
    pub detail: crate::types::xtypes::MinimalMemberDetail,
}

impl MinimalEnumeratedLiteral {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonEnumeratedLiteral>::default(),
            detail: <crate::types::xtypes::MinimalMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for MinimalEnumeratedLiteral {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalEnumeratedLiteral {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalEnumeratedLiteral",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonEnumeratedLiteral>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalEnumeratedLiteral {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalEnumeratedLiteral {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type MinimalEnumeratedLiteralSeq =
    ::std::vec::Vec<crate::types::xtypes::MinimalEnumeratedLiteral>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonEnumeratedHeader {
    pub bit_bound: crate::types::xtypes::BitBound,
}

impl CommonEnumeratedHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bit_bound: <crate::types::xtypes::BitBound>::default(),
        }
    }
}

impl ::std::default::Default for CommonEnumeratedHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonEnumeratedHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonEnumeratedHeader",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "bit_bound",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitBound>(),
    }];

    impl ::intercom_cts::Marshal for CommonEnumeratedHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bit_bound)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonEnumeratedHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bit_bound)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteEnumeratedHeader {
    pub common: crate::types::xtypes::CommonEnumeratedHeader,
    pub detail: crate::types::xtypes::CompleteTypeDetail,
}

impl CompleteEnumeratedHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonEnumeratedHeader>::default(),
            detail: <crate::types::xtypes::CompleteTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteEnumeratedHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteEnumeratedHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteEnumeratedHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonEnumeratedHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteEnumeratedHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteEnumeratedHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalEnumeratedHeader {
    pub common: crate::types::xtypes::CommonEnumeratedHeader,
}

impl MinimalEnumeratedHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonEnumeratedHeader>::default(),
        }
    }
}

impl ::std::default::Default for MinimalEnumeratedHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalEnumeratedHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalEnumeratedHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "common",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonEnumeratedHeader>(),
    }];

    impl ::intercom_cts::Marshal for MinimalEnumeratedHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalEnumeratedHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteEnumeratedType {
    pub enum_flags: crate::types::xtypes::EnumTypeFlag,
    pub header: crate::types::xtypes::CompleteEnumeratedHeader,
    pub literal_seq: crate::types::xtypes::CompleteEnumeratedLiteralSeq,
}

impl CompleteEnumeratedType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            enum_flags: <crate::types::xtypes::EnumTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteEnumeratedHeader>::default(),
            literal_seq: <crate::types::xtypes::CompleteEnumeratedLiteralSeq>::default(),
        }
    }
}

impl ::std::default::Default for CompleteEnumeratedType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteEnumeratedType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteEnumeratedType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "enum_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::EnumTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteEnumeratedHeader>(
            ),
        },
        ::intercom_cts::MemberInfo {
            name: "literal_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::CompleteEnumeratedLiteralSeq,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteEnumeratedType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.enum_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.literal_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteEnumeratedType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.enum_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.literal_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalEnumeratedType {
    pub enum_flags: crate::types::xtypes::EnumTypeFlag,
    pub header: crate::types::xtypes::MinimalEnumeratedHeader,
    pub literal_seq: crate::types::xtypes::MinimalEnumeratedLiteralSeq,
}

impl MinimalEnumeratedType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            enum_flags: <crate::types::xtypes::EnumTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalEnumeratedHeader>::default(),
            literal_seq: <crate::types::xtypes::MinimalEnumeratedLiteralSeq>::default(),
        }
    }
}

impl ::std::default::Default for MinimalEnumeratedType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalEnumeratedType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalEnumeratedType",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "enum_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::EnumTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalEnumeratedHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "literal_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalEnumeratedLiteralSeq>(
            ),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalEnumeratedType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.enum_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.literal_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalEnumeratedType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.enum_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.literal_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonBitflag {
    pub position: u16,
    pub flags: crate::types::xtypes::BitflagFlag,
}

impl CommonBitflag {
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: 0,
            flags: <crate::types::xtypes::BitflagFlag>::default(),
        }
    }
}

impl ::std::default::Default for CommonBitflag {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonBitflag {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonBitflag",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "position",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u16>(),
        },
        ::intercom_cts::MemberInfo {
            name: "flags",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitflagFlag>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonBitflag {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.position)?;
            state.encode_field(&MEMBER_INFO[1], &self.flags)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonBitflag {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.position)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.flags)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteBitflag {
    pub common: crate::types::xtypes::CommonBitflag,
    pub detail: crate::types::xtypes::CompleteMemberDetail,
}

impl CompleteBitflag {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonBitflag>::default(),
            detail: <crate::types::xtypes::CompleteMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteBitflag {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteBitflag {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteBitflag",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonBitflag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteBitflag {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteBitflag {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteBitflagSeq = ::std::vec::Vec<crate::types::xtypes::CompleteBitflag>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalBitflag {
    pub common: crate::types::xtypes::CommonBitflag,
    pub detail: crate::types::xtypes::MinimalMemberDetail,
}

impl MinimalBitflag {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonBitflag>::default(),
            detail: <crate::types::xtypes::MinimalMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for MinimalBitflag {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalBitflag {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalBitflag",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonBitflag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalBitflag {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalBitflag {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type MinimalBitflagSeq = ::std::vec::Vec<crate::types::xtypes::MinimalBitflag>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonBitmaskHeader {
    pub bit_bound: crate::types::xtypes::BitBound,
}

impl CommonBitmaskHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bit_bound: <crate::types::xtypes::BitBound>::default(),
        }
    }
}

impl ::std::default::Default for CommonBitmaskHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonBitmaskHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonBitmaskHeader",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "bit_bound",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitBound>(),
    }];

    impl ::intercom_cts::Marshal for CommonBitmaskHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bit_bound)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonBitmaskHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bit_bound)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteBitmaskHeader = crate::types::xtypes::CompleteEnumeratedHeader;

pub type MinimalBitmaskHeader = crate::types::xtypes::MinimalEnumeratedHeader;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteBitmaskType {
    pub bitmask_flags: crate::types::xtypes::BitmaskTypeFlag,
    pub header: crate::types::xtypes::CompleteBitmaskHeader,
    pub flag_seq: crate::types::xtypes::CompleteBitflagSeq,
}

impl CompleteBitmaskType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitmask_flags: <crate::types::xtypes::BitmaskTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteBitmaskHeader>::default(),
            flag_seq: <crate::types::xtypes::CompleteBitflagSeq>::default(),
        }
    }
}

impl ::std::default::Default for CompleteBitmaskType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteBitmaskType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteBitmaskType",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "bitmask_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitmaskTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteBitmaskHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "flag_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteBitflagSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteBitmaskType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bitmask_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.flag_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteBitmaskType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bitmask_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.flag_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalBitmaskType {
    pub bitmask_flags: crate::types::xtypes::BitmaskTypeFlag,
    pub header: crate::types::xtypes::MinimalBitmaskHeader,
    pub flag_seq: crate::types::xtypes::MinimalBitflagSeq,
}

impl MinimalBitmaskType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitmask_flags: <crate::types::xtypes::BitmaskTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalBitmaskHeader>::default(),
            flag_seq: <crate::types::xtypes::MinimalBitflagSeq>::default(),
        }
    }
}

impl ::std::default::Default for MinimalBitmaskType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalBitmaskType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalBitmaskType",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "bitmask_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitmaskTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalBitmaskHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "flag_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalBitflagSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalBitmaskType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bitmask_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.flag_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalBitmaskType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bitmask_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.flag_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommonBitfield {
    pub position: u16,
    pub flags: crate::types::xtypes::BitsetMemberFlag,
    pub bitcount: u8,
    pub holder_type: crate::types::xtypes::TypeKind,
}

impl CommonBitfield {
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: 0,
            flags: <crate::types::xtypes::BitsetMemberFlag>::default(),
            bitcount: 0,
            holder_type: <crate::types::xtypes::TypeKind>::default(),
        }
    }
}

impl ::std::default::Default for CommonBitfield {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CommonBitfield {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CommonBitfield",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "position",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u16>(),
        },
        ::intercom_cts::MemberInfo {
            name: "flags",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitsetMemberFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bitcount",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u8>(),
        },
        ::intercom_cts::MemberInfo {
            name: "holder_type",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeKind>(),
        },
    ];

    impl ::intercom_cts::Marshal for CommonBitfield {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.position)?;
            state.encode_field(&MEMBER_INFO[1], &self.flags)?;
            state.encode_field(&MEMBER_INFO[2], &self.bitcount)?;
            state.encode_field(&MEMBER_INFO[3], &self.holder_type)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CommonBitfield {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.position)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.flags)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.bitcount)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.holder_type)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteBitfield {
    pub common: crate::types::xtypes::CommonBitfield,
    pub detail: crate::types::xtypes::CompleteMemberDetail,
}

impl CompleteBitfield {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonBitfield>::default(),
            detail: <crate::types::xtypes::CompleteMemberDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteBitfield {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteBitfield {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteBitfield",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonBitfield>(),
        },
        ::intercom_cts::MemberInfo {
            name: "detail",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteMemberDetail>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteBitfield {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteBitfield {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type CompleteBitfieldSeq = ::std::vec::Vec<crate::types::xtypes::CompleteBitfield>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalBitfield {
    pub common: crate::types::xtypes::CommonBitfield,
    pub name_hash: crate::types::xtypes::NameHash,
}

impl MinimalBitfield {
    #[must_use]
    pub fn new() -> Self {
        Self {
            common: <crate::types::xtypes::CommonBitfield>::default(),
            name_hash: <crate::types::xtypes::NameHash>::default(),
        }
    }
}

impl ::std::default::Default for MinimalBitfield {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalBitfield {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalBitfield",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "common",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CommonBitfield>(),
        },
        ::intercom_cts::MemberInfo {
            name: "name_hash",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::NameHash>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalBitfield {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.common)?;
            state.encode_field(&MEMBER_INFO[1], &self.name_hash)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalBitfield {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.common)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.name_hash)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type MinimalBitfieldSeq = ::std::vec::Vec<crate::types::xtypes::MinimalBitfield>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteBitsetHeader {
    pub detail: crate::types::xtypes::CompleteTypeDetail,
}

impl CompleteBitsetHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            detail: <crate::types::xtypes::CompleteTypeDetail>::default(),
        }
    }
}

impl ::std::default::Default for CompleteBitsetHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteBitsetHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteBitsetHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[::intercom_cts::MemberInfo {
        name: "detail",
        member_id: 0,
        flags: ::intercom_cts::MemberFlag::nil(),
        type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeDetail>(),
    }];

    impl ::intercom_cts::Marshal for CompleteBitsetHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.detail)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteBitsetHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.detail)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalBitsetHeader {}

impl MinimalBitsetHeader {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for MinimalBitsetHeader {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalBitsetHeader {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalBitsetHeader",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for MinimalBitsetHeader {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalBitsetHeader {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CompleteBitsetType {
    pub bitset_flags: crate::types::xtypes::BitsetTypeFlag,
    pub header: crate::types::xtypes::CompleteBitsetHeader,
    pub field_seq: crate::types::xtypes::CompleteBitfieldSeq,
}

impl CompleteBitsetType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitset_flags: <crate::types::xtypes::BitsetTypeFlag>::default(),
            header: <crate::types::xtypes::CompleteBitsetHeader>::default(),
            field_seq: <crate::types::xtypes::CompleteBitfieldSeq>::default(),
        }
    }
}

impl ::std::default::Default for CompleteBitsetType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteBitsetType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteBitsetType",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "bitset_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitsetTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteBitsetHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "field_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteBitfieldSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteBitsetType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bitset_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.field_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteBitsetType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bitset_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.field_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalBitsetType {
    pub bitset_flags: crate::types::xtypes::BitsetTypeFlag,
    pub header: crate::types::xtypes::MinimalBitsetHeader,
    pub field_seq: crate::types::xtypes::MinimalBitfieldSeq,
}

impl MinimalBitsetType {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitset_flags: <crate::types::xtypes::BitsetTypeFlag>::default(),
            header: <crate::types::xtypes::MinimalBitsetHeader>::default(),
            field_seq: <crate::types::xtypes::MinimalBitfieldSeq>::default(),
        }
    }
}

impl ::std::default::Default for MinimalBitsetType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalBitsetType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalBitsetType",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "bitset_flags",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::BitsetTypeFlag>(),
        },
        ::intercom_cts::MemberInfo {
            name: "header",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalBitsetHeader>(),
        },
        ::intercom_cts::MemberInfo {
            name: "field_seq",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalBitfieldSeq>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalBitsetType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.bitset_flags)?;
            state.encode_field(&MEMBER_INFO[1], &self.header)?;
            state.encode_field(&MEMBER_INFO[2], &self.field_seq)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalBitsetType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.bitset_flags)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.header)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.field_seq)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CompleteExtendedType {}

impl CompleteExtendedType {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for CompleteExtendedType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteExtendedType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteExtendedType",
        flags: ::intercom_cts::TypeFlag::IS_MUTABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for CompleteExtendedType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteExtendedType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum CompleteTypeObject {
    AliasType(crate::types::xtypes::CompleteAliasType),
    AnnotationType(crate::types::xtypes::CompleteAnnotationType),
    StructType(crate::types::xtypes::CompleteStructType),
    UnionType(crate::types::xtypes::CompleteUnionType),
    BitsetType(crate::types::xtypes::CompleteBitsetType),
    SequenceType(crate::types::xtypes::CompleteSequenceType),
    ArrayType(crate::types::xtypes::CompleteArrayType),
    MapType(crate::types::xtypes::CompleteMapType),
    EnumeratedType(crate::types::xtypes::CompleteEnumeratedType),
    BitmaskType(crate::types::xtypes::CompleteBitmaskType),
    ExtendedType(crate::types::xtypes::CompleteExtendedType),
}

impl CompleteTypeObject {
    #[must_use]
    pub fn new() -> Self {
        Self::ExtendedType(<crate::types::xtypes::CompleteExtendedType>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> u8 {
        match self {
            Self::AliasType(_) => crate::types::xtypes::TK_ALIAS,
            Self::AnnotationType(_) => crate::types::xtypes::TK_ANNOTATION,
            Self::StructType(_) => crate::types::xtypes::TK_STRUCTURE,
            Self::UnionType(_) => crate::types::xtypes::TK_UNION,
            Self::BitsetType(_) => crate::types::xtypes::TK_BITSET,
            Self::SequenceType(_) => crate::types::xtypes::TK_SEQUENCE,
            Self::ArrayType(_) => crate::types::xtypes::TK_ARRAY,
            Self::MapType(_) => crate::types::xtypes::TK_MAP,
            Self::EnumeratedType(_) => crate::types::xtypes::TK_ENUM,
            Self::BitmaskType(_) => crate::types::xtypes::TK_BITMASK,
            Self::ExtendedType(_) => 0,
        }
    }
}

impl From<u8> for CompleteTypeObject {
    fn from(disc: u8) -> Self {
        match disc {
            crate::types::xtypes::TK_ALIAS => {
                Self::AliasType(<crate::types::xtypes::CompleteAliasType>::default())
            }
            crate::types::xtypes::TK_ANNOTATION => {
                Self::AnnotationType(<crate::types::xtypes::CompleteAnnotationType>::default())
            }
            crate::types::xtypes::TK_STRUCTURE => {
                Self::StructType(<crate::types::xtypes::CompleteStructType>::default())
            }
            crate::types::xtypes::TK_UNION => {
                Self::UnionType(<crate::types::xtypes::CompleteUnionType>::default())
            }
            crate::types::xtypes::TK_BITSET => {
                Self::BitsetType(<crate::types::xtypes::CompleteBitsetType>::default())
            }
            crate::types::xtypes::TK_SEQUENCE => {
                Self::SequenceType(<crate::types::xtypes::CompleteSequenceType>::default())
            }
            crate::types::xtypes::TK_ARRAY => {
                Self::ArrayType(<crate::types::xtypes::CompleteArrayType>::default())
            }
            crate::types::xtypes::TK_MAP => {
                Self::MapType(<crate::types::xtypes::CompleteMapType>::default())
            }
            crate::types::xtypes::TK_ENUM => {
                Self::EnumeratedType(<crate::types::xtypes::CompleteEnumeratedType>::default())
            }
            crate::types::xtypes::TK_BITMASK => {
                Self::BitmaskType(<crate::types::xtypes::CompleteBitmaskType>::default())
            }
            _ => Self::default(),
        }
    }
}

impl ::std::default::Default for CompleteTypeObject {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for CompleteTypeObject {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::CompleteTypeObject",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Union,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "alias_type",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteAliasType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "annotation_type",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteAnnotationType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "struct_type",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteStructType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "union_type",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteUnionType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bitset_type",
            member_id: 5,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteBitsetType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "sequence_type",
            member_id: 6,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteSequenceType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "array_type",
            member_id: 7,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteArrayType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "map_type",
            member_id: 8,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteMapType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "enumerated_type",
            member_id: 9,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteEnumeratedType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bitmask_type",
            member_id: 10,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteBitmaskType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "extended_type",
            member_id: 11,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteExtendedType>(),
        },
    ];

    impl ::intercom_cts::Marshal for CompleteTypeObject {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::UnionSerializer as _;

            let mut state = ar.encode_union(&TYPE_INFO)?;
            state.encode_discriminant(&self.disc())?;
            match self {
                Self::AliasType(v) => state.encode_variant(&MEMBER_INFO[0], v),
                Self::AnnotationType(v) => state.encode_variant(&MEMBER_INFO[1], v),
                Self::StructType(v) => state.encode_variant(&MEMBER_INFO[2], v),
                Self::UnionType(v) => state.encode_variant(&MEMBER_INFO[3], v),
                Self::BitsetType(v) => state.encode_variant(&MEMBER_INFO[4], v),
                Self::SequenceType(v) => state.encode_variant(&MEMBER_INFO[5], v),
                Self::ArrayType(v) => state.encode_variant(&MEMBER_INFO[6], v),
                Self::MapType(v) => state.encode_variant(&MEMBER_INFO[7], v),
                Self::EnumeratedType(v) => state.encode_variant(&MEMBER_INFO[8], v),
                Self::BitmaskType(v) => state.encode_variant(&MEMBER_INFO[9], v),
                _ => state.encode_null(),
            }
        }
    }

    impl ::intercom_cts::Unmarshal for CompleteTypeObject {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::UnionDeserializer as _;

            let mut state = ar.decode_union(&TYPE_INFO)?;
            let mut disc = u8::default();
            state.decode_discriminant(&mut disc)?;
            *self = match disc {
                crate::types::xtypes::TK_ALIAS => {
                    let mut value = <crate::types::xtypes::CompleteAliasType>::default();
                    state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                    Self::AliasType(value)
                }
                crate::types::xtypes::TK_ANNOTATION => {
                    let mut value = <crate::types::xtypes::CompleteAnnotationType>::default();
                    state.decode_variant(&MEMBER_INFO[1], &mut value)?;
                    Self::AnnotationType(value)
                }
                crate::types::xtypes::TK_STRUCTURE => {
                    let mut value = <crate::types::xtypes::CompleteStructType>::default();
                    state.decode_variant(&MEMBER_INFO[2], &mut value)?;
                    Self::StructType(value)
                }
                crate::types::xtypes::TK_UNION => {
                    let mut value = <crate::types::xtypes::CompleteUnionType>::default();
                    state.decode_variant(&MEMBER_INFO[3], &mut value)?;
                    Self::UnionType(value)
                }
                crate::types::xtypes::TK_BITSET => {
                    let mut value = <crate::types::xtypes::CompleteBitsetType>::default();
                    state.decode_variant(&MEMBER_INFO[4], &mut value)?;
                    Self::BitsetType(value)
                }
                crate::types::xtypes::TK_SEQUENCE => {
                    let mut value = <crate::types::xtypes::CompleteSequenceType>::default();
                    state.decode_variant(&MEMBER_INFO[5], &mut value)?;
                    Self::SequenceType(value)
                }
                crate::types::xtypes::TK_ARRAY => {
                    let mut value = <crate::types::xtypes::CompleteArrayType>::default();
                    state.decode_variant(&MEMBER_INFO[6], &mut value)?;
                    Self::ArrayType(value)
                }
                crate::types::xtypes::TK_MAP => {
                    let mut value = <crate::types::xtypes::CompleteMapType>::default();
                    state.decode_variant(&MEMBER_INFO[7], &mut value)?;
                    Self::MapType(value)
                }
                crate::types::xtypes::TK_ENUM => {
                    let mut value = <crate::types::xtypes::CompleteEnumeratedType>::default();
                    state.decode_variant(&MEMBER_INFO[8], &mut value)?;
                    Self::EnumeratedType(value)
                }
                crate::types::xtypes::TK_BITMASK => {
                    let mut value = <crate::types::xtypes::CompleteBitmaskType>::default();
                    state.decode_variant(&MEMBER_INFO[9], &mut value)?;
                    Self::BitmaskType(value)
                }
                _ => {
                    let mut value = <crate::types::xtypes::CompleteExtendedType>::default();
                    state.decode_variant(&MEMBER_INFO[10], &mut value)?;
                    Self::ExtendedType(value)
                }
            };
            Ok(())
        }
    }
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinimalExtendedType {}

impl MinimalExtendedType {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::default::Default for MinimalExtendedType {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalExtendedType {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalExtendedType",
        flags: ::intercom_cts::TypeFlag::IS_MUTABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    impl ::intercom_cts::Marshal for MinimalExtendedType {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let state = ar.encode_struct(&TYPE_INFO)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalExtendedType {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let state = ar.decode_struct(&TYPE_INFO)?;
            state.end()?;
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum MinimalTypeObject {
    AliasType(crate::types::xtypes::MinimalAliasType),
    AnnotationType(crate::types::xtypes::MinimalAnnotationType),
    StructType(crate::types::xtypes::MinimalStructType),
    UnionType(crate::types::xtypes::MinimalUnionType),
    BitsetType(crate::types::xtypes::MinimalBitsetType),
    SequenceType(crate::types::xtypes::MinimalSequenceType),
    ArrayType(crate::types::xtypes::MinimalArrayType),
    MapType(crate::types::xtypes::MinimalMapType),
    EnumeratedType(crate::types::xtypes::MinimalEnumeratedType),
    BitmaskType(crate::types::xtypes::MinimalBitmaskType),
    ExtendedType(crate::types::xtypes::MinimalExtendedType),
}

impl MinimalTypeObject {
    #[must_use]
    pub fn new() -> Self {
        Self::ExtendedType(<crate::types::xtypes::MinimalExtendedType>::default())
    }

    #[must_use]
    pub const fn disc(&self) -> u8 {
        match self {
            Self::AliasType(_) => crate::types::xtypes::TK_ALIAS,
            Self::AnnotationType(_) => crate::types::xtypes::TK_ANNOTATION,
            Self::StructType(_) => crate::types::xtypes::TK_STRUCTURE,
            Self::UnionType(_) => crate::types::xtypes::TK_UNION,
            Self::BitsetType(_) => crate::types::xtypes::TK_BITSET,
            Self::SequenceType(_) => crate::types::xtypes::TK_SEQUENCE,
            Self::ArrayType(_) => crate::types::xtypes::TK_ARRAY,
            Self::MapType(_) => crate::types::xtypes::TK_MAP,
            Self::EnumeratedType(_) => crate::types::xtypes::TK_ENUM,
            Self::BitmaskType(_) => crate::types::xtypes::TK_BITMASK,
            Self::ExtendedType(_) => 0,
        }
    }
}

impl From<u8> for MinimalTypeObject {
    fn from(disc: u8) -> Self {
        match disc {
            crate::types::xtypes::TK_ALIAS => {
                Self::AliasType(<crate::types::xtypes::MinimalAliasType>::default())
            }
            crate::types::xtypes::TK_ANNOTATION => {
                Self::AnnotationType(<crate::types::xtypes::MinimalAnnotationType>::default())
            }
            crate::types::xtypes::TK_STRUCTURE => {
                Self::StructType(<crate::types::xtypes::MinimalStructType>::default())
            }
            crate::types::xtypes::TK_UNION => {
                Self::UnionType(<crate::types::xtypes::MinimalUnionType>::default())
            }
            crate::types::xtypes::TK_BITSET => {
                Self::BitsetType(<crate::types::xtypes::MinimalBitsetType>::default())
            }
            crate::types::xtypes::TK_SEQUENCE => {
                Self::SequenceType(<crate::types::xtypes::MinimalSequenceType>::default())
            }
            crate::types::xtypes::TK_ARRAY => {
                Self::ArrayType(<crate::types::xtypes::MinimalArrayType>::default())
            }
            crate::types::xtypes::TK_MAP => {
                Self::MapType(<crate::types::xtypes::MinimalMapType>::default())
            }
            crate::types::xtypes::TK_ENUM => {
                Self::EnumeratedType(<crate::types::xtypes::MinimalEnumeratedType>::default())
            }
            crate::types::xtypes::TK_BITMASK => {
                Self::BitmaskType(<crate::types::xtypes::MinimalBitmaskType>::default())
            }
            _ => Self::default(),
        }
    }
}

impl ::std::default::Default for MinimalTypeObject {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for MinimalTypeObject {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::MinimalTypeObject",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Union,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "alias_type",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalAliasType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "annotation_type",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalAnnotationType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "struct_type",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalStructType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "union_type",
            member_id: 4,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalUnionType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bitset_type",
            member_id: 5,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalBitsetType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "sequence_type",
            member_id: 6,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalSequenceType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "array_type",
            member_id: 7,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalArrayType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "map_type",
            member_id: 8,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalMapType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "enumerated_type",
            member_id: 9,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalEnumeratedType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "bitmask_type",
            member_id: 10,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalBitmaskType>(),
        },
        ::intercom_cts::MemberInfo {
            name: "extended_type",
            member_id: 11,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalExtendedType>(),
        },
    ];

    impl ::intercom_cts::Marshal for MinimalTypeObject {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::UnionSerializer as _;

            let mut state = ar.encode_union(&TYPE_INFO)?;
            state.encode_discriminant(&self.disc())?;
            match self {
                Self::AliasType(v) => state.encode_variant(&MEMBER_INFO[0], v),
                Self::AnnotationType(v) => state.encode_variant(&MEMBER_INFO[1], v),
                Self::StructType(v) => state.encode_variant(&MEMBER_INFO[2], v),
                Self::UnionType(v) => state.encode_variant(&MEMBER_INFO[3], v),
                Self::BitsetType(v) => state.encode_variant(&MEMBER_INFO[4], v),
                Self::SequenceType(v) => state.encode_variant(&MEMBER_INFO[5], v),
                Self::ArrayType(v) => state.encode_variant(&MEMBER_INFO[6], v),
                Self::MapType(v) => state.encode_variant(&MEMBER_INFO[7], v),
                Self::EnumeratedType(v) => state.encode_variant(&MEMBER_INFO[8], v),
                Self::BitmaskType(v) => state.encode_variant(&MEMBER_INFO[9], v),
                _ => state.encode_null(),
            }
        }
    }

    impl ::intercom_cts::Unmarshal for MinimalTypeObject {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::UnionDeserializer as _;

            let mut state = ar.decode_union(&TYPE_INFO)?;
            let mut disc = u8::default();
            state.decode_discriminant(&mut disc)?;
            *self = match disc {
                crate::types::xtypes::TK_ALIAS => {
                    let mut value = <crate::types::xtypes::MinimalAliasType>::default();
                    state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                    Self::AliasType(value)
                }
                crate::types::xtypes::TK_ANNOTATION => {
                    let mut value = <crate::types::xtypes::MinimalAnnotationType>::default();
                    state.decode_variant(&MEMBER_INFO[1], &mut value)?;
                    Self::AnnotationType(value)
                }
                crate::types::xtypes::TK_STRUCTURE => {
                    let mut value = <crate::types::xtypes::MinimalStructType>::default();
                    state.decode_variant(&MEMBER_INFO[2], &mut value)?;
                    Self::StructType(value)
                }
                crate::types::xtypes::TK_UNION => {
                    let mut value = <crate::types::xtypes::MinimalUnionType>::default();
                    state.decode_variant(&MEMBER_INFO[3], &mut value)?;
                    Self::UnionType(value)
                }
                crate::types::xtypes::TK_BITSET => {
                    let mut value = <crate::types::xtypes::MinimalBitsetType>::default();
                    state.decode_variant(&MEMBER_INFO[4], &mut value)?;
                    Self::BitsetType(value)
                }
                crate::types::xtypes::TK_SEQUENCE => {
                    let mut value = <crate::types::xtypes::MinimalSequenceType>::default();
                    state.decode_variant(&MEMBER_INFO[5], &mut value)?;
                    Self::SequenceType(value)
                }
                crate::types::xtypes::TK_ARRAY => {
                    let mut value = <crate::types::xtypes::MinimalArrayType>::default();
                    state.decode_variant(&MEMBER_INFO[6], &mut value)?;
                    Self::ArrayType(value)
                }
                crate::types::xtypes::TK_MAP => {
                    let mut value = <crate::types::xtypes::MinimalMapType>::default();
                    state.decode_variant(&MEMBER_INFO[7], &mut value)?;
                    Self::MapType(value)
                }
                crate::types::xtypes::TK_ENUM => {
                    let mut value = <crate::types::xtypes::MinimalEnumeratedType>::default();
                    state.decode_variant(&MEMBER_INFO[8], &mut value)?;
                    Self::EnumeratedType(value)
                }
                crate::types::xtypes::TK_BITMASK => {
                    let mut value = <crate::types::xtypes::MinimalBitmaskType>::default();
                    state.decode_variant(&MEMBER_INFO[9], &mut value)?;
                    Self::BitmaskType(value)
                }
                _ => {
                    let mut value = <crate::types::xtypes::MinimalExtendedType>::default();
                    state.decode_variant(&MEMBER_INFO[10], &mut value)?;
                    Self::ExtendedType(value)
                }
            };
            Ok(())
        }
    }
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum TypeObject {
    Complete(crate::types::xtypes::CompleteTypeObject),
    Minimal(crate::types::xtypes::MinimalTypeObject),
    Null,
}

impl TypeObject {
    #[must_use]
    pub fn new() -> Self {
        Self::Null
    }

    #[must_use]
    pub const fn disc(&self) -> u8 {
        match self {
            Self::Complete(_) => crate::types::xtypes::EK_COMPLETE,
            Self::Minimal(_) => crate::types::xtypes::EK_MINIMAL,
            Self::Null => 0,
        }
    }
}

impl From<u8> for TypeObject {
    fn from(disc: u8) -> Self {
        match disc {
            crate::types::xtypes::EK_COMPLETE => {
                Self::Complete(<crate::types::xtypes::CompleteTypeObject>::default())
            }
            crate::types::xtypes::EK_MINIMAL => {
                Self::Minimal(<crate::types::xtypes::MinimalTypeObject>::default())
            }
            _ => Self::default(),
        }
    }
}

impl ::std::default::Default for TypeObject {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeObject {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeObject",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Union,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "complete",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::CompleteTypeObject>(),
        },
        ::intercom_cts::MemberInfo {
            name: "minimal",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::MinimalTypeObject>(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeObject {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::UnionSerializer as _;

            let mut state = ar.encode_union(&TYPE_INFO)?;
            state.encode_discriminant(&self.disc())?;
            match self {
                Self::Complete(v) => state.encode_variant(&MEMBER_INFO[0], v),
                Self::Minimal(v) => state.encode_variant(&MEMBER_INFO[1], v),
                _ => state.encode_null(),
            }
        }
    }

    impl ::intercom_cts::Unmarshal for TypeObject {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::UnionDeserializer as _;

            let mut state = ar.decode_union(&TYPE_INFO)?;
            let mut disc = u8::default();
            state.decode_discriminant(&mut disc)?;
            *self = match disc {
                crate::types::xtypes::EK_COMPLETE => {
                    let mut value = <crate::types::xtypes::CompleteTypeObject>::default();
                    state.decode_variant(&MEMBER_INFO[0], &mut value)?;
                    Self::Complete(value)
                }
                crate::types::xtypes::EK_MINIMAL => {
                    let mut value = <crate::types::xtypes::MinimalTypeObject>::default();
                    state.decode_variant(&MEMBER_INFO[1], &mut value)?;
                    Self::Minimal(value)
                }
                _ => Self::Null,
            };
            Ok(())
        }
    }
};

pub type TypeObjectSeq = ::std::vec::Vec<crate::types::xtypes::TypeObject>;

pub type StronglyConnectedComponent = crate::types::xtypes::TypeObjectSeq;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TypeIdentifierTypeObjectPair {
    pub type_identifier: crate::types::xtypes::TypeIdentifier,
    pub type_object: crate::types::xtypes::TypeObject,
}

impl TypeIdentifierTypeObjectPair {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_identifier: <crate::types::xtypes::TypeIdentifier>::default(),
            type_object: <crate::types::xtypes::TypeObject>::default(),
        }
    }
}

impl ::std::default::Default for TypeIdentifierTypeObjectPair {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeIdentifierTypeObjectPair {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeIdentifierTypeObjectPair",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "type_identifier",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_object",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeObject>(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeIdentifierTypeObjectPair {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.type_identifier)?;
            state.encode_field(&MEMBER_INFO[1], &self.type_object)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for TypeIdentifierTypeObjectPair {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.type_identifier)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.type_object)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type TypeIdentifierTypeObjectPairSeq =
    ::std::vec::Vec<crate::types::xtypes::TypeIdentifierTypeObjectPair>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TypeIdentifierPair {
    pub type_identifier1: crate::types::xtypes::TypeIdentifier,
    pub type_identifier2: crate::types::xtypes::TypeIdentifier,
}

impl TypeIdentifierPair {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_identifier1: <crate::types::xtypes::TypeIdentifier>::default(),
            type_identifier2: <crate::types::xtypes::TypeIdentifier>::default(),
        }
    }
}

impl ::std::default::Default for TypeIdentifierPair {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeIdentifierPair {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeIdentifierPair",
        flags: ::intercom_cts::TypeFlag::IS_FINAL.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "type_identifier1",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_identifier2",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeIdentifierPair {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.type_identifier1)?;
            state.encode_field(&MEMBER_INFO[1], &self.type_identifier2)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for TypeIdentifierPair {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.type_identifier1)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.type_identifier2)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type TypeIdentifierPairSeq = ::std::vec::Vec<crate::types::xtypes::TypeIdentifierPair>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TypeIdentifierWithSize {
    pub type_id: crate::types::xtypes::TypeIdentifier,
    pub typeobject_serialized_size: u32,
}

impl TypeIdentifierWithSize {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_id: <crate::types::xtypes::TypeIdentifier>::default(),
            typeobject_serialized_size: 0,
        }
    }
}

impl ::std::default::Default for TypeIdentifierWithSize {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeIdentifierWithSize {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeIdentifierWithSize",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "type_id",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifier>(),
        },
        ::intercom_cts::MemberInfo {
            name: "typeobject_serialized_size",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u32>(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeIdentifierWithSize {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.type_id)?;
            state.encode_field(&MEMBER_INFO[1], &self.typeobject_serialized_size)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for TypeIdentifierWithSize {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.type_id)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.typeobject_serialized_size)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type TypeIdentifierWithSizeSeq = ::std::vec::Vec<crate::types::xtypes::TypeIdentifierWithSize>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TypeIdentifierWithDependencies {
    pub typeid_with_size: crate::types::xtypes::TypeIdentifierWithSize,
    pub dependent_typeid_count: i32,
    pub dependent_typeids: ::std::vec::Vec<crate::types::xtypes::TypeIdentifierWithSize>,
}

impl TypeIdentifierWithDependencies {
    #[must_use]
    pub fn new() -> Self {
        Self {
            typeid_with_size: <crate::types::xtypes::TypeIdentifierWithSize>::default(),
            dependent_typeid_count: 0,
            dependent_typeids: vec![],
        }
    }
}

impl ::std::default::Default for TypeIdentifierWithDependencies {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeIdentifierWithDependencies {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeIdentifierWithDependencies",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "typeid_with_size",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifierWithSize>(),
        },
        ::intercom_cts::MemberInfo {
            name: "dependent_typeid_count",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<i32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "dependent_typeids",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                ::std::vec::Vec<crate::types::xtypes::TypeIdentifierWithSize>,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeIdentifierWithDependencies {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.typeid_with_size)?;
            state.encode_field(&MEMBER_INFO[1], &self.dependent_typeid_count)?;
            state.encode_field(&MEMBER_INFO[2], &self.dependent_typeids)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for TypeIdentifierWithDependencies {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.typeid_with_size)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.dependent_typeid_count)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.dependent_typeids)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type TypeIdentifierWithDependenciesSeq =
    ::std::vec::Vec<crate::types::xtypes::TypeIdentifierWithDependencies>;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TypeInformation {
    pub minimal: crate::types::xtypes::TypeIdentifierWithDependencies,
    pub complete: crate::types::xtypes::TypeIdentifierWithDependencies,
}

impl TypeInformation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            minimal: <crate::types::xtypes::TypeIdentifierWithDependencies>::default(),
            complete: <crate::types::xtypes::TypeIdentifierWithDependencies>::default(),
        }
    }
}

impl ::std::default::Default for TypeInformation {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeInformation {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeInformation",
        flags: ::intercom_cts::TypeFlag::IS_MUTABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "minimal",
            member_id: 4097,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::TypeIdentifierWithDependencies,
            >(),
        },
        ::intercom_cts::MemberInfo {
            name: "complete",
            member_id: 4098,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::TypeIdentifierWithDependencies,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeInformation {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.minimal)?;
            state.encode_field(&MEMBER_INFO[1], &self.complete)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for TypeInformation {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.minimal)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.complete)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type TypeInformationSeq = ::std::vec::Vec<crate::types::xtypes::TypeInformation>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TypeDefinition {
    pub type_name: ::std::string::String,
    pub type_info: crate::types::xtypes::TypeInformation,
    pub complete_to_minimal: crate::types::xtypes::TypeIdentifierPairSeq,
    pub type_objects: crate::types::xtypes::TypeIdentifierTypeObjectPairSeq,
}

impl TypeDefinition {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: <::std::string::String>::default(),
            type_info: <crate::types::xtypes::TypeInformation>::default(),
            complete_to_minimal: <crate::types::xtypes::TypeIdentifierPairSeq>::default(),
            type_objects: <crate::types::xtypes::TypeIdentifierTypeObjectPairSeq>::default(),
        }
    }
}

impl ::std::default::Default for TypeDefinition {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for TypeDefinition {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "DDS::XTypes::TypeDefinition",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "type_name",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<::std::string::String>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_info",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeInformation>(),
        },
        ::intercom_cts::MemberInfo {
            name: "complete_to_minimal",
            member_id: 2,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<crate::types::xtypes::TypeIdentifierPairSeq>(),
        },
        ::intercom_cts::MemberInfo {
            name: "type_objects",
            member_id: 3,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<
                crate::types::xtypes::TypeIdentifierTypeObjectPairSeq,
            >(),
        },
    ];

    impl ::intercom_cts::Marshal for TypeDefinition {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.type_name)?;
            state.encode_field(&MEMBER_INFO[1], &self.type_info)?;
            state.encode_field(&MEMBER_INFO[2], &self.complete_to_minimal)?;
            state.encode_field(&MEMBER_INFO[3], &self.type_objects)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for TypeDefinition {
        fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'a>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.type_name)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.type_info)?;
            state.decode_field(&MEMBER_INFO[2], &mut self.complete_to_minimal)?;
            state.decode_field(&MEMBER_INFO[3], &mut self.type_objects)?;
            state.end()?;
            Ok(())
        }
    }
};

pub type TypeIdentifierTypeObjectMap = ::std::collections::BTreeMap<
    crate::types::xtypes::TypeIdentifier,
    crate::types::xtypes::TypeObject,
>;
