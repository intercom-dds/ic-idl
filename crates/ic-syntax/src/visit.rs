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

#![allow(unused, dead_code)]

use crate::{
    AliasDef, AnnotationAppl, AnnotationArg, AnnotationDef, AnnotationField, Attribute, Binary,
    Bit, Bitfield, BitmaskDef, BitsetDef, ConstDef, Decl, Declarator, Discriminator, EnumDef,
    Enumerator, ExceptDef, Expr, Field, Group, Ident, InitList, InterfaceDef, InterfaceMember,
    Item, ItemKind, Label, Literal, ModuleDef, NamedExpr, Param, Path, Prototype, Span, StructDef,
    Type, Unary, UnionDef, UnionElement, UnionField, UnionMember, UnionNull, ValueElement,
    ValuetypeDef,
};

pub trait Visitor<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        walk_item(self, item);
    }

    fn visit_annotation_def(&mut self, def: &'a AnnotationDef) {
        walk_annotation_def(self, def);
    }

    fn visit_annotation_field(&mut self, def: &'a AnnotationField) {
        walk_annotation_field(self, def);
    }

    fn visit_annotation_appl(&mut self, def: &'a AnnotationAppl) {
        walk_annotation_appl(self, def);
    }

    fn visit_annotation_arg(&mut self, def: &'a AnnotationArg) {
        walk_annotation_arg(self, def);
    }

    fn visit_module(&mut self, module: &'a ModuleDef) {
        walk_module(self, module);
    }

    fn visit_struct(&mut self, def: &'a StructDef) {
        walk_struct(self, def);
    }

    fn visit_struct_field(&mut self, def: &'a Field) {
        walk_struct_field(self, def);
    }

    fn visit_union(&mut self, def: &'a UnionDef) {
        walk_union(self, def);
    }

    fn visit_discriminant(&mut self, def: &'a Discriminator) {
        walk_discriminant(self, def);
    }

    fn visit_union_variant(&mut self, variant: &'a UnionField) {
        walk_union_variant(self, variant);
    }

    fn visit_union_label(&mut self, def: &'a Label) {
        walk_union_label(self, def);
    }

    fn visit_union_member(&mut self, def: &'a UnionMember) {}

    fn visit_union_null(&mut self, def: &'a UnionNull) {}

    fn visit_enum(&mut self, def: &'a EnumDef) {
        walk_enum(self, def);
    }

    fn visit_enum_variant(&mut self, enumerator: &'a Enumerator) {
        walk_enum_variant(self, enumerator);
    }

    fn visit_exception(&mut self, exception: &'a ExceptDef) {
        walk_exception(self, exception);
    }

    fn visit_interface(&mut self, interface: &'a InterfaceDef) {
        walk_interface(self, interface);
    }

    fn visit_valuetype(&mut self, def: &'a ValuetypeDef) {
        walk_valuetype(self, def);
    }

    fn visit_attribute(&mut self, def: &'a Attribute) {
        walk_attribute(self, def);
    }

    fn visit_prototype(&mut self, def: &'a Prototype) {
        walk_prototype(self, def);
    }

    fn visit_prototype_param(&mut self, param: &'a Param) {
        walk_param(self, param);
    }

    fn visit_bitmask(&mut self, bitmask: &'a BitmaskDef) {
        walk_bitmask(self, bitmask);
    }

    fn visit_bitmask_bit(&mut self, bit: &'a Bit) {
        walk_bitmask_bit(self, bit);
    }

    fn visit_bitset(&mut self, bitset: &'a BitsetDef) {
        walk_bitset(self, bitset);
    }

    fn visit_bitfield(&mut self, bitfield: &'a Bitfield) {
        walk_bitfield(self, bitfield);
    }

    fn visit_const(&mut self, def: &'a ConstDef) {
        walk_const(self, def);
    }

    fn visit_typedef(&mut self, def: &'a AliasDef) {
        walk_typedef(self, def);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        walk_expr(self, expr);
    }

    fn visit_expr_unary(&mut self, unary: &'a Unary) {
        walk_expr_unary(self, unary);
    }

    fn visit_expr_binary(&mut self, binary: &'a Binary) {
        walk_expr_binary(self, binary);
    }

    fn visit_expr_init_list(&mut self, init_list: &'a InitList) {
        walk_expr_init_list(self, init_list);
    }
    fn visit_expr_group(&mut self, group: &'a Group) {
        walk_expr_group(self, group);
    }

    fn visit_forward_decl(&mut self, decl: &'a Decl) {
        walk_decl(self, decl);
    }

    fn visit_ident(&mut self, ident: &'a Ident) {}

    fn visit_declarator(&mut self, decl: &'a Declarator) {
        walk_declarator(self, decl);
    }

    fn visit_path(&mut self, path: &'a Path) {
        walk_path(self, path);
    }

    fn visit_type(&mut self, ident: &'a Type) {
        walk_type(self, ident);
    }

    fn visit_literal(&mut self, num: &'a Literal) {}
}

pub fn walk_tree<'a, V>(visitor: &mut V, tree: &'a [Item])
where
    V: Visitor<'a> + ?Sized,
{
    for item in tree {
        visitor.visit_item(item);
    }
}

pub fn walk_item<'a, V>(visitor: &mut V, item: &'a Item)
where
    V: Visitor<'a> + ?Sized,
{
    match item {
        Item::AnnotationValue(v) => visitor.visit_annotation_def(v),
        Item::ModuleValue(v) => visitor.visit_module(v),
        Item::StructValue(v) => visitor.visit_struct(v),
        Item::UnionValue(v) => visitor.visit_union(v),
        Item::EnumValue(v) => visitor.visit_enum(v),
        Item::ExceptionValue(v) => visitor.visit_exception(v),
        Item::BitmaskValue(v) => visitor.visit_bitmask(v),
        Item::BitsetValue(v) => visitor.visit_bitset(v),
        Item::ConstValue(v) => visitor.visit_const(v),
        Item::AliasValue(v) => visitor.visit_typedef(v),
        Item::DeclValue(v) => visitor.visit_forward_decl(v),
        Item::InterfaceValue(v) => visitor.visit_interface(v),
        Item::ValuetypeValue(v) => visitor.visit_valuetype(v),
    }
}

pub fn walk_annotation_def<'a, V>(visitor: &mut V, def: &'a AnnotationDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for param in &def.params {
        visitor.visit_annotation_field(param);
    }
}

pub fn walk_annotation_field<'a, V>(visitor: &mut V, def: &'a AnnotationField)
where
    V: Visitor<'a> + ?Sized,
{
    match def {
        AnnotationField::Item(v) => visitor.visit_item(v),
        AnnotationField::Member(_) => (),
    }
}

pub fn walk_annotation_appl<'a, V>(visitor: &mut V, def: &'a AnnotationAppl)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_path(&def.ident);
    for arg in &def.args {
        visitor.visit_annotation_arg(arg);
    }
}

pub fn walk_annotation_arg<'a, V>(visitor: &mut V, def: &'a AnnotationArg)
where
    V: Visitor<'a> + ?Sized,
{
    if let Some(ident) = &def.ident {
        visitor.visit_ident(ident);
    }
    visitor.visit_expr(&def.value);
}

pub fn walk_module<'a, V>(visitor: &mut V, module: &'a ModuleDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &module.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&module.ident);
    for def in &module.definitions {
        visitor.visit_item(def);
    }
}

pub fn walk_struct<'a, V>(visitor: &mut V, def: &'a StructDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    for mem in &def.members {
        visitor.visit_struct_field(mem);
    }
}

pub fn walk_struct_field<'a, V>(visitor: &mut V, def: &'a Field)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_type(&def.ty);
    for decl in &def.names {
        visitor.visit_declarator(decl);
    }
}

pub fn walk_union<'a, V>(visitor: &mut V, def: &'a UnionDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    visitor.visit_discriminant(&def.disc);
    for mem in &def.fields {
        visitor.visit_union_variant(mem);
    }
}

pub fn walk_discriminant<'a, V>(visitor: &mut V, def: &'a Discriminator)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_type(&def.ty);
}

pub fn walk_union_variant<'a, V>(visitor: &mut V, def: &'a UnionField)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    for label in &def.labels {
        visitor.visit_union_label(label);
    }
    match &def.field {
        UnionElement::Member(v) => visitor.visit_union_member(v),
        UnionElement::Null(v) => visitor.visit_union_null(v),
    }
}

pub fn walk_union_label<'a, V>(visitor: &mut V, def: &'a Label)
where
    V: Visitor<'a> + ?Sized,
{
    if let Label::Case(v) = def {
        visitor.visit_expr(v);
    }
}

pub fn walk_enum<'a, V>(visitor: &mut V, def: &'a EnumDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    for var in &def.fields {
        visitor.visit_enum_variant(var);
    }
}

pub fn walk_enum_variant<'a, V>(visitor: &mut V, def: &'a Enumerator)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    if let Some(expr) = &def.value {
        visitor.visit_expr(expr);
    }
}

pub fn walk_exception<'a, V>(visitor: &mut V, def: &'a ExceptDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    for member in &def.members {
        visitor.visit_struct_field(member);
    }
}

pub fn walk_interface<'a, V>(visitor: &mut V, def: &'a InterfaceDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);

    for base in &def.inherits {
        visitor.visit_path(base);
    }
    for proto in &def.members {
        match proto {
            InterfaceMember::Attr(v) => visitor.visit_attribute(v),
            InterfaceMember::Proto(v) => visitor.visit_prototype(v),
            InterfaceMember::Item(v) => visitor.visit_item(v),
        }
    }
}

pub fn walk_valuetype<'a, V>(visitor: &mut V, def: &'a ValuetypeDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);

    for element in &def.elements {
        match element {
            ValueElement::State(member) => {
                // Visit value member fields
                for decl in &member.decl {
                    visitor.visit_declarator(decl);
                }
                visitor.visit_type(&member.ty);
            }
            ValueElement::Attr(attr) => {
                visitor.visit_attribute(attr);
            }
            ValueElement::Proto(proto) => {
                visitor.visit_prototype(proto);
            }
            ValueElement::Item(item) => {
                visitor.visit_item(item);
            }
        }
    }
}

pub fn walk_attribute<'a, V>(visitor: &mut V, def: &'a Attribute)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
    for decl in &def.decl {
        visitor.visit_declarator(decl);
    }
}

pub fn walk_prototype<'a, V>(visitor: &mut V, def: &'a Prototype)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for param in &def.params {
        visitor.visit_prototype_param(param);
    }
}

pub fn walk_param<'a, V>(visitor: &mut V, def: &'a Param)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
    visitor.visit_declarator(&def.decl);
}

pub fn walk_bitmask<'a, V>(visitor: &mut V, def: &'a BitmaskDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    for bit in &def.bits {
        visitor.visit_bitmask_bit(bit);
    }
}

pub fn walk_bitmask_bit<'a, V>(visitor: &mut V, def: &'a Bit)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    if let Some(expr) = &def.value {
        visitor.visit_expr(expr);
    }
}

pub fn walk_bitset<'a, V>(visitor: &mut V, def: &'a BitsetDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    for bitfield in &def.fields {
        visitor.visit_bitfield(bitfield);
    }
}

pub fn walk_bitfield<'a, V>(visitor: &mut V, def: &'a Bitfield)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&def.ident);
    visitor.visit_expr(&def.size);
    if let Some(ty) = &def.ty {
        visitor.visit_type(ty);
    }
}

pub fn walk_const<'a, V>(visitor: &mut V, def: &'a ConstDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_type(&def.ty);
    visitor.visit_declarator(&def.decl);
    visitor.visit_expr(&def.value);
}

pub fn walk_typedef<'a, V>(visitor: &mut V, def: &'a AliasDef)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &def.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_type(&def.ty);
    for decl in &def.decl {
        visitor.visit_declarator(decl);
    }
}

pub fn walk_expr<'a, V>(visitor: &mut V, expr: &'a Expr)
where
    V: Visitor<'a> + ?Sized,
{
    match expr {
        Expr::Unary(v) => visitor.visit_expr_unary(v),
        Expr::Binary(v) => visitor.visit_expr_binary(v),
        Expr::Literal(v) => visitor.visit_literal(v),
        Expr::InitList(v) => visitor.visit_expr_init_list(v),
        Expr::Path(v) => visitor.visit_path(v),
        Expr::Group(v) => visitor.visit_expr_group(v),
    }
}

pub fn walk_expr_unary<'a, V>(visitor: &mut V, unary: &'a Unary)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_expr(&unary.expr);
}

pub fn walk_expr_binary<'a, V>(visitor: &mut V, binary: &'a Binary)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_expr(&binary.lhs);
    visitor.visit_expr(&binary.rhs);
}

pub fn walk_expr_init_list<'a, V>(visitor: &mut V, init_list: &'a InitList)
where
    V: Visitor<'a> + ?Sized,
{
    for NamedExpr { value, .. } in &init_list.values {
        visitor.visit_expr(value);
    }
}

pub fn walk_expr_group<'a, V>(visitor: &mut V, group: &'a Group)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_expr(&group.expr);
}

pub fn walk_path<'a, V>(visitor: &mut V, path: &'a Path)
where
    V: Visitor<'a> + ?Sized,
{
    for p in &path.segments {
        visitor.visit_ident(p);
    }
}

pub fn walk_type<'a, V>(visitor: &mut V, ty: &'a Type)
where
    V: Visitor<'a> + ?Sized,
{
    match ty {
        Type::Fixed(_) => (),
        Type::Sequence(v) => {
            visitor.visit_type(&v.ty);
            if let Some(expr) = &v.bound {
                visitor.visit_expr(expr);
            }
        }
        Type::String(v) => {
            if let Some(expr) = &v.bound {
                visitor.visit_expr(expr);
            }
        }
        Type::Map(v) => {
            visitor.visit_type(&v.key);
            visitor.visit_type(&v.value);
            if let Some(expr) = &v.bound {
                visitor.visit_expr(expr);
            }
        }
        Type::Path(v) => visitor.visit_path(v),
    }
}

pub fn walk_declarator<'a, V>(visitor: &mut V, decl: &'a Declarator)
where
    V: Visitor<'a> + ?Sized,
{
    match decl {
        Declarator::Simple(ident) => visitor.visit_ident(ident),
        Declarator::Array(array) => {
            visitor.visit_ident(&array.ident);
            for bound in &array.bounds {
                visitor.visit_expr(bound);
            }
        }
    }
}

pub fn walk_decl<'a, V>(visitor: &mut V, decl: &'a Decl)
where
    V: Visitor<'a> + ?Sized,
{
    for ann in &decl.annotations {
        visitor.visit_annotation_appl(ann);
    }
    visitor.visit_ident(&decl.ident);
}
