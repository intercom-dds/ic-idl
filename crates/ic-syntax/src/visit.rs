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
    AliasDef, AnnotationAppl, AnnotationArg, AnnotationDef, AnnotationField, Binary, Bit, Bitfield,
    BitmaskDef, BitsetDef, ConstDef, Decl, Discriminator, EnumDef, Enumerator, ExceptDef, Expr,
    Field, Ident, InitList, InterfaceDef, InterfaceMember, Item, ItemKind, Label, Literal,
    ModuleDef, Param, Prototype, Span, StructDef, Type, Unary, UnionDef, UnionElement, UnionField,
    UnionMember, UnionNull, ValuetypeDef,
};

pub trait Visitor<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        visit_item(self, item);
    }

    fn visit_annotation_def(&mut self, def: &'a AnnotationDef) {}

    fn visit_annotation_field(&mut self, def: &'a AnnotationField) {}

    fn visit_annotation_appl(&mut self, def: &'a AnnotationAppl) {}

    fn visit_annotation_arg(&mut self, def: &'a AnnotationArg) {}

    fn visit_module(&mut self, module: &'a ModuleDef) {
        visit_module(self, module);
    }

    fn visit_struct(&mut self, def: &'a StructDef) {
        visit_struct(self, def);
    }

    fn visit_struct_field(&mut self, def: &'a Field) {
        visit_struct_field(self, def);
    }

    fn visit_union(&mut self, def: &'a UnionDef) {
        visit_union(self, def);
    }

    fn visit_discriminant(&mut self, def: &'a Discriminator) {
        visit_discriminant(self, def);
    }

    fn visit_union_variant(&mut self, variant: &'a UnionField) {
        visit_union_variant(self, variant);
    }

    fn visit_union_label(&mut self, def: &'a Label) {
        visit_union_label(self, def);
    }

    fn visit_union_member(&mut self, def: &'a UnionMember) {}

    fn visit_union_null(&mut self, def: &'a UnionNull) {}

    fn visit_enum(&mut self, def: &'a EnumDef) {
        visit_enum(self, def);
    }

    fn visit_enum_variant(&mut self, enumerator: &'a Enumerator) {
        visit_enum_variant(self, enumerator);
    }

    fn visit_exception(&mut self, exception: &'a ExceptDef) {
        visit_exception(self, exception);
    }

    fn visit_interface(&mut self, interface: &'a InterfaceDef) {
        visit_interface(self, interface);
    }

    fn visit_valuetype(&mut self, def: &'a ValuetypeDef) {
        visit_valuetype(self, def);
    }

    fn visit_prototype(&mut self, def: &'a Prototype) {
        visit_prototype(self, def);
    }

    fn visit_prototype_param(&mut self, param: &'a Param) {
        visit_param(self, param);
    }

    fn visit_bitmask(&mut self, bitmask: &'a BitmaskDef) {
        visit_bitmask(self, bitmask);
    }

    fn visit_bitmask_bit(&mut self, bit: &'a Bit) {
        visit_bitmask_bit(self, bit);
    }

    fn visit_bitset(&mut self, bitset: &'a BitsetDef) {
        visit_bitset(self, bitset);
    }

    fn visit_bitfield(&mut self, bitfield: &'a Bitfield) {}

    fn visit_const(&mut self, def: &'a ConstDef) {
        visit_const(self, def);
    }

    fn visit_typedef(&mut self, def: &'a AliasDef) {
        visit_typedef(self, def);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Unary(v) => self.visit_expr_unary(v),
            Expr::Binary(v) => self.visit_expr_binary(v),
            _ => (),
        }
    }

    fn visit_expr_unary(&mut self, binary: &'a Unary) {}

    fn visit_expr_binary(&mut self, binary: &'a Binary) {}

    fn visit_expr_init_list(&mut self, binary: &'a InitList) {}

    fn visit_decl(&mut self, decl: &'a Decl) {}

    fn visit_ident(&mut self, ident: &'a Ident) {}

    fn visit_type(&mut self, ident: &'a Type) {}

    fn visit_literal(&mut self, num: &'a Literal) {}
}

pub fn visit_tree<'a, V>(visitor: &mut V, tree: &'a [Item])
where
    V: Visitor<'a> + ?Sized,
{
    for item in tree {
        visitor.visit_item(item);
    }
}

pub fn visit_item<'a, V>(visitor: &mut V, item: &'a Item)
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
        Item::DeclValue(v) => visitor.visit_decl(v),
        Item::InterfaceValue(v) => visitor.visit_interface(v),
        Item::ValuetypeValue(v) => visitor.visit_valuetype(v),
    }
}

pub fn visit_module<'a, V>(visitor: &mut V, module: &'a ModuleDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&module.ident);
    for def in &module.definitions {
        visitor.visit_item(def);
    }
}

pub fn visit_struct<'a, V>(visitor: &mut V, def: &'a StructDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for mem in &def.members {
        visitor.visit_struct_field(mem);
    }
}

pub fn visit_struct_field<'a, V>(visitor: &mut V, def: &'a Field)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
    for _decl in &def.names {
        // visitor.visit_decl(decl.clon);
    }
}

pub fn visit_union<'a, V>(visitor: &mut V, def: &'a UnionDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    visitor.visit_discriminant(&def.disc);
    for mem in &def.fields {
        visitor.visit_union_variant(mem);
    }
}

pub fn visit_discriminant<'a, V>(visitor: &mut V, def: &'a Discriminator)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
}

pub fn visit_union_variant<'a, V>(visitor: &mut V, def: &'a UnionField)
where
    V: Visitor<'a> + ?Sized,
{
    for label in &def.labels {
        visitor.visit_union_label(label);
    }
    match &def.field {
        UnionElement::Member(v) => visitor.visit_union_member(v),
        UnionElement::Null(v) => visitor.visit_union_null(v),
    }
}

pub fn visit_union_label<'a, V>(visitor: &mut V, def: &'a Label)
where
    V: Visitor<'a> + ?Sized,
{
    if let Label::Case(v) = def {
        visitor.visit_expr(v);
    }
}

pub fn visit_enum<'a, V>(visitor: &mut V, def: &'a EnumDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for var in &def.fields {
        visitor.visit_enum_variant(var);
    }
}

pub fn visit_enum_variant<'a, V>(visitor: &mut V, def: &'a Enumerator)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    if let Some(expr) = &def.value {
        visitor.visit_expr(expr);
    }
}

pub fn visit_exception<'a, V>(visitor: &mut V, def: &'a ExceptDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for member in &def.members {
        visitor.visit_struct_field(member);
    }
}

pub fn visit_interface<'a, V>(visitor: &mut V, def: &'a InterfaceDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    // TODO: inherit? attributes?
    for proto in &def.members {
        match proto {
            InterfaceMember::Attr(_) => todo!(),
            InterfaceMember::Proto(v) => visitor.visit_prototype(v),
            InterfaceMember::Item(v) => visitor.visit_item(v),
        }
    }
}

pub fn visit_valuetype<'a, V>(visitor: &mut V, def: &'a ValuetypeDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);

    for proto in &def.prototypes {
        visitor.visit_prototype(proto);
    }

    // TODO:
    // for proto in &def.members {
    //     visitor.visit_valuetype_member(&proto);
    // }
}

pub fn visit_prototype<'a, V>(visitor: &mut V, def: &'a Prototype)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for param in &def.params {
        visitor.visit_prototype_param(param);
    }
}

pub fn visit_param<'a, V>(visitor: &mut V, def: &'a Param)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
    visitor.visit_ident(&def.ident);
}

pub fn visit_bitmask<'a, V>(visitor: &mut V, def: &'a BitmaskDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for bit in &def.bits {
        visitor.visit_bitmask_bit(bit);
    }
}

pub fn visit_bitmask_bit<'a, V>(visitor: &mut V, def: &'a Bit)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    if let Some(expr) = &def.value {
        visitor.visit_expr(expr);
    }
}

pub fn visit_bitset<'a, V>(visitor: &mut V, def: &'a BitsetDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_ident(&def.ident);
    for bitfield in &def.fields {
        visitor.visit_bitfield(bitfield);
    }
}

pub fn visit_const<'a, V>(visitor: &mut V, def: &'a ConstDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
    visitor.visit_ident(&def.ident);
    visitor.visit_expr(&def.value);
}

pub fn visit_typedef<'a, V>(visitor: &mut V, def: &'a AliasDef)
where
    V: Visitor<'a> + ?Sized,
{
    visitor.visit_type(&def.ty);
    // TODO:
    // for decl in &def.decl {
    //     visitor.visit_decl(decl);
    // }
}

pub trait Visit {
    fn visit<'a, V>(&'a self, visitor: &mut V)
    where
        V: Visitor<'a> + ?Sized;
}

impl<T: Visit> Visit for Option<T> {
    fn visit<'a, V>(&'a self, visitor: &mut V)
    where
        V: Visitor<'a> + ?Sized,
    {
        if let Some(v) = self {
            v.visit(visitor);
        }
    }
}

impl Visit for Item {
    fn visit<'a, V>(&'a self, visitor: &mut V)
    where
        V: Visitor<'a> + ?Sized,
    {
        match &self {
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
            Item::DeclValue(v) => visitor.visit_decl(v),
            Item::InterfaceValue(v) => visitor.visit_interface(v),
            Item::ValuetypeValue(v) => visitor.visit_valuetype(v),
        }
    }
}

impl Visit for ModuleDef {
    fn visit<'a, V>(&'a self, visitor: &mut V)
    where
        V: Visitor<'a> + ?Sized,
    {
        visitor.visit_ident(&self.ident);
        for item in &self.definitions {
            visitor.visit_item(item);
        }
    }
}
