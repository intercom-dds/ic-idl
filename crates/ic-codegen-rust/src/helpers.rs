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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::fmt::Write;

use ic_hir::annotation::{Optional, find_annotation};
use ic_hir::hir::{Def, DefFlags, DefId, DefKind, Member, Numeric, PrimitiveTy, Ty, TyKind};

use crate::codegen::RustGen;

impl RustGen<'_> {
    pub fn original_name(&self, def_id: DefId) -> &str {
        &self.original_hir.context.definitions.get(def_id).ident.name
    }

    pub fn original_qualified_name(&self, def_id: DefId) -> String {
        self.original_hir.context.qualified_name(def_id)
    }

    pub fn scoped_name(&self, target_def_id: DefId, _relative_to_def_id: DefId) -> String {
        let def = self.hir.context.definitions.get(target_def_id);

        let mut scope = Vec::new();
        let mut current_id = def.parent;

        while let Some(id) = current_id {
            let parent_def = self.hir.context.definitions.get(id);
            if matches!(
                parent_def.kind,
                DefKind::Module(_) | DefKind::Enum(_) | DefKind::Bitmask(_)
            ) {
                scope.push(parent_def.ident.name.clone());
            }
            current_id = parent_def.parent;
        }

        let mut path = String::from("crate");
        for name in scope.iter().rev() {
            path.push_str("::");
            path.push_str(name);
        }
        path.push_str("::");
        path.push_str(&def.ident.name);

        path
    }

    pub(crate) fn struct_members(
        &self,
        struct_ty: &ic_hir::hir::StructTy,
    ) -> Vec<ic_hir::hir::Member> {
        let mut members = Vec::new();
        if let Some(parent) = struct_ty.parent {
            let parent_def = self.hir.context.definitions.get(parent.def_id);
            if let DefKind::Struct(parent_struct) = &parent_def.kind {
                members.extend(self.struct_members(parent_struct));
            }
        }
        members.extend(struct_ty.members.clone());
        members
    }

    pub(crate) fn original_struct_members(
        &self,
        struct_ty: &ic_hir::hir::StructTy,
    ) -> Vec<ic_hir::hir::Member> {
        let mut members = Vec::new();
        if let Some(parent) = struct_ty.parent {
            let parent_def = self.original_hir.context.definitions.get(parent.def_id);
            if let DefKind::Struct(parent_struct) = &parent_def.kind {
                members.extend(self.original_struct_members(parent_struct));
            }
        }
        members.extend(struct_ty.members.clone());
        members
    }

    pub fn member_type(&self, ty: &Ty) -> String {
        self.rust_type(ty, self.hir.order[0])
    }

    pub(crate) fn valuetype_members(
        &self,
        value_ty: &ic_hir::hir::ValueTy,
    ) -> Vec<ic_hir::hir::Member> {
        let mut members = Vec::new();
        if let Some(parent) = value_ty.parent {
            let parent_def = self.hir.context.definitions.get(parent.def_id);
            if let DefKind::Valuetype(parent_value) = &parent_def.kind {
                members.extend(self.valuetype_members(parent_value));
            }
        }
        members.extend(value_ty.members.clone());
        members
    }

    pub(crate) fn original_valuetype_members(
        &self,
        value_ty: &ic_hir::hir::ValueTy,
    ) -> Vec<ic_hir::hir::Member> {
        let mut members = Vec::new();
        if let Some(parent) = value_ty.parent {
            let parent_def = self.original_hir.context.definitions.get(parent.def_id);
            if let DefKind::Valuetype(parent_value) = &parent_def.kind {
                members.extend(self.original_valuetype_members(parent_value));
            }
        }
        members.extend(value_ty.members.clone());
        members
    }

    pub fn is_copy_type(&self, ty: &Ty) -> bool {
        match &ty.kind {
            TyKind::Primitive(_) => true,
            TyKind::Array { ty: elem_ty, .. } => self.is_copy_type(elem_ty),
            TyKind::Adt(def_id) => {
                let def = self.hir.context.definitions.get(*def_id);
                matches!(def.kind, DefKind::Enum(_) | DefKind::Bitmask(_))
            }
            _ => false,
        }
    }

    pub fn is_string_const_literal(&self, ty: &Ty, value: &Numeric) -> bool {
        let resolved_ty = self.hir.context.resolve_ty(ty);
        if !matches!(resolved_ty.kind, TyKind::String { .. }) {
            return false;
        }
        match value {
            Numeric::String(_) | Numeric::WString(_) => true,
            Numeric::Const(def_id) => {
                let const_def = self.hir.context.definitions.get(*def_id);
                if let DefKind::Const(const_ty) = &const_def.kind {
                    self.is_string_const_literal(&const_ty.ty, &const_ty.value)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

pub fn is_trivial(def: &Def) -> bool {
    def.flags.contains(DefFlags::IS_TRIVIAL)
}

pub fn is_copy(def: &Def) -> bool {
    is_trivial(def)
}

pub fn is_debug(def: &Def) -> bool {
    // Bitmasks do not derive debug. We provide a nicer implementation of
    // the `Debug` trait which emits the name of the constants instead of
    // just the underlying value.
    !matches!(def.kind, DefKind::Bitmask(_))
}

pub fn is_ord(def: &Def) -> bool {
    def.flags.contains(DefFlags::TOTAL_ORDER)
}

pub fn is_eq(def: &Def) -> bool {
    is_ord(def)
}

pub fn is_hash(def: &Def) -> bool {
    is_ord(def)
}

/// Check if a member has the @optional annotation
pub fn is_optional(member: &Member) -> bool {
    find_annotation::<Optional>(&member.annotations, "optional")
        .and_then(Result::ok)
        .is_some_and(|opt| opt.value)
}

pub fn is_must_understand(member: &Member) -> bool {
    member
        .annotations
        .iter()
        .any(|a| a.ident.name == "must_understand")
}

pub fn is_key(member: &Member) -> bool {
    member.annotations.iter().any(|a| a.ident.name == "key")
}

pub fn is_shared(member: &Member) -> bool {
    member
        .annotations
        .iter()
        .any(|a| a.ident.name == "shared" || a.ident.name == "external")
}

pub fn default_value(member: &Member) -> &Numeric {
    static NULL: Numeric = Numeric::Null;

    member
        .annotations
        .iter()
        .find(|ann| ann.ident.name == "default")
        .and_then(|ann| ann.args.first())
        .map_or(&NULL, |arg| &arg.value)
}

pub fn member_id(member: &Member, default_id: usize) -> usize {
    member
        .annotations
        .iter()
        .find(|ann| ann.ident.name == "id")
        .and_then(|ann| ann.args.first())
        .and_then(|arg| match &arg.value {
            Numeric::UInt32(v) => Some(*v as usize),
            _ => None,
        })
        .unwrap_or(default_id)
}

pub fn format_integer(val: i128) -> String {
    let s = val.to_string();
    let (sign, digits) = s
        .strip_prefix('-')
        .map_or(("", s.as_str()), |rest| ("-", rest));

    let mut result = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('_');
        }
        result.push(ch);
    }

    format!("{}{}", sign, result.chars().rev().collect::<String>())
}

pub fn rust_primitive(ty: PrimitiveTy) -> &'static str {
    match ty {
        PrimitiveTy::Void => "()",
        PrimitiveTy::Bool => "bool",
        PrimitiveTy::Int8 => "i8",
        PrimitiveTy::UInt8 => "u8",
        PrimitiveTy::Int16 => "i16",
        PrimitiveTy::UInt16 => "u16",
        PrimitiveTy::Int32 => "i32",
        PrimitiveTy::UInt32 => "u32",
        PrimitiveTy::Int64 => "i64",
        PrimitiveTy::UInt64 => "u64",
        PrimitiveTy::Float32 => "f32",
        PrimitiveTy::Float64 | PrimitiveTy::Float128 => "f64",
        PrimitiveTy::Char | PrimitiveTy::WChar => "char",
    }
}

fn is_nested(def: &Def) -> bool {
    def.annotations.iter().any(|a| a.ident.name == "nested")
}

fn is_autoid_hash(ctx: &ic_hir::Context, def: &Def) -> bool {
    def.annotations.iter().any(|a| {
        a.ident.name == "autoid"
            && a.args.first().is_some_and(|arg| {
                if let Numeric::Const(def_id) = &arg.value {
                    ctx.type_of(*def_id).ident.name == "HASH"
                } else {
                    false
                }
            })
    })
}

fn has_key_member(def: &Def) -> bool {
    let members = match &def.kind {
        DefKind::Struct(s) => &s.members,
        DefKind::Valuetype(v) => &v.members,
        DefKind::Except(e) => &e.members,
        _ => return false,
    };
    members.iter().any(is_key)
}

pub fn type_flags(ctx: &ic_hir::Context, def: &Def) -> String {
    let mut flags = Vec::new();

    let is_final = def.annotations.iter().any(|a| a.ident.name == "final");
    let is_mutable = def.annotations.iter().any(|a| a.ident.name == "mutable");

    if is_final {
        flags.push("IS_FINAL");
    } else if is_mutable {
        flags.push("IS_MUTABLE");
    } else {
        flags.push("IS_APPENDABLE");
    }

    if is_nested(def) {
        flags.push("IS_NESTED");
    }

    if is_autoid_hash(ctx, def) {
        flags.push("IS_AUTOID_HASH");
    }

    if has_key_member(def) {
        flags.push("IS_KEYED");
    }

    let mut result = format!("::intercom_cts::TypeFlag::{}", flags[0]);
    for flag in &flags[1..] {
        _ = write!(result, ".union(::intercom_cts::TypeFlag::{flag})");
    }
    result
}
