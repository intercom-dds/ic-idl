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

use crate::hir::{
    AliasTy, Ann, AnnotationTy, BitmaskTy, BitsetTy, ConstTy, Decl, Def, DefKind, EnumTy, ExceptTy,
    InterfaceTy, Member, ModuleTy, Numeric, Parameter, ProtoTy, StructTy, Ty, TyKind, UnionTy,
    ValueTy, Variant,
};

pub trait Visitor<'a> {
    fn visit_def(&mut self, def: &'a Def) {
        walk_def(self, def);
    }

    fn visit_annotation_def(&mut self, def: &'a Def, data: &'a AnnotationTy) {
        walk_annotation_def(self, def, data);
    }

    fn visit_module(&mut self, def: &'a Def, data: &'a ModuleTy) {
        walk_module(self, def, data);
    }

    fn visit_struct(&mut self, _def: &'a Def, data: &'a StructTy) {
        walk_struct(self, data);
    }

    fn visit_except(&mut self, _def: &'a Def, data: &'a ExceptTy) {
        walk_except(self, data);
    }

    fn visit_enum(&mut self, _def: &'a Def, data: &'a EnumTy) {
        walk_enum(self, data);
    }

    fn visit_union(&mut self, _def: &'a Def, data: &'a UnionTy) {
        walk_union(self, data);
    }

    fn visit_alias(&mut self, _def: &'a Def, data: &'a AliasTy) {
        walk_alias(self, data);
    }

    fn visit_bitmask(&mut self, _def: &'a Def, data: &'a BitmaskTy) {
        walk_bitmask(self, data);
    }

    fn visit_bitset(&mut self, _def: &'a Def, data: &'a BitsetTy) {
        walk_bitset(self, data);
    }

    fn visit_const(&mut self, _def: &'a Def, data: &'a ConstTy) {
        walk_const(self, data);
    }

    fn visit_interface(&mut self, def: &'a Def, data: &'a InterfaceTy) {
        walk_interface(self, def, data);
    }

    fn visit_valuetype(&mut self, def: &'a Def, data: &'a ValueTy) {
        walk_valuetype(self, def, data);
    }

    fn visit_decl(&mut self, _def: &'a Def, data: &'a Decl) {
        walk_decl(self, data);
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        walk_ty(self, ty);
    }

    fn visit_numeric(&mut self, num: &'a Numeric) {
        walk_numeric(self, num);
    }

    fn visit_member(&mut self, member: &'a Member) {
        walk_member(self, member);
    }

    fn visit_variant(&mut self, variant: &'a Variant) {
        walk_variant(self, variant);
    }

    fn visit_proto(&mut self, proto: &'a ProtoTy) {
        walk_proto(self, proto);
    }

    fn visit_parameter(&mut self, param: &'a Parameter) {
        walk_parameter(self, param);
    }

    fn visit_annotation(&mut self, ann: &'a Ann) {
        walk_annotation(self, ann);
    }
}

pub fn walk_tree<'a, V>(visitor: &mut V, tree: &'a [Def])
where
    V: Visitor<'a> + ?Sized,
{
    for def in tree {
        walk_def(visitor, def);
    }
}

pub fn walk_def<'a, V>(visitor: &mut V, def: &'a Def)
where
    V: Visitor<'a> + ?Sized,
{
    // First visit annotations on the definition itself
    for ann in &def.annotations {
        visitor.visit_annotation(ann);
    }

    // Then visit the specific definition kind
    match &def.kind {
        DefKind::Annotation(v) => visitor.visit_annotation_def(def, v),
        DefKind::Module(v) => visitor.visit_module(def, v),
        DefKind::Struct(v) => visitor.visit_struct(def, v),
        DefKind::Except(v) => visitor.visit_except(def, v),
        DefKind::Union(v) => visitor.visit_union(def, v),
        DefKind::Enum(v) => visitor.visit_enum(def, v),
        DefKind::Const(v) => visitor.visit_const(def, v),
        DefKind::Bitmask(v) => visitor.visit_bitmask(def, v),
        DefKind::Bitset(v) => visitor.visit_bitset(def, v),
        DefKind::Alias(v) => visitor.visit_alias(def, v),
        DefKind::Interface(v) => visitor.visit_interface(def, v),
        DefKind::Valuetype(v) => visitor.visit_valuetype(def, v),
        DefKind::Decl(v) => visitor.visit_decl(def, v),
    }
}

pub fn walk_struct<'a, V>(visitor: &mut V, data: &'a StructTy)
where
    V: Visitor<'a> + ?Sized,
{
    for member in &data.members {
        visitor.visit_member(member);
    }
}

pub fn walk_annotation_def<'a, V>(visitor: &mut V, _def: &'a Def, data: &'a AnnotationTy)
where
    V: Visitor<'a> + ?Sized,
{
    for member in &data.members {
        visitor.visit_member(member);
    }
    // Note: We don't visit nested types here as they're already visited via walk_def
}

pub fn walk_module<'a, V>(_visitor: &mut V, _def: &'a Def, _data: &'a ModuleTy)
where
    V: Visitor<'a> + ?Sized,
{
    // Module definitions are visited separately via walk_def, not here
}

pub fn walk_except<'a, V>(visitor: &mut V, data: &'a ExceptTy)
where
    V: Visitor<'a> + ?Sized,
{
    for member in &data.members {
        visitor.visit_member(member);
    }
}

pub fn walk_enum<'a, V>(visitor: &mut V, data: &'a EnumTy)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&data.ty);
    for field in &data.fields {
        // Visit annotations on enum literals
        for ann in &field.annotations {
            visitor.visit_annotation(ann);
        }
    }
}

pub fn walk_union<'a, V>(visitor: &mut V, data: &'a UnionTy)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&data.disc);
    for variant in &data.variants {
        visitor.visit_variant(variant);
    }
}

pub fn walk_alias<'a, V>(visitor: &mut V, data: &'a AliasTy)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&data.ty);
}

pub fn walk_bitmask<'a, V>(visitor: &mut V, data: &'a BitmaskTy)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&data.ty);
    for flag in &data.flags {
        // Visit annotations on bit flags
        for ann in &flag.annotations {
            visitor.visit_annotation(ann);
        }
    }
}

pub fn walk_bitset<'a, V>(visitor: &mut V, data: &'a BitsetTy)
where
    V: Visitor<'a> + ?Sized,
{
    for field in &data.fields {
        // Visit the field's type and annotations directly
        visitor.visit_ty(&field.ty);
        for ann in &field.annotations {
            visitor.visit_annotation(ann);
        }
    }
}

pub fn walk_const<'a, V>(visitor: &mut V, data: &'a ConstTy)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&data.ty);
    visitor.visit_numeric(&data.value);
}

pub fn walk_interface<'a, V>(visitor: &mut V, _def: &'a Def, data: &'a InterfaceTy)
where
    V: Visitor<'a> + ?Sized,
{
    for proto in &data.prototypes {
        visitor.visit_proto(proto);
    }
    // Note: definitions and attributes would be visited separately
}

pub fn walk_valuetype<'a, V>(visitor: &mut V, _def: &'a Def, data: &'a ValueTy)
where
    V: Visitor<'a> + ?Sized,
{
    for proto in &data.prototypes {
        visitor.visit_proto(proto);
    }
    // Note: members and definitions would be visited separately when implemented
}

pub fn walk_decl<'a, V>(_visitor: &mut V, _data: &'a Decl)
where
    V: Visitor<'a> + ?Sized,
{
    // Forward declarations have no content to visit
}

pub fn walk_ty<'a, V>(visitor: &mut V, ty: &'a Ty)
where
    V: Visitor<'a> + ?Sized,
{
    match &ty.kind {
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => visitor.visit_ty(ty),
        TyKind::Map { key, elem, .. } => {
            visitor.visit_ty(key);
            visitor.visit_ty(elem);
        }
        TyKind::String { .. }
        | TyKind::Primitive(_)
        | TyKind::Any
        | TyKind::Fixed
        | TyKind::Adt(_) => {
            // No nested types to visit
        }
    }
}

pub fn walk_numeric<'a, V>(visitor: &mut V, num: &'a Numeric)
where
    V: Visitor<'a> + ?Sized,
{
    match num {
        Numeric::Array { values, .. } | Numeric::Sequence { values, .. } => {
            for value in values {
                visitor.visit_numeric(value);
            }
        }
        Numeric::Map { values, .. } => {
            for (key, value) in values {
                visitor.visit_numeric(key);
                visitor.visit_numeric(value);
            }
        }
        Numeric::Struct { fields, .. } => {
            for (_, value) in fields {
                visitor.visit_numeric(value);
            }
        }
        Numeric::Union {
            discriminant,
            value,
            ..
        } => {
            visitor.visit_numeric(discriminant);
            visitor.visit_numeric(value);
        }
        _ => {
            // Primitive numeric values have nothing to visit
        }
    }
}

pub fn walk_member<'a, V>(visitor: &mut V, member: &'a Member)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&member.ty);
    for ann in &member.annotations {
        visitor.visit_annotation(ann);
    }
}

pub fn walk_variant<'a, V>(visitor: &mut V, variant: &'a Variant)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &variant.annotations {
        visitor.visit_annotation(ann);
    }
    visitor.visit_ty(&variant.ty);
    for label in &variant.labels {
        visitor.visit_numeric(label);
    }
}

pub fn walk_proto<'a, V>(visitor: &mut V, proto: &'a ProtoTy)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&proto.ty);
    for param in &proto.params {
        visitor.visit_parameter(param);
    }
}

pub fn walk_parameter<'a, V>(visitor: &mut V, param: &'a Parameter)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ty(&param.ty);
}

pub fn walk_annotation<'a, V>(visitor: &mut V, ann: &'a Ann)
where
    V: Visitor<'a> + ?Sized,
{
    // Ann no longer has a ty field
    for arg in &ann.args {
        // Visit the argument's value directly
        visitor.visit_numeric(&arg.value);
    }
}
