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

use crate::{
    AliasDef, Annotation, AnnotationArg, AnnotationDef, AnnotationMember, Attribute, BinaryExpr,
    Bit, Bitfield, BitmaskDef, BitsetDef, ConstDef, Decl, Declarator, Disc, EnumDef, Enumerator,
    ExceptDef, Expr, ExprKind, Field, Ident, InterfaceDef, InterfaceMember, Item, Label, Literal,
    Meta, ModuleDef, NamedExpr, Param, Path, Proto, StructDef, Type, UnaryExpr, UnionCase,
    UnionDef, ValueMember, ValuetypeDef,
};

pub trait Visitor<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        walk_item(self, item);
    }

    fn visit_annotation_def(&mut self, value: &'a AnnotationDef) {
        walk_annotation_def(self, value);
    }

    fn visit_annotation_field(&mut self, value: &'a AnnotationMember) {
        walk_annotation_field(self, value);
    }

    fn visit_annotation_appl(&mut self, value: &'a Annotation) {
        walk_annotation_appl(self, value);
    }

    fn visit_annotation_arg(&mut self, value: &'a AnnotationArg) {
        walk_annotation_arg(self, value);
    }

    fn visit_module(&mut self, value: &'a ModuleDef) {
        walk_module(self, value);
    }

    fn visit_struct(&mut self, value: &'a StructDef) {
        walk_struct(self, value);
    }

    fn visit_struct_field(&mut self, value: &'a Field) {
        walk_struct_field(self, value);
    }

    fn visit_union(&mut self, value: &'a UnionDef) {
        walk_union(self, value);
    }

    fn visit_discriminant(&mut self, value: &'a Disc) {
        walk_discriminant(self, value);
    }

    fn visit_union_variant(&mut self, value: &'a UnionCase) {
        walk_union_variant(self, value);
    }

    fn visit_union_label(&mut self, value: &'a Label) {
        walk_union_label(self, value);
    }

    fn visit_enum(&mut self, value: &'a EnumDef) {
        walk_enum(self, value);
    }

    fn visit_enum_variant(&mut self, value: &'a Enumerator) {
        walk_enum_variant(self, value);
    }

    fn visit_exception(&mut self, value: &'a ExceptDef) {
        walk_exception(self, value);
    }

    fn visit_interface(&mut self, value: &'a InterfaceDef) {
        walk_interface(self, value);
    }

    fn visit_valuetype(&mut self, value: &'a ValuetypeDef) {
        walk_valuetype(self, value);
    }

    fn visit_attribute(&mut self, value: &'a Attribute) {
        walk_attribute(self, value);
    }

    fn visit_prototype(&mut self, value: &'a Proto) {
        walk_prototype(self, value);
    }

    fn visit_prototype_param(&mut self, value: &'a Param) {
        walk_param(self, value);
    }

    fn visit_bitmask(&mut self, value: &'a BitmaskDef) {
        walk_bitmask(self, value);
    }

    fn visit_bitmask_bit(&mut self, value: &'a Bit) {
        walk_bitmask_bit(self, value);
    }

    fn visit_bitset(&mut self, value: &'a BitsetDef) {
        walk_bitset(self, value);
    }

    fn visit_bitfield(&mut self, value: &'a Bitfield) {
        walk_bitfield(self, value);
    }

    fn visit_const(&mut self, value: &'a ConstDef) {
        walk_const(self, value);
    }

    fn visit_typedef(&mut self, value: &'a AliasDef) {
        walk_typedef(self, value);
    }

    fn visit_expr(&mut self, value: &'a Expr) {
        walk_expr(self, value);
    }

    fn visit_expr_unary(&mut self, value: &'a UnaryExpr) {
        walk_expr_unary(self, value);
    }

    fn visit_expr_binary(&mut self, value: &'a BinaryExpr) {
        walk_expr_binary(self, value);
    }

    fn visit_expr_init_list(&mut self, value: &'a [NamedExpr]) {
        walk_expr_init_list(self, value);
    }

    fn visit_expr_group(&mut self, value: &'a Expr) {
        walk_expr_group(self, value);
    }

    fn visit_forward_decl(&mut self, value: &'a Decl) {
        walk_decl(self, value);
    }

    fn visit_ident(&mut self, _: &'a Ident) {}

    fn visit_declarator(&mut self, value: &'a Declarator) {
        walk_declarator(self, value);
    }

    fn visit_path(&mut self, value: &'a Path) {
        walk_path(self, value);
    }

    fn visit_type(&mut self, value: &'a Type) {
        walk_type(self, value);
    }

    fn visit_literal(&mut self, _: &'a Literal) {}
}

pub fn walk_tree<'a, V: Visitor<'a> + ?Sized>(visitor: &mut V, tree: &'a [Item]) {
    for item in tree {
        visitor.visit_item(item);
    }
}

pub fn walk_item<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Item) {
    match x {
        Item::Annotation(x) => v.visit_annotation_def(x),
        Item::Module(x) => v.visit_module(x),
        Item::Struct(x) => v.visit_struct(x),
        Item::Union(x) => v.visit_union(x),
        Item::Enum(x) => v.visit_enum(x),
        Item::Exception(x) => v.visit_exception(x),
        Item::Bitmask(x) => v.visit_bitmask(x),
        Item::Bitset(x) => v.visit_bitset(x),
        Item::Const(x) => v.visit_const(x),
        Item::Alias(x) => v.visit_typedef(x),
        Item::Interface(x) => v.visit_interface(x),
        Item::Valuetype(x) => v.visit_valuetype(x),
        Item::Decl(x) => v.visit_forward_decl(x),
    }
}

fn walk_meta<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Meta) {
    for x in &x.annotations {
        v.visit_annotation_appl(x);
    }
}

pub fn walk_annotation_def<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a AnnotationDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    for x in &x.members {
        v.visit_annotation_field(x);
    }
}

pub fn walk_annotation_field<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a AnnotationMember) {
    match x {
        AnnotationMember::Item(x) => v.visit_item(x),
        AnnotationMember::Value(_) => {}
    }
}

pub fn walk_annotation_appl<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Annotation) {
    v.visit_path(&x.path);
    for x in &x.arguments {
        v.visit_annotation_arg(x);
    }
}

pub fn walk_annotation_arg<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a AnnotationArg) {
    if let Some(x) = &x.name {
        v.visit_ident(x);
    }
    v.visit_expr(&x.value);
}

pub fn walk_module<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a ModuleDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    for x in &x.items {
        v.visit_item(x);
    }
}

pub fn walk_struct<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a StructDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    if let Some(x) = &x.parent {
        v.visit_path(x);
    }
    for x in &x.fields {
        v.visit_struct_field(x);
    }
}

pub fn walk_struct_field<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Field) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.ty);
    for x in &x.declarators {
        v.visit_declarator(x);
    }
}

pub fn walk_union<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a UnionDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    v.visit_discriminant(&x.disc);
    for x in &x.cases {
        v.visit_union_variant(x);
    }
}

pub fn walk_discriminant<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Disc) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.ty);
}

pub fn walk_union_variant<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a UnionCase) {
    walk_meta(v, &x.meta);
    for x in &x.labels {
        v.visit_union_label(x);
    }
    v.visit_type(&x.ty);
    v.visit_declarator(&x.declarator);
}

pub fn walk_union_label<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Label) {
    if let Label::Value(x) = x {
        v.visit_expr(x);
    }
}

pub fn walk_enum<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a EnumDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    for x in &x.enumerators {
        v.visit_enum_variant(x);
    }
}

pub fn walk_enum_variant<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Enumerator) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    if let Some(x) = &x.value {
        v.visit_expr(x);
    }
}

pub fn walk_exception<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a ExceptDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    for x in &x.fields {
        v.visit_struct_field(x);
    }
}

pub fn walk_interface<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a InterfaceDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    for x in &x.inherits {
        v.visit_path(x);
    }
    for x in &x.members {
        match x {
            InterfaceMember::Attribute(x) => v.visit_attribute(x),
            InterfaceMember::Proto(x) => v.visit_prototype(x),
            InterfaceMember::Item(x) => v.visit_item(x),
        }
    }
}

pub fn walk_valuetype<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a ValuetypeDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    if let Some(x) = &x.inherits {
        v.visit_path(x);
    }
    for x in &x.supports {
        v.visit_path(x);
    }
    for x in &x.members {
        match x {
            ValueMember::State(x) => {
                walk_meta(v, &x.meta);
                for x in &x.declarators {
                    v.visit_declarator(x);
                }
                v.visit_type(&x.ty);
            }
            ValueMember::Attribute(x) => v.visit_attribute(x),
            ValueMember::Proto(x) => v.visit_prototype(x),
            ValueMember::Item(x) => v.visit_item(x),
        }
    }
}

pub fn walk_attribute<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Attribute) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.ty);
    for x in &x.declarators {
        v.visit_declarator(x);
    }
    for x in x.setraises.iter().chain(&x.getraises) {
        v.visit_path(x);
    }
}

pub fn walk_prototype<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Proto) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.return_type);
    v.visit_ident(&x.name);
    for x in &x.parameters {
        v.visit_prototype_param(x);
    }
    for x in &x.raises {
        v.visit_path(x);
    }
}

pub fn walk_param<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Param) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.ty);
    v.visit_declarator(&x.declarator);
}

pub fn walk_bitmask<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a BitmaskDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    for x in &x.bits {
        v.visit_bitmask_bit(x);
    }
}

pub fn walk_bitmask_bit<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Bit) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    if let Some(x) = &x.value {
        v.visit_expr(x);
    }
}

pub fn walk_bitset<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a BitsetDef) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
    if let Some(x) = &x.parent {
        v.visit_path(x);
    }
    for x in &x.fields {
        v.visit_bitfield(x);
    }
}

pub fn walk_bitfield<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Bitfield) {
    walk_meta(v, &x.meta);
    for x in &x.declarators {
        v.visit_declarator(x);
    }

    v.visit_expr(&x.size);
    if let Some(x) = &x.ty {
        v.visit_type(x);
    }
}

pub fn walk_const<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a ConstDef) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.ty);
    v.visit_declarator(&x.declarator);
    v.visit_expr(&x.value);
}

pub fn walk_typedef<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a AliasDef) {
    walk_meta(v, &x.meta);
    v.visit_type(&x.ty);
    for x in &x.declarators {
        v.visit_declarator(x);
    }
}

pub fn walk_expr<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Expr) {
    match &x.value {
        ExprKind::Literal(x) => v.visit_literal(x),
        ExprKind::Path(x) => v.visit_path(x),
        ExprKind::Unary(x) => v.visit_expr_unary(x),
        ExprKind::Binary(x) => v.visit_expr_binary(x),
        ExprKind::InitList(x) => v.visit_expr_init_list(x),
        ExprKind::Group(x) => v.visit_expr_group(x),
    }
}

pub fn walk_expr_unary<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a UnaryExpr) {
    v.visit_expr(&x.operand);
}

pub fn walk_expr_binary<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a BinaryExpr) {
    v.visit_expr(&x.lhs);
    v.visit_expr(&x.rhs);
}

pub fn walk_expr_init_list<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a [NamedExpr]) {
    for x in x {
        if let Some(name) = &x.name {
            v.visit_ident(name);
        }
        v.visit_expr(&x.value);
    }
}

pub fn walk_expr_group<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Expr) {
    v.visit_expr(x);
}

pub fn walk_path<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Path) {
    for x in &x.segments {
        v.visit_ident(x);
    }
}

pub fn walk_type<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Type) {
    match x {
        Type::Fixed(x) => {
            if let Some(x) = &x.bounds {
                v.visit_expr(&x.total);
                v.visit_expr(&x.fractional);
            }
        }
        Type::Sequence(x) => {
            for x in &x.element_annotations {
                v.visit_annotation_appl(x);
            }
            v.visit_type(&x.element);
            if let Some(x) = &x.bound {
                v.visit_expr(x);
            }
        }
        Type::String(x) => {
            if let Some(x) = &x.bound {
                v.visit_expr(x);
            }
        }
        Type::Map(x) => {
            for x in x.key_annotations.iter().chain(&x.value_annotations) {
                v.visit_annotation_appl(x);
            }
            v.visit_type(&x.key);
            v.visit_type(&x.value);
            if let Some(x) = &x.bound {
                v.visit_expr(x);
            }
        }
        Type::Named(x) => v.visit_path(x),
    }
}

pub fn walk_declarator<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Declarator) {
    match x {
        Declarator::Name(x) => v.visit_ident(x),
        Declarator::Array(x) => {
            v.visit_ident(&x.name);
            for x in &x.bounds {
                v.visit_expr(x);
            }
        }
    }
}

pub fn walk_decl<'a, V: Visitor<'a> + ?Sized>(v: &mut V, x: &'a Decl) {
    walk_meta(v, &x.meta);
    v.visit_ident(&x.name);
}
