// Copyright 2024 KONGSBERG
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

use std::ops::Range;

use crate::FileId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub offset: u32,
    pub file_id: FileId,
}

impl Location {
    #[must_use]
    pub fn new(offset: u32, file_id: FileId) -> Self {
        Self { offset, file_id }
    }
}

// This really shouldn't be default constructible, but all generated code
// relies on it. We have a sanity lint that verifies all `Position`s in the AST
// has a valid `FileId`, so that should hopefully catch such cases.
impl Default for Location {
    fn default() -> Self {
        Self::new(0, FileId::_do_not_use())
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        debug_assert_eq!(
            self.start.file_id, self.end.file_id,
            "attempted to derive Range<> from Span where start.file_id != end.file_id",
        );
        self.start.offset as usize..self.end.offset as usize
    }
}

impl From<Span> for Range<usize> {
    #[inline]
    fn from(val: Span) -> Self {
        val.range()
    }
}

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for Location {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
        const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] = &MEMBER_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "ast::Location",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "offset",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<u32>(),
        },
        ::intercom_cts::MemberInfo {
            name: "file_id",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<usize>(),
        },
    ];

    impl ::intercom_cts::Marshal for Location {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;
            let file_id = usize::from(self.file_id);

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.offset)?;
            state.encode_field(&MEMBER_INFO[1], &file_id)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for Location {
        fn unmarshal_mut<'de, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'de>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;
            let mut file_id: usize = 0;
            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.offset)?;
            state.decode_field(&MEMBER_INFO[1], &mut file_id)?;
            state.end()?;
            self.file_id = FileId::from(file_id);
            Ok(())
        }
    }
};

const _: () = {
    impl ::intercom_cts::type_info::TypeDescriptor for Span {
        const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;
        const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] = &MEMBER_INFO;
    }

    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "ast::Span",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE.union(::intercom_cts::TypeFlag::IS_NESTED),
        kind: ::intercom_cts::TypeKind::Struct,
        key_info: None,
        element_info: None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "start",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<Location>(),
        },
        ::intercom_cts::MemberInfo {
            name: "end",
            member_id: 1,
            flags: ::intercom_cts::MemberFlag::nil(),
            type_info: ::intercom_cts::type_info::<Location>(),
        },
    ];

    impl ::intercom_cts::Marshal for Span {
        fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer<'a>,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.start)?;
            state.encode_field(&MEMBER_INFO[1], &self.end)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for Span {
        fn unmarshal_mut<'de, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer<'de>,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.start)?;
            state.decode_field(&MEMBER_INFO[1], &mut self.end)?;
            state.end()?;
            Ok(())
        }
    }
};
