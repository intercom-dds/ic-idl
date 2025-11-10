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

#![allow(clippy::cast_sign_loss)]

use ic_hir::hir::{Def, DefFlags, DefId, DefKind, Numeric, PrimitiveTy, Ty, TyKind};

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
        if let Some(parent_id) = struct_ty.parent {
            let parent_def = self.hir.context.definitions.get(parent_id);
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
        if let Some(parent_id) = struct_ty.parent {
            let parent_def = self.original_hir.context.definitions.get(parent_id);
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
        if let Some(parent_id) = value_ty.parent {
            let parent_def = self.hir.context.definitions.get(parent_id);
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
        if let Some(parent_id) = value_ty.parent {
            let parent_def = self.original_hir.context.definitions.get(parent_id);
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
}

pub fn is_trivial(def: &Def) -> bool {
    // TODO: string constants
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

pub fn log2(value: &Numeric) -> u64 {
    let val = match value {
        Numeric::UInt8(v) => u64::from(*v),
        Numeric::UInt16(v) => u64::from(*v),
        Numeric::UInt32(v) => u64::from(*v),
        Numeric::UInt64(v) => *v,
        Numeric::Int8(v) => *v as u64,
        Numeric::Int16(v) => *v as u64,
        Numeric::Int32(v) => *v as u64,
        Numeric::Int64(v) => *v as u64,
        _ => 0,
    };

    let mut res = 0;
    let mut v = val;
    while v > 1 {
        v >>= 1;
        res += 1;
    }
    res
}
