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

//! Fold trait for transforming HIR nodes.
//!
//! The fold pattern allows mutation of the HIR tree by consuming nodes
//! and producing new ones. This avoids the borrowing complexity of mutable
//! visitors while still allowing arbitrary transformations.

use crate::hir::{
    AliasTy, Ann, AnnotationTy, Attribute, BitmaskTy, BitsetTy, ConstTy, Decl, Def, DefKind,
    EnumTy, ExceptTy, InterfaceTy, Member, ModuleTy, Numeric, Parameter, ProtoTy, StructTy, Ty,
    TyKind, UnionTy, ValueTy, Variant,
};

/// Trait for folding (transforming) HIR nodes at the type level.
pub trait Fold {
    /// Transform a definition. Override to modify definitions.
    fn fold_def(&mut self, def: Def) -> Def {
        fold_def(self, def)
    }

    /// Transform a declaration. Override to modify declarations.
    fn fold_decl(&mut self, decl: Decl) -> Decl {
        decl
    }

    /// Transform a type. Override to modify types.
    fn fold_ty(&mut self, ty: Ty) -> Ty {
        fold_ty(self, ty)
    }

    /// Transform a numeric value. Override to modify numeric values.
    fn fold_numeric(&mut self, num: Numeric) -> Numeric {
        fold_numeric(self, num)
    }

    /// Transform a struct type.
    fn fold_struct_ty(&mut self, s: StructTy) -> StructTy {
        fold_struct_ty(self, s)
    }

    /// Transform an enum type.
    fn fold_enum_ty(&mut self, e: EnumTy) -> EnumTy {
        fold_enum_ty(self, e)
    }

    /// Transform a union type.
    fn fold_union_ty(&mut self, u: UnionTy) -> UnionTy {
        fold_union_ty(self, u)
    }

    /// Transform an alias type.
    fn fold_alias_ty(&mut self, a: AliasTy) -> AliasTy {
        fold_alias_ty(self, a)
    }

    /// Transform a bitmask type.
    fn fold_bitmask_ty(&mut self, b: BitmaskTy) -> BitmaskTy {
        fold_bitmask_ty(self, b)
    }

    /// Transform a bitset type.
    fn fold_bitset_ty(&mut self, b: BitsetTy) -> BitsetTy {
        fold_bitset_ty(self, b)
    }

    /// Transform a const type.
    fn fold_const_ty(&mut self, c: ConstTy) -> ConstTy {
        fold_const_ty(self, c)
    }

    /// Transform an interface type.
    fn fold_interface_ty(&mut self, i: InterfaceTy) -> InterfaceTy {
        fold_interface_ty(self, i)
    }

    /// Transform a valuetype.
    fn fold_valuetype(&mut self, v: ValueTy) -> ValueTy {
        fold_valuetype(self, v)
    }

    /// Transform an except type.
    fn fold_except_ty(&mut self, e: ExceptTy) -> ExceptTy {
        fold_except_ty(self, e)
    }

    /// Transform a module type.
    fn fold_module_ty(&mut self, m: ModuleTy) -> ModuleTy {
        fold_module_ty(self, m)
    }

    /// Transform an annotation type.
    fn fold_annotation_ty(&mut self, a: AnnotationTy) -> AnnotationTy {
        fold_annotation_ty(self, a)
    }

    /// Transform a member.
    fn fold_member(&mut self, m: Member) -> Member {
        fold_member(self, m)
    }

    /// Transform a variant.
    fn fold_variant(&mut self, v: Variant) -> Variant {
        fold_variant(self, v)
    }

    /// Transform a parameter.
    fn fold_parameter(&mut self, p: Parameter) -> Parameter {
        fold_parameter(self, p)
    }

    /// Transform a prototype.
    fn fold_proto_ty(&mut self, p: ProtoTy) -> ProtoTy {
        fold_proto_ty(self, p)
    }

    /// Transform an annotation.
    fn fold_annotation(&mut self, a: Ann) -> Ann {
        fold_annotation(self, a)
    }

    /// Transform an attribute.
    fn fold_attribute(&mut self, a: Attribute) -> Attribute {
        fold_attribute(self, a)
    }
}

pub fn fold_def<F: Fold + ?Sized>(folder: &mut F, mut def: Def) -> Def {
    def.annotations = def
        .annotations
        .into_iter()
        .map(|a| folder.fold_annotation(a))
        .collect();

    def.kind = match def.kind {
        DefKind::Annotation(a) => DefKind::Annotation(folder.fold_annotation_ty(a)),
        DefKind::Module(m) => DefKind::Module(folder.fold_module_ty(m)),
        DefKind::Struct(s) => DefKind::Struct(folder.fold_struct_ty(s)),
        DefKind::Except(e) => DefKind::Except(folder.fold_except_ty(e)),
        DefKind::Union(u) => DefKind::Union(folder.fold_union_ty(u)),
        DefKind::Enum(e) => DefKind::Enum(folder.fold_enum_ty(e)),
        DefKind::Const(c) => DefKind::Const(folder.fold_const_ty(c)),
        DefKind::Bitmask(b) => DefKind::Bitmask(folder.fold_bitmask_ty(b)),
        DefKind::Bitset(b) => DefKind::Bitset(folder.fold_bitset_ty(b)),
        DefKind::Alias(a) => DefKind::Alias(folder.fold_alias_ty(a)),
        DefKind::Interface(i) => DefKind::Interface(folder.fold_interface_ty(i)),
        DefKind::Valuetype(v) => DefKind::Valuetype(folder.fold_valuetype(v)),
        DefKind::Decl(d) => DefKind::Decl(folder.fold_decl(d)),
    };

    def
}

pub fn fold_ty<F: Fold + ?Sized>(folder: &mut F, mut ty: Ty) -> Ty {
    ty.kind = match ty.kind {
        TyKind::Primitive(p) => TyKind::Primitive(p),
        TyKind::String {
            wide,
            bound,
            bound_span,
        } => TyKind::String {
            wide,
            bound,
            bound_span,
        },
        TyKind::Array {
            ty: inner,
            len,
            len_span,
        } => TyKind::Array {
            ty: Box::new(folder.fold_ty(*inner)),
            len,
            len_span,
        },
        TyKind::Sequence {
            ty: inner,
            bound,
            bound_span,
        } => TyKind::Sequence {
            ty: Box::new(folder.fold_ty(*inner)),
            bound,
            bound_span,
        },
        TyKind::Map {
            key,
            elem,
            bound,
            bound_span,
        } => TyKind::Map {
            key: Box::new(folder.fold_ty(*key)),
            elem: Box::new(folder.fold_ty(*elem)),
            bound,
            bound_span,
        },
        TyKind::Fixed => TyKind::Fixed,
        TyKind::Any => TyKind::Any,
        TyKind::Null => TyKind::Null,
        TyKind::Adt(id) => TyKind::Adt(id),
    };
    ty
}

pub fn fold_numeric<F: Fold + ?Sized>(folder: &mut F, num: Numeric) -> Numeric {
    match num {
        Numeric::Array { values, ty } => Numeric::Array {
            values: values
                .into_vec()
                .into_iter()
                .map(|v| folder.fold_numeric(v))
                .collect(),
            ty: folder.fold_ty(ty),
        },
        Numeric::Sequence { values, ty } => Numeric::Sequence {
            values: values
                .into_vec()
                .into_iter()
                .map(|v| folder.fold_numeric(v))
                .collect(),
            ty: folder.fold_ty(ty),
        },
        Numeric::Map {
            entries,
            key,
            value,
        } => Numeric::Map {
            entries: entries
                .into_vec()
                .into_iter()
                .map(|(k, v)| (folder.fold_numeric(k), folder.fold_numeric(v)))
                .collect(),
            key: folder.fold_ty(key),
            value: folder.fold_ty(value),
        },
        Numeric::Struct { ty, fields } => Numeric::Struct {
            ty,
            fields: fields
                .into_vec()
                .into_iter()
                .map(|(name, val)| (name, folder.fold_numeric(val)))
                .collect(),
        },
        Numeric::Union {
            ty,
            discriminant,
            field,
            value,
        } => Numeric::Union {
            ty,
            discriminant: Box::new(folder.fold_numeric(*discriminant)),
            field,
            value: Box::new(folder.fold_numeric(*value)),
        },
        Numeric::Const(id) => Numeric::Const(id),
        // Primitive numeric types remain unchanged
        n => n,
    }
}

pub fn fold_struct_ty<F: Fold + ?Sized>(folder: &mut F, mut s: StructTy) -> StructTy {
    s.members = s
        .members
        .into_iter()
        .map(|m| folder.fold_member(m))
        .collect();
    s.parent = s.parent.map(|id| id);
    s
}

pub fn fold_enum_ty<F: Fold + ?Sized>(folder: &mut F, mut e: EnumTy) -> EnumTy {
    e.fields = e.fields.into_iter().map(|id| id).collect();
    e
}

pub fn fold_union_ty<F: Fold + ?Sized>(folder: &mut F, mut u: UnionTy) -> UnionTy {
    u.disc.ty = folder.fold_ty(u.disc.ty);
    u.disc.annotations = u
        .disc
        .annotations
        .into_iter()
        .map(|a| folder.fold_annotation(a))
        .collect();
    u.variants = u
        .variants
        .into_iter()
        .map(|v| folder.fold_variant(v))
        .collect();
    u
}

pub fn fold_alias_ty<F: Fold + ?Sized>(folder: &mut F, mut a: AliasTy) -> AliasTy {
    a.ty = folder.fold_ty(a.ty);
    a
}

pub fn fold_bitmask_ty<F: Fold + ?Sized>(folder: &mut F, b: BitmaskTy) -> BitmaskTy {
    b
}

pub fn fold_bitset_ty<F: Fold + ?Sized>(folder: &mut F, mut b: BitsetTy) -> BitsetTy {
    b.parent = b.parent.map(|id| id);
    for field in &mut b.fields {
        field.ty = folder.fold_ty(field.ty.clone());
        field.annotations = field
            .annotations
            .clone()
            .into_iter()
            .map(|a| folder.fold_annotation(a))
            .collect();
    }
    b
}

pub fn fold_const_ty<F: Fold + ?Sized>(folder: &mut F, mut c: ConstTy) -> ConstTy {
    c.ty = folder.fold_ty(c.ty);
    c.value = folder.fold_numeric(c.value);
    c
}

pub fn fold_interface_ty<F: Fold + ?Sized>(folder: &mut F, mut i: InterfaceTy) -> InterfaceTy {
    i.parents = i.parents.into_iter().map(|id| id).collect();
    i.prototypes = i
        .prototypes
        .into_iter()
        .map(|p| folder.fold_proto_ty(p))
        .collect();
    i.attributes = i
        .attributes
        .into_iter()
        .map(|a| folder.fold_attribute(a))
        .collect();
    i.definitions = i.definitions.into_iter().map(|id| id).collect();
    i
}

pub fn fold_valuetype<F: Fold + ?Sized>(folder: &mut F, mut v: ValueTy) -> ValueTy {
    v.parent = v.parent.map(|id| id);
    v.supports = v.supports.map(|id| id);
    v.prototypes = v
        .prototypes
        .into_iter()
        .map(|p| folder.fold_proto_ty(p))
        .collect();
    v.attributes = v
        .attributes
        .into_iter()
        .map(|a| folder.fold_attribute(a))
        .collect();
    v.members = v
        .members
        .into_iter()
        .map(|m| folder.fold_member(m))
        .collect();
    v.definitions = v.definitions.into_iter().map(|id| id).collect();
    v
}

pub fn fold_except_ty<F: Fold + ?Sized>(folder: &mut F, mut e: ExceptTy) -> ExceptTy {
    e.members = e
        .members
        .into_iter()
        .map(|m| folder.fold_member(m))
        .collect();
    e
}

pub fn fold_module_ty<F: Fold + ?Sized>(_folder: &mut F, mut m: ModuleTy) -> ModuleTy {
    m.definitions = m.definitions.into_iter().map(|id| id).collect();
    m
}

pub fn fold_annotation_ty<F: Fold + ?Sized>(folder: &mut F, mut a: AnnotationTy) -> AnnotationTy {
    for param in &mut a.params {
        param.ty = folder.fold_ty(param.ty.clone());
        if let Some(ref mut default) = param.default {
            *default = folder.fold_numeric(default.clone());
        }
    }
    a.types = a.types.into_iter().map(|id| id).collect();
    a
}

pub fn fold_member<F: Fold + ?Sized>(folder: &mut F, mut m: Member) -> Member {
    m.ty = folder.fold_ty(m.ty);
    m.annotations = m
        .annotations
        .into_iter()
        .map(|a| folder.fold_annotation(a))
        .collect();
    m
}

pub fn fold_variant<F: Fold + ?Sized>(folder: &mut F, mut v: Variant) -> Variant {
    v.ty = folder.fold_ty(v.ty);
    for label in &mut v.labels {
        label.value = folder.fold_numeric(label.value.clone());
    }
    v.annotations = v
        .annotations
        .into_iter()
        .map(|a| folder.fold_annotation(a))
        .collect();
    v
}

pub fn fold_parameter<F: Fold + ?Sized>(folder: &mut F, mut p: Parameter) -> Parameter {
    p.ty = folder.fold_ty(p.ty);
    p
}

pub fn fold_proto_ty<F: Fold + ?Sized>(folder: &mut F, mut p: ProtoTy) -> ProtoTy {
    p.ty = folder.fold_ty(p.ty);
    p.params = p
        .params
        .into_iter()
        .map(|param| folder.fold_parameter(param))
        .collect();
    p
}

pub fn fold_annotation<F: Fold + ?Sized>(folder: &mut F, mut a: Ann) -> Ann {
    a.def_id = a.def_id.map(|id| id);
    for arg in &mut a.args {
        arg.value = folder.fold_numeric(arg.value.clone());
        if let Some(ref mut ty) = arg.ty {
            *ty = folder.fold_ty(ty.clone());
        }
    }
    a
}

pub fn fold_attribute<F: Fold + ?Sized>(folder: &mut F, mut a: Attribute) -> Attribute {
    a.ty = folder.fold_ty(a.ty);
    a
}
