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
use ic_hir::hir::{Def, DefKind, PrimitiveTy, TyKind};

use crate::codegen::RustGen;
use crate::helpers::rust_primitive;

fn primitive_type_kind(prim: PrimitiveTy) -> &'static str {
    match prim {
        PrimitiveTy::Int8 => "I8",
        PrimitiveTy::UInt8 => "U8",
        PrimitiveTy::Int16 => "I16",
        PrimitiveTy::UInt16 => "U16",
        PrimitiveTy::UInt32 | PrimitiveTy::UInt64 => "U32",
        _ => "I32",
    }
}

impl RustGen<'_> {
    pub(crate) fn emit_type_info(&self, def: &Def, w: &mut Twine) {
        let full_name = self.scoped_name(def.id, def.id).replace("crate::", "");
        let (kind, element_kind) = match &def.kind {
            DefKind::Union(_) => ("Union", "None"),
            DefKind::Enum(enum_ty) => {
                let elem = primitive_type_kind(enum_ty.ty);
                ("Enum", elem)
            }
            DefKind::Struct(_)
            | DefKind::Valuetype(_)
            | DefKind::Except(_)
            | DefKind::Module(_)
            | DefKind::Const(_)
            | DefKind::Bitmask(_)
            | DefKind::Bitset(_)
            | DefKind::Alias(_)
            | DefKind::Interface(_)
            | DefKind::Annotation(_)
            | DefKind::Decl(_) => ("Struct", "None"),
        };

        w!(w, "const _: () = {\n");
        w!(w, "const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {\n");
        w!(w, "name: \"", full_name, "\",\n");
        w!(w, "flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,\n");
        w!(w, "kind: ::intercom_cts::TypeKind::", kind, ",\n");
        w!(w, "key_kind: ::intercom_cts::TypeKind::None,\n");
        w!(w, "element_kind: ::intercom_cts::TypeKind::", element_kind, ",\n");
        w!(w, "};\n\n");
    }

    pub(crate) fn emit_type_info_close(w: &mut Twine) {
        w!(w, "};\n\n");
    }

    fn emit_member_info_array(members: &[(&str, usize)], w: &mut Twine) {
        if members.is_empty() {
            return;
        }

        w!(w, "const MEMBER_INFO: &[::intercom_cts::MemberInfo<'static>] = &[\n");
        for (name, id) in members {
            w!(w, "::intercom_cts::MemberInfo {\n");
            w!(w, "name: \"", name, "\",\n");
            w!(w, "member_id: ", id.to_string(), ",\n");
            w!(w, "flags: ::intercom_cts::MemberFlag::nil(),\n");
            w!(w, "},\n");
        }
        w!(w, "];\n\n");
    }

    pub(crate) fn emit_member_info<'c, I>(members: I, w: &mut Twine)
    where
        I: IntoIterator<Item = &'c ic_hir::hir::Member>,
    {
        let member_info: Vec<_> = members
            .into_iter()
            .enumerate()
            .map(|(i, m)| (m.ident.name.as_str(), i))
            .collect();
        Self::emit_member_info_array(&member_info, w);
    }

    pub(crate) fn emit_marshal_impl<'c, I>(def: &Def, members: I, w: &mut Twine)
    where
        I: IntoIterator<Item = &'c ic_hir::hir::Member>,
    {
        let member_data: Vec<_> = members
            .into_iter()
            .enumerate()
            .map(|(i, m)| (m.ident.name.clone(), i))
            .collect();

        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::encode::StructSerializer as _;\n\n");

        w!(w, "let ");
        if !member_data.is_empty() {
            w!(w, "mut ");
        }
        w!(w, "state = ar.encode_struct(&TYPE_INFO)?;\n");

        for (name, idx) in &member_data {
            w!(w, "state.encode_field(&MEMBER_INFO[", idx.to_string(), "], &self.", name, ")?;\n");
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
        w!(w, "fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer,\n");
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
        let full_name = self.scoped_name(def.id, def.id).replace("crate::", "");
        let element_ty = primitive_type_kind(enum_ty.ty);

        w!(w, "const _: () = {\n");
        w!(w, "const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {\n");
        w!(w, "name: \"", full_name, "\",\n");
        w!(w, "flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,\n");
        w!(w, "kind: ::intercom_cts::TypeKind::Enum,\n");
        w!(w, "key_kind: ::intercom_cts::TypeKind::None,\n");
        w!(w, "element_kind: ::intercom_cts::TypeKind::", element_ty, ",\n");
        w!(w, "};\n\n");
    }

    pub(crate) fn emit_enum_marshal_impl(
        &self,
        def: &Def,
        enum_ty: &ic_hir::hir::EnumTy,
        w: &mut Twine,
    ) {
        let rust_ty = rust_primitive(enum_ty.ty);

        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::encode::EnumSerializer as _;\n\n");
        w!(w, "let state = ar.encode_enum(TYPE_INFO.name)?;\n");
        w!(w, "match self {\n");
        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            if let DefKind::Const(const_ty) = &field_def.kind {
                let value = Self::format_numeric(&const_ty.value);
                w!(w, "Self::", field_def, " => state.encode_variant::<", rust_ty, ">(\"", field_def, "\", ", value, "),\n");
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
        w!(w, "fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::decode::EnumDeserializer as _;\n\n");
        w!(w, "let state = ar.decode_enum(TYPE_INFO.name)?;\n");
        w!(w, "*self = state.decode_enumerator(*self)?;\n");
        w!(w, "Ok(())\n");
        w!(w, "}\n");
        w!(w, "}\n\n");

        w!(w, "impl ::intercom_cts::decode::EnumVisitor for ", def, " {\n");
        w!(w, "fn member_id<D>(self, de: D) -> ::std::result::Result<Self, D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer,\n");
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

        w!(w, "fn member_field<D>(self, name: &str) -> ::std::result::Result<Self, D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer,\n");
        w!(w, "{\n");
        w!(w, "use ::intercom_cts::error::Error as _;\n\n");
        w!(w, "let value = match name {\n");
        for &field_id in &enum_ty.fields {
            let field_def = self.hir.context.definitions.get(field_id);
            w!(w, "\"", field_def, "\" => Self::", field_def, ",\n");
        }

        w!(w, "_ => return Err(D::Error::custom(\"invalid enum value for type ", qual, "\")),\n");
        w!(w, "};\n");
        w!(w, "Ok(value)\n");
        w!(w, "}\n");
        w!(w, "}\n");
    }

    pub(crate) fn emit_union_member_info(
        _def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        w: &mut Twine,
    ) {
        // Discriminator has member_id 0, so start at 1 for members
        let mut member_id = 1;
        let member_info: Vec<_> = union_ty
            .variants
            .iter()
            .filter(|v| !matches!(v.ty.kind, TyKind::Null))
            .map(|v| {
                let id = member_id;
                member_id += 1;
                (v.ident.name.as_str(), id)
            })
            .collect();

        Self::emit_member_info_array(&member_info, w);
    }

    pub(crate) fn emit_union_marshal_impl(
        &self,
        def: &Def,
        union_ty: &ic_hir::hir::UnionTy,
        w: &mut Twine,
    ) {
        w!(w, "impl ::intercom_cts::Marshal for ", def, " {\n");
        w!(w, "fn marshal<S>(&self, ar: S) -> ::std::result::Result<S::Ok, S::Error>\n");
        w!(w, "where\n");
        w!(w, "\tS: ::intercom_cts::encode::Serializer,\n");
        w!(w, "{\n");

        w!(w, "use ::intercom_cts::encode::UnionSerializer as _;\n\n");
        w!(w, "let mut state = ar.encode_union(&TYPE_INFO)?;\n");
        w!(w, "state.encode_discriminant(&self.disc())?;\n");
        w!(w, "match self {\n");
        let mut info_idx = 0;
        for variant in &union_ty.variants {
            if variant.is_default {
                continue;
            }

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

        if union_ty.variants.iter().any(|v| v.is_default) {
            w!(w, "_ => state.encode_null(),\n");
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
        w!(w, "fn unmarshal_mut<D>(&mut self, ar: D) -> ::std::result::Result<(), D::Error>\n");
        w!(w, "where\n");
        w!(w, "\tD: ::intercom_cts::decode::Deserializer,\n");
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
                if matches!(variant.ty.kind, TyKind::Null) {
                    w!(w, "_ => Self::", variant.ident.name, ",\n");
                } else {
                    w!(w, "_ => {\n");
                    w!(w, "let mut value = ");
                    self.emit_default_value(&variant.ty, def.id, w);
                    w!(w, ";\n");
                    w!(w, "state.decode_variant(&MEMBER_INFO[", info_idx.to_string(), "], &mut value)?;\n");
                    w!(w, "Self::", variant.ident.name, "(value)\n");
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
