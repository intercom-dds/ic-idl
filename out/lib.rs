// @generated

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Foo {
    pub value: Box<String>,
}

impl Foo {
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: Box::new(<String>::default()),
        }
    }
}

impl ::std::default::Default for Foo {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
        name: "Foo",
        flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,
        kind: ::intercom_cts::TypeKind::Struct,
        key_kind: ::intercom_cts::TypeKind::None,
        element_kind: ::intercom_cts::TypeKind::None,
    };

    const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[
        ::intercom_cts::MemberInfo {
            name: "value",
            member_id: 0,
            flags: ::intercom_cts::MemberFlag::IS_EXTERNAL,
        },
    ];

    impl ::intercom_cts::Marshal for Foo {
        fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>
        where
            S: ::intercom_cts::encode::Serializer,
        {
            use ::intercom_cts::encode::StructSerializer as _;

            let mut state = ar.encode_struct(&TYPE_INFO)?;
            state.encode_field(&MEMBER_INFO[0], &self.value)?;
            state.end()
        }
    }

    impl ::intercom_cts::Unmarshal for Foo {
        fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>
        where
            D: ::intercom_cts::decode::Deserializer,
        {
            use ::intercom_cts::decode::StructDeserializer as _;

            let mut state = ar.decode_struct(&TYPE_INFO)?;
            state.decode_field(&MEMBER_INFO[0], &mut self.value)?;
            state.end()?;
            Ok(())
        }
    }
};

