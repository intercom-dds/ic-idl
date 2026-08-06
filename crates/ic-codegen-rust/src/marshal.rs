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

use ic_emit::printer::Twine;
use ic_emit::w;
use ic_hir::hir::{AliasTy, Def, DefKind, Member, TyKind};

use crate::codegen::RustGen;
use crate::helpers::{
    is_key, is_must_understand, is_optional, is_shared, member_id, rust_primitive, type_flags,
};

fn emit_member_flags(member: &Member, w: &mut Twine) {
    let mut flags = Vec::new();

    if is_key(member) {
        flags.push("IS_KEY");
    }

    if is_optional(member) {
        flags.push("IS_OPTIONAL");
    }

    if is_shared(member) {
        flags.push("IS_EXTERNAL");
    }

    if is_must_understand(member) {
        flags.push("IS_MUST_UNDERSTAND");
    }

    if flags.is_empty() {
        w!(w, "::intercom_cts::MemberFlag::nil()");
    } else {
        w!(w, "::intercom_cts::MemberFlag::", flags[0]);
        for flag in &flags[1..] {
            w!(w, ".union(::intercom_cts::MemberFlag::", flag, ")");
        }
    }
}

impl RustGen<'_> {
    pub(crate) fn emit_type_info(&self, def: &Def, w: &mut Twine) {
        let full_name = self.original_qualified_name(def.id);
        let kind = match &def.kind {
            DefKind::Union(_) => "Union",
            DefKind::Enum(_) => "Enum",
            DefKind::Bitmask(_) => "Bitmask",
            DefKind::Struct(_)
            | DefKind::Valuetype(_)
            | DefKind::Except(_)
            | DefKind::Module(_)
            | DefKind::Const(_)
            | DefKind::Bitset(_)
            | DefKind::Alias(_)
            | DefKind::Interface(_)
            | DefKind::Annotation(_)
            | DefKind::Decl(_) => "Struct",
        };

        let element_info = match &def.kind {
            DefKind::Enum(enum_ty) => Some(rust_primitive(enum_ty.ty)),
            DefKind::Bitmask(bitmask_ty) => Some(rust_primitive(bitmask_ty.ty)),
            _ => None,
        };

        w!(w, "const _: () = {\n");
        Self::emit_type_descriptor(def, w);
        w!(w, "const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {\n");
        w!(w, "name: \"", full_name, "\",\n");
        w!(w, "flags: ", type_flags(&self.hir.context, def), ",\n");
        w!(w, "kind: ::intercom_cts::TypeKind::", kind, ",\n");
        w!(w, "key_info: None,\n");
        if let Some(elem_ty) = element_info {
            w!(w, "element_info: Some(::intercom_cts::type_info::<", elem_ty, ">()),\n");
        } else {
            w!(w, "element_info: None,\n");
        }
        w!(w, "};\n\n");
    }

    fn emit_type_descriptor(def: &Def, w: &mut Twine) {
        w!(w, "impl ::intercom_cts::type_info::TypeDescriptor for ", def, " {\n");
        w!(w, "const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> = &TYPE_INFO;\n");
        w!(w, "const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] = MEMBER_INFO;\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_type_info_close(w: &mut Twine) {
        w!(w, "};\n\n");
    }

    pub(crate) fn emit_newtype_type_descriptor(&self, def: &Def, alias: &AliasTy, w: &mut Twine) {
        let inner_ty = self.rust_type(&alias.ty, def.id);
        w!(w, "impl ::intercom_cts::type_info::TypeDescriptor for ", def, " {\n");

        w!(w, "const TYPE_INFO: &'static ::intercom_cts::TypeInfo<'static> =\n");
        w!(w, "\t::intercom_cts::type_info::<", inner_ty, ">();\n");

        w!(w, "const MEMBER_INFO: &'static [::intercom_cts::MemberInfo<'static>] =\n");
        w!(w, "\t::intercom_cts::member_info::<", inner_ty, ">();\n");

        w!(w, "}\n\n");
    }

    pub(crate) fn emit_newtype_marshal_impl(def: &Def, w: &mut Twine) {
        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "self.0.marshal(ar)\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_newtype_unmarshal_impl(def: &Def, w: &mut Twine) {
        w!(w, "impl ::intercom_cts::Unmarshal for ", def, " {\n");
        w!(w, "fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "self.0.unmarshal_mut(ar)\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_member_info(&self, def_id: ic_hir::hir::DefId, w: &mut Twine) {
        let original_def = self.original_hir.context.definitions.get(def_id);
        let original_members = match &original_def.kind {
            DefKind::Struct(s) => self.original_struct_members(s),
            DefKind::Except(e) => e.members.clone(),
            DefKind::Valuetype(v) => self.original_valuetype_members(v),
            _ => vec![],
        };

        // Get the regular (renamed) members to check for optional annotations and types
        let def = self.hir.context.definitions.get(def_id);
        let members = match &def.kind {
            DefKind::Struct(s) => self.struct_members(s),
            DefKind::Except(e) => e.members.clone(),
            DefKind::Valuetype(v) => self.valuetype_members(v),
            _ => vec![],
        };

        w!(w, "const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[\n");
        let mut id = 0usize;
        for (orig_member, member) in original_members.iter().zip(members.iter()) {
            let type_str = self.rust_type(&member.ty, def_id);
            id = member_id(orig_member, id);
            w!(w, "::intercom_cts::MemberInfo {\n");
            w!(w, "name: \"", orig_member.ident.name, "\",\n");
            w!(w, "member_id: ", id.to_string(), ",\n");
            id += 1;
            w!(w, "flags: ");
            emit_member_flags(member, w);
            w!(w, ",\n");
            w!(w, "type_info: ::intercom_cts::type_info::<", type_str, ">(),\n");
            w!(w, "},\n");
        }
        w!(w, "];\n\n");
    }

    pub(crate) fn emit_marshal_impl<'c, I>(def: &Def, members: I, w: &mut Twine)
    where
        I: IntoIterator<Item = &'c ic_hir::hir::Member>,
    {
        let member_data: Vec<_> = members
            .into_iter()
            .enumerate()
            .map(|(i, m)| (m.ident.name.clone(), i, is_optional(m)))
            .collect();

        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::encode::StructSerializer as _;\n\n");

        w!(w, "let ");
        if !member_data.is_empty() {
            w!(w, "mut ");
        }
        w!(w, "state = ar.encode_struct(&TYPE_INFO)?;\n");

        for (name, idx, is_optional) in &member_data {
            if *is_optional {
                w!(w, "state.encode_optional(&MEMBER_INFO[", idx.to_string(), "], &self.", name, ")?;\n");
            } else {
                w!(w, "state.encode_field(&MEMBER_INFO[", idx.to_string(), "], &self.", name, ")?;\n");
            }
        }
        w!(w, "state.end()\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_unmarshal_impl<'c, I>(def: &Def, members: I, w: &mut Twine)
    where
        I: IntoIterator<Item = &'c ic_hir::hir::Member>,
    {
        let member_data: Vec<_> = members
            .into_iter()
            .enumerate()
            .map(|(i, m)| (m.ident.name.clone(), i))
            .collect();

        w!(w, "impl ::intercom_cts::Unmarshal for ", def, " {\n");
        w!(w, "fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::decode::StructDeserializer as _;\n\n");

        w!(w, "let ");
        if !member_data.is_empty() {
            w!(w, "mut ");
        }
        w!(w, "state = ar.decode_struct(&TYPE_INFO)?;\n");

        for (name, idx) in &member_data {
            w!(w, "state.decode_field(&MEMBER_INFO[", idx.to_string(), "], &mut self.", name, ")?;\n");
        }
        w!(w, "state.end()?;\n");
        w!(w, "Ok(())\n");
        w!(w, "}\n");
        w!(w, "}\n");
    }

    pub(crate) fn emit_enum_type_info(
        &self,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
        w: &mut Twine,
    ) {
        let full_name = self.original_qualified_name(def.id);
        let element_ty = rust_primitive(enum_ty.ty);

        w!(w, "const _: () = {\n");
        Self::emit_type_descriptor(def, w);
        w!(w, "const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {\n");
        w!(w, "name: \"", full_name, "\",\n");
        w!(w, "flags: ", type_flags(&self.hir.context, def), ",\n");
        w!(w, "kind: ::intercom_cts::TypeKind::Enum,\n");
        w!(w, "key_info: None,\n");
        w!(w, "element_info: Some(::intercom_cts::type_info::<", element_ty, ">()),\n");
        w!(w, "};\n\n");
    }

    pub(crate) fn emit_bitmask_type_info(
        &self,
        def: &Def,
        bitmask_ty: &ic_hir::hir::BitmaskTy,
        w: &mut Twine,
    ) {
        let full_name = self.original_qualified_name(def.id);
        let element_ty = rust_primitive(bitmask_ty.ty);

        w!(w, "const _: () = {\n");
        Self::emit_type_descriptor(def, w);
        w!(w, "const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {\n");
        w!(w, "name: \"", full_name, "\",\n");
        w!(w, "flags: ", type_flags(&self.hir.context, def), ",\n");
        w!(w, "kind: ::intercom_cts::TypeKind::Bitmask,\n");
        w!(w, "key_info: None,\n");
        w!(w, "element_info: Some(::intercom_cts::type_info::<", element_ty, ">()),\n");
        w!(w, "};\n\n");
    }

    pub(crate) fn emit_bitmask_member_info(
        &self,
        def: &Def,
        bitmask_ty: &ic_hir::hir::BitmaskTy,
        w: &mut Twine,
    ) {
        let scoped_name = self.scoped_name(def.id, def.id);

        w!(w, "const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[\n");
        for &flag_id in &bitmask_ty.flags {
            let original_flag_def = self.original_hir.context.definitions.get(flag_id);
            let position = if let DefKind::Const(const_ty) = &original_flag_def.kind {
                self.original_hir
                    .context
                    .unsigned_value(&const_ty.value)
                    .trailing_zeros()
            } else {
                0
            };

            w!(w, "::intercom_cts::MemberInfo {\n");
            w!(w, "name: \"", original_flag_def.ident.name, "\",\n");
            w!(w, "member_id: ", position.to_string(), ",\n");
            w!(w, "flags: ::intercom_cts::MemberFlag::nil(),\n");
            w!(w, "type_info: ::intercom_cts::type_info::<", scoped_name, ">(),\n");
            w!(w, "},\n");
        }
        w!(w, "];\n\n");
    }

    pub(crate) fn emit_bitmask_marshal_impl(def: &Def, w: &mut Twine) {
        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::encode::BitmaskSerializer as _;\n\n");
        w!(w, "let state = ar.encode_bitmask(&TYPE_INFO)?;\n");
        w!(w, "state.encode_flag(self.0, MEMBER_INFO)\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_bitmask_unmarshal_impl(def: &Def, w: &mut Twine) {
        w!(w, "impl ::intercom_cts::Unmarshal for ", def, " {\n");
        w!(w, "fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::decode::BitmaskDeserializer as _;\n\n");
        w!(w, "let state = ar.decode_bitmask(&TYPE_INFO)?;\n");
        w!(w, "self.0 = state.decode_flags(MEMBER_INFO)?;\n");
        w!(w, "Ok(())\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_enum_member_info(
        &self,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
        w: &mut Twine,
    ) {
        let scoped_name = self.scoped_name(def.id, def.id);

        w!(w, "const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[\n");
        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let original_field_def = self.original_hir.context.definitions.get(field_id);
            w!(w, "::intercom_cts::MemberInfo {\n");
            w!(w, "name: \"", original_field_def.ident.name, "\",\n");
            w!(w, "member_id: ", i.to_string(), ",\n");
            w!(w, "flags: ::intercom_cts::MemberFlag::nil(),\n");
            w!(w, "type_info: ::intercom_cts::type_info::<", scoped_name, ">(),\n");
            w!(w, "},\n");
        }
        w!(w, "];\n\n");
    }

    pub(crate) fn emit_enum_marshal_impl(
        &self,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
        w: &mut Twine,
    ) {
        let rust_ty = rust_primitive(enum_ty.ty);

        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::encode::EnumSerializer as _;\n\n");
        w!(w, "let state = ar.encode_enum(&TYPE_INFO)?;\n");
        w!(w, "match self {\n");

        for (i, &field_id) in enum_ty.fields.iter().enumerate() {
            let field_def = self.hir.context.definitions.get(field_id);
            if let DefKind::Const(const_ty) = &field_def.kind {
                let value = Self::format_numeric(&const_ty.value);
                w!(w, "Self::", field_def, " => state.encode_variant::<", rust_ty, ">(&MEMBER_INFO[", i.to_string(), "], ", value, "),\n");
            }
        }

        w!(w, "}\n");
        w!(w, "}\n");
        w!(w, "}\n\n");
    }

    pub(crate) fn emit_enum_unmarshal_impl(
        &self,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
        w: &mut Twine,
    ) {
        let rust_ty = rust_primitive(enum_ty.ty);
        let qual = self.hir.context.qualified_name(def.id);

        w!(w, "impl ::intercom_cts::Unmarshal for ", def, " {\n");
        w!(w, "fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::decode::EnumDeserializer as _;\n\n");
        w!(w, "let state = ar.decode_enum(&TYPE_INFO)?;\n");
        w!(w, "*self = state.decode_enumerator(*self)?;\n");
        w!(w, "Ok(())\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "impl ::intercom_cts::decode::EnumVisitor for ", def, " {\n");
        w!(w, "fn member_id<'a, D>(self, de: D) -> ::std::result::Result<Self, D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::error::Error as _;\n\n");
        w!(w, "let value = match de.decode_", rust_ty, "()? {\n");

        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            if let DefKind::Const(const_ty) = &field_def.kind {
                let value = Self::format_numeric(&const_ty.value);
                w!(w, value, " => Self::", field_def, ",\n");
            }
        }

        w!(w, "_ => return Err(D::Error::custom(\"invalid enum value for type ", qual, "\")),\n");
        w!(w, "};\n");
        w!(w, "Ok(value)\n");
        w!(w, "}\n\n");

        w!(w, "fn member_field<'a, D>(self, name: &str) -> ::std::result::Result<Self, D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::error::Error as _;\n\n");
        w!(w, "let value = match name {\n");

        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            let original_field_def = self.original_hir.context.definitions.get(field_id);
            let original_name = &original_field_def.ident.name;
            w!(w, "\"", original_name, "\" => Self::", field_def, ",\n");
        }

        w!(w, "_ => return Err(D::Error::custom(\"invalid enum value for type ", qual, "\")),\n");
        w!(w, "};\n");
        w!(w, "Ok(value)\n");
        w!(w, "}\n");
        w!(w, "}\n");
    }

    pub(crate) fn emit_union_member_info(
        &self,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        w: &mut Twine,
    ) {
        let original_def = self.original_hir.context.definitions.get(def.id);
        let original_union = match &original_def.kind {
            DefKind::Union(u) => u,
            _ => union_ty,
        };

        let variants: Vec<_> = union_ty
            .variants
            .iter()
            .zip(&original_union.variants)
            .filter(|(v, _)| !matches!(v.ty.kind, TyKind::Null))
            .collect();

        if variants.is_empty() {
            return;
        }

        w!(w, "const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[\n");
        // Discriminator has member_id 0, so start at 1 for members
        for (i, (variant, orig_variant)) in variants.iter().enumerate() {
            let type_str = self.rust_type(&variant.ty, def.id);
            w!(w, "::intercom_cts::MemberInfo {\n");
            w!(w, "name: \"", orig_variant.ident.name, "\",\n");
            w!(w, "member_id: ", (i + 1).to_string(), ",\n");
            w!(w, "flags: ::intercom_cts::MemberFlag::nil(),\n");
            w!(w, "type_info: ::intercom_cts::type_info::<", type_str, ">(),\n");
            w!(w, "},\n");
        }
        w!(w, "];\n\n");
    }

    pub(crate) fn emit_union_marshal_impl(
        &self,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        w: &mut Twine,
    ) {
        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<'a, S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer<'a>,\n");
        w!(w, "{\n");

        w!(w, "use ::intercom_cts::encode::UnionSerializer as _;\n\n");
        w!(w, "let mut state = ar.encode_union(&TYPE_INFO)?;\n");
        w!(w, "state.encode_discriminant(&self.disc())?;\n");
        w!(w, "match self {\n");
        let mut info_idx = 0;
        for variant in &union_ty.variants {
            let is_null = matches!(variant.ty.kind, TyKind::Null);
            if variant.labels.is_empty() {
                if is_null {
                    w!(w, "Self::", variant.ident.name, " => state.encode_null(),\n");
                } else {
                    w!(w, "Self::", variant.ident.name, "(v) => state.encode_variant(&MEMBER_INFO[", info_idx.to_string(), "], v),\n");
                    info_idx += 1;
                }
            } else {
                for (i, label) in variant.labels.iter().enumerate() {
                    if i > 0 {
                        w!(w, " | ");
                    }
                    let variant_name = self.union_variant_name(variant, label, union_ty);
                    if is_null {
                        w!(w, "Self::", variant_name);
                    } else {
                        w!(w, "Self::", variant_name, "(v)");
                    }
                }
                if is_null {
                    w!(w, " => state.encode_null(),\n");
                } else {
                    w!(w, " => state.encode_variant(&MEMBER_INFO[", info_idx.to_string(), "], v),\n");
                    info_idx += 1;
                }
            }
        }
        w!(w, "}\n");
        w!(w, "}\n");

        w!(w, "}\n\n");
    }

    pub(crate) fn emit_union_unmarshal_impl(
        &self,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        w: &mut Twine,
    ) {
        let disc_ty = self.rust_type(&union_ty.disc.ty, def.id);

        w!(w, "impl ::intercom_cts::Unmarshal for ", def, " {\n");
        w!(w, "fn unmarshal_mut<'a, D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer<'a>,\n");
        w!(w, "{\n");

        w!(w, "use ::intercom_cts::decode::UnionDeserializer as _;\n\n");
        w!(w, "let mut state = ar.decode_union(&TYPE_INFO)?;\n");
        w!(w, "let mut disc = ", disc_ty, "::default();\n");
        w!(w, "state.decode_discriminant(&mut disc)?;\n");
        w!(w, "*self = match disc {\n");
        let mut info_idx = 0;
        for variant in &union_ty.variants {
            if !variant.labels.is_empty() && !variant.is_default {
                if matches!(variant.ty.kind, TyKind::Null) {
                    for label in &variant.labels {
                        let variant_name = self.union_variant_name(variant, label, union_ty);
                        self.emit_const_value(&label.value, &union_ty.disc.ty, def.id, w);
                        w!(w, " => Self::", variant_name, ",\n");
                    }
                } else {
                    let variant_name =
                        self.union_variant_name(variant, &variant.labels[0], union_ty);
                    for (i, label) in variant.labels.iter().enumerate() {
                        if i > 0 {
                            w!(w, " | ");
                        }
                        self.emit_const_value(&label.value, &union_ty.disc.ty, def.id, w);
                    }
                    w!(w, " => {\n");
                    w!(w, "let mut value = ");
                    self.emit_default_value(&variant.ty, def.id, w);
                    w!(w, ";\n");
                    w!(w, "state.decode_variant(&MEMBER_INFO[", info_idx.to_string(), "], &mut value)?;\n");
                    w!(w, "Self::", variant_name, "(value)\n");
                    w!(w, "},\n");
                    info_idx += 1;
                }
            }
        }

        for variant in &union_ty.variants {
            if variant.is_default {
                let variant_name = if variant.labels.is_empty() {
                    variant.ident.name.clone()
                } else {
                    self.union_variant_name(variant, &variant.labels[0], union_ty)
                };

                if matches!(variant.ty.kind, TyKind::Null) {
                    w!(w, "_ => Self::", variant_name, ",\n");
                } else {
                    w!(w, "_ => {\n");
                    w!(w, "let mut value = ");
                    self.emit_default_value(&variant.ty, def.id, w);
                    w!(w, ";\n");
                    w!(w, "state.decode_variant(&MEMBER_INFO[", info_idx.to_string(), "], &mut value)?;\n");
                    w!(w, "Self::", variant_name, "(value)\n");
                    w!(w, "},\n");
                }
                break;
            }
        }

        w!(w, "};\n");
        w!(w, "Ok(())\n");
        w!(w, "}\n");

        w!(w, "}\n");
    }
}
