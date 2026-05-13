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

use std::collections::BTreeMap;

use ic_hir::Context;
use ic_hir::hir::{Ann, Def, DefFlags, DefId, DefKind, Numeric};
use ic_omgidl::types::xtypes::{
    ANNOTATION_STR_VALUE_MAX_LEN, AnnotationParameterValue, AppliedAnnotation,
    AppliedAnnotationParameter, AppliedBuiltinMemberAnnotations, AppliedBuiltinTypeAnnotations,
    AppliedVerbatimAnnotation, BitmaskTypeFlag, CompleteDiscriminatorMember, CompleteMemberDetail,
    CompleteTypeDetail, EnumTypeFlag, MemberFlag, MemberId, TypeFlag, TypeIdentifier,
};
use tracing::warn;

use crate::util::name_hash;

pub fn is_builtin_annotation(ctx: &Context, def_id: DefId) -> bool {
    ctx.type_of(def_id).flags.contains(DefFlags::IS_BUILTIN)
}

fn get_annotation<'a>(annotations: &'a [Ann], name: &str) -> Option<&'a Ann> {
    annotations
        .iter()
        .find(|ann| ann.ident.name.ends_with(name))
}

fn get_annotation_int_value(ann: &Ann, param_name: &str) -> Option<i32> {
    ann.args
        .iter()
        .find(|arg| arg.ident.name == param_name)
        .and_then(|arg| {
            if let Numeric::Int32(val) = arg.value {
                Some(val)
            } else {
                None
            }
        })
}

fn get_annotation_string_value<'a>(ann: &'a Ann, param_name: &str) -> Option<&'a str> {
    ann.args
        .iter()
        .find(|arg| arg.ident.name == param_name)
        .and_then(|arg| {
            if let Numeric::String(value) | Numeric::WString(value) = &arg.value {
                Some(value.as_str())
            } else {
                None
            }
        })
}

fn get_annotation_numeric_value<'a>(ann: &'a Ann, param_name: &str) -> Option<&'a Numeric> {
    ann.args
        .iter()
        .find(|arg| arg.ident.name == param_name)
        .map(|arg| &arg.value)
}

fn truncate_annotation_string(mut s: String) -> String {
    if s.len() > ANNOTATION_STR_VALUE_MAX_LEN as usize {
        warn!(
            original_len = s.len(),
            max_len = ANNOTATION_STR_VALUE_MAX_LEN,
            "annotation string value truncated"
        );
        s.truncate(ANNOTATION_STR_VALUE_MAX_LEN as usize);
    }
    s
}

#[allow(clippy::cast_sign_loss)]
pub fn numeric_to_annotation_value(ctx: &Context, num: &Numeric) -> AnnotationParameterValue {
    match num {
        Numeric::Bool(b) => AnnotationParameterValue::BooleanValue(*b),
        Numeric::Char(c) => AnnotationParameterValue::CharValue(*c),
        Numeric::WChar(c) => AnnotationParameterValue::WcharValue(*c),
        Numeric::Int8(v) => AnnotationParameterValue::TkInt8(*v as u8),
        Numeric::UInt8(v) => AnnotationParameterValue::TkUint8(*v),
        Numeric::Int16(v) => AnnotationParameterValue::Int16Value(*v),
        Numeric::UInt16(v) => AnnotationParameterValue::Uint16Value(*v),
        Numeric::Int32(v) => AnnotationParameterValue::Int32Value(*v),
        Numeric::UInt32(v) => AnnotationParameterValue::Uint32Value(*v),
        Numeric::Int64(v) => AnnotationParameterValue::Int64Value(*v),
        Numeric::UInt64(v) => AnnotationParameterValue::Uint64Value(*v),
        Numeric::Float(v) => AnnotationParameterValue::Float32Value(*v),
        Numeric::Double(v) => AnnotationParameterValue::Float64Value(*v),
        Numeric::String(s) => {
            AnnotationParameterValue::String8Value(truncate_annotation_string(s.clone()))
        }
        Numeric::WString(s) => {
            AnnotationParameterValue::String16Value(truncate_annotation_string(s.clone()))
        }
        Numeric::Const(def_id) => {
            let def = ctx.type_of(*def_id);
            if let DefKind::Const(const_def) = &def.kind {
                numeric_to_annotation_value(ctx, &const_def.value)
            } else {
                AnnotationParameterValue::default()
            }
        }
        _ => AnnotationParameterValue::default(),
    }
}

pub fn get_member_flags(_ctx: &Context, annotations: &[Ann]) -> MemberFlag {
    let mut flags = MemberFlag::new();

    if get_annotation(annotations, "optional").is_some() {
        flags |= MemberFlag::IS_OPTIONAL;
    }
    if get_annotation(annotations, "shared").is_some()
        || get_annotation(annotations, "external").is_some()
    {
        flags |= MemberFlag::IS_EXTERNAL;
    }
    if get_annotation(annotations, "must_understand").is_some() {
        flags |= MemberFlag::IS_MUST_UNDERSTAND;
    }
    if get_annotation(annotations, "key").is_some() {
        flags |= MemberFlag::IS_KEY;
    }

    if let Some(ann) = get_annotation(annotations, "try_construct") {
        if let Some(val) = get_annotation_int_value(ann, "value") {
            match val {
                0 => flags |= MemberFlag::TRY_CONSTRUCT1,
                1 => flags |= MemberFlag::TRY_CONSTRUCT2,
                2 => flags |= MemberFlag::TRY_CONSTRUCT1 | MemberFlag::TRY_CONSTRUCT2,
                _ => {}
            }
        }
    } else {
        flags |= MemberFlag::TRY_CONSTRUCT1;
    }

    flags
}

pub fn get_literal_flags(_ctx: &Context, annotations: &[Ann]) -> MemberFlag {
    let mut flags = MemberFlag::new();
    if get_annotation(annotations, "default_literal").is_some() {
        flags |= MemberFlag::IS_DEFAULT;
    }
    flags
}

fn get_extensibility_from_annotations(annotations: &[Ann]) -> TypeFlag {
    if get_annotation(annotations, "final").is_some() {
        return TypeFlag::IS_FINAL;
    }
    if get_annotation(annotations, "mutable").is_some() {
        return TypeFlag::IS_MUTABLE;
    }
    TypeFlag::IS_APPENDABLE
}

pub fn get_struct_flags(def: &Def) -> TypeFlag {
    let mut flags = TypeFlag::new();

    if get_annotation(&def.annotations, "nested").is_some() {
        flags |= TypeFlag::IS_NESTED;
    }

    flags |= get_extensibility_from_annotations(&def.annotations);

    if get_annotation(&def.annotations, "autoid").is_some() {
        flags |= TypeFlag::IS_AUTOID_HASH;
    }

    flags
}

pub fn get_union_flags(def: &Def) -> TypeFlag {
    get_struct_flags(def)
}

pub fn get_enumerated_flags(def: &Def) -> EnumTypeFlag {
    get_extensibility_from_annotations(&def.annotations)
}

pub fn get_bitmask_flags(def: &Def) -> BitmaskTypeFlag {
    get_extensibility_from_annotations(&def.annotations)
}

pub fn get_bitset_flags(def: &Def) -> TypeFlag {
    let extensibility = get_extensibility_from_annotations(&def.annotations);
    if extensibility == TypeFlag::IS_FINAL {
        TypeFlag::IS_FINAL
    } else {
        TypeFlag::IS_APPENDABLE
    }
}

#[allow(clippy::cast_sign_loss)]
pub fn get_member_id(_ctx: &Context, annotations: &[Ann], current_id: MemberId) -> MemberId {
    if let Some(ann) = get_annotation(annotations, "id")
        && let Some(val) = get_annotation_int_value(ann, "value")
    {
        return val as u32;
    }
    current_id.wrapping_add(1)
}

pub fn get_parents_last_member_id(ctx: &Context, def_id: DefId) -> MemberId {
    let def = ctx.type_of(def_id);

    let parent_id = match &def.kind {
        DefKind::Struct(s) => s.parent,
        DefKind::Bitset(b) => b.parent,
        _ => None,
    };

    if let Some(parent) = parent_id {
        let parent_last = get_parents_last_member_id(ctx, parent.def_id);
        let parent_def = ctx.type_of(parent.def_id);

        let mut current_id = parent_last;
        if let DefKind::Struct(s) = &parent_def.kind {
            for member in &s.members {
                current_id = get_member_id(ctx, &member.annotations, current_id);
            }
        }
        current_id
    } else {
        MemberId::MAX
    }
}

#[allow(clippy::cast_sign_loss)]
pub fn get_bit_bound_for_enum(def: &Def) -> u16 {
    if let Some(ann) = get_annotation(&def.annotations, "bit_bound")
        && let Some(val) = get_annotation_int_value(ann, "value")
    {
        return val as u16;
    }
    32
}

#[allow(clippy::cast_sign_loss)]
pub fn get_bit_bound_for_bitmask(def: &Def) -> u16 {
    if let Some(ann) = get_annotation(&def.annotations, "bit_bound")
        && let Some(val) = get_annotation_int_value(ann, "value")
    {
        return val as u16;
    }
    32
}

#[allow(clippy::cast_sign_loss)]
pub fn get_bitfield_position(_ctx: &Context, annotations: &[Ann]) -> u16 {
    if let Some(ann) = get_annotation(annotations, "position")
        && let Some(val) = get_annotation_int_value(ann, "value")
    {
        return val as u16;
    }
    0
}

fn populate_annotation_details(
    ctx: &Context,
    type_id_map: &BTreeMap<DefId, TypeIdentifier>,
    annotations: &[Ann],
    detail: &mut impl AnnotationDetail,
) {
    for ann in annotations {
        let Some(def_id) = ann.def_id else {
            continue;
        };

        if is_builtin_annotation(ctx, def_id) {
            detail.add_builtin_annotation(ctx, ann);
        } else if let Some(type_id) = type_id_map.get(&def_id) {
            let mut applied_ann = AppliedAnnotation::new();
            applied_ann.annotation_typeid = type_id.clone();

            let mut params: Vec<_> = ann
                .args
                .iter()
                .map(|arg| {
                    let mut param = AppliedAnnotationParameter::new();
                    param.paramname_hash = name_hash(&arg.ident.name);
                    param.value = numeric_to_annotation_value(ctx, &arg.value);
                    param
                })
                .collect();

            params.sort_by_key(|a| a.paramname_hash);
            applied_ann.param_seq = Some(params);

            detail.add_custom_annotation(applied_ann);
        }
    }
}

trait AnnotationDetail {
    fn add_builtin_annotation(&mut self, ctx: &Context, ann: &Ann);
    fn add_custom_annotation(&mut self, applied_ann: AppliedAnnotation);
}

impl AnnotationDetail for CompleteTypeDetail {
    fn add_builtin_annotation(&mut self, ctx: &Context, ann: &Ann) {
        let Some(def_id) = ann.def_id else {
            return;
        };
        let name = ctx.qualified_name(def_id);

        if name.ends_with("::verbatim") {
            let builtin = self
                .ann_builtin
                .get_or_insert_with(AppliedBuiltinTypeAnnotations::new);
            let mut verbatim = AppliedVerbatimAnnotation::new();
            if let Some(val) = get_annotation_string_value(ann, "placement") {
                verbatim.placement = val.to_string();
            }
            if let Some(val) = get_annotation_string_value(ann, "language") {
                verbatim.language = val.to_string();
            }
            if let Some(val) = get_annotation_string_value(ann, "text") {
                verbatim.text = val.to_string();
            }
            builtin.verbatim = Some(verbatim);
        }
    }

    fn add_custom_annotation(&mut self, applied_ann: AppliedAnnotation) {
        self.ann_custom
            .get_or_insert_with(Vec::new)
            .push(applied_ann);
    }
}

impl AnnotationDetail for CompleteMemberDetail {
    fn add_builtin_annotation(&mut self, ctx: &Context, ann: &Ann) {
        let Some(def_id) = ann.def_id else {
            return;
        };
        let name = ctx.qualified_name(def_id);

        let builtin = self
            .ann_builtin
            .get_or_insert_with(AppliedBuiltinMemberAnnotations::new);
        match name.as_str() {
            s if s.ends_with("::min") => {
                if let Some(val) = get_annotation_numeric_value(ann, "value") {
                    builtin.min = Some(numeric_to_annotation_value(ctx, val));
                }
            }
            s if s.ends_with("::max") => {
                if let Some(val) = get_annotation_numeric_value(ann, "value") {
                    builtin.max = Some(numeric_to_annotation_value(ctx, val));
                }
            }
            s if s.ends_with("::unit") => {
                if let Some(val) = get_annotation_string_value(ann, "value") {
                    builtin.unit = Some(val.to_string());
                }
            }
            s if s.ends_with("::hash_id") => {
                if let Some(val) = get_annotation_string_value(ann, "value") {
                    builtin.hash_id = Some(val.to_string());
                }
            }
            _ => {}
        }
    }

    fn add_custom_annotation(&mut self, applied_ann: AppliedAnnotation) {
        self.ann_custom
            .get_or_insert_with(Vec::new)
            .push(applied_ann);
    }
}

pub fn create_complete_type_detail(
    ctx: &Context,
    type_id_map: &BTreeMap<DefId, TypeIdentifier>,
    def: &Def,
) -> CompleteTypeDetail {
    let mut detail = CompleteTypeDetail::new();
    detail.type_name = ctx.qualified_name(def.id);
    populate_annotation_details(ctx, type_id_map, &def.annotations, &mut detail);
    detail
}

pub fn create_complete_member_detail(
    ctx: &Context,
    type_id_map: &BTreeMap<DefId, TypeIdentifier>,
    name: &str,
    annotations: &[Ann],
) -> CompleteMemberDetail {
    let mut detail = CompleteMemberDetail::new();
    detail.name = name.to_string();
    populate_annotation_details(ctx, type_id_map, annotations, &mut detail);
    detail
}

pub fn populate_discriminator_annotations(
    ctx: &Context,
    type_id_map: &BTreeMap<DefId, TypeIdentifier>,
    annotations: &[Ann],
    discriminator: &mut CompleteDiscriminatorMember,
) {
    populate_annotation_details(ctx, type_id_map, annotations, discriminator);
}

impl AnnotationDetail for CompleteDiscriminatorMember {
    fn add_builtin_annotation(&mut self, ctx: &Context, ann: &Ann) {
        let Some(def_id) = ann.def_id else {
            return;
        };
        let name = ctx.qualified_name(def_id);

        if name.ends_with("::verbatim") {
            let builtin = self
                .ann_builtin
                .get_or_insert_with(AppliedBuiltinTypeAnnotations::new);
            let mut verbatim = AppliedVerbatimAnnotation::new();
            if let Some(val) = get_annotation_string_value(ann, "placement") {
                verbatim.placement = val.to_string();
            }
            if let Some(val) = get_annotation_string_value(ann, "language") {
                verbatim.language = val.to_string();
            }
            if let Some(val) = get_annotation_string_value(ann, "text") {
                verbatim.text = val.to_string();
            }
            builtin.verbatim = Some(verbatim);
        }
    }

    fn add_custom_annotation(&mut self, applied_ann: AppliedAnnotation) {
        self.ann_custom
            .get_or_insert_with(Vec::new)
            .push(applied_ann);
    }
}
