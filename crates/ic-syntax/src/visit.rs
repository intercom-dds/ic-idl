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
    AnnotationAppl, AnnotationArg, AnnotationDef, AnnotationField, Bit, Bitfield, BitmaskDef,
    BitsetDef, ConstDef, Decl, Discriminator, EnumDef, Enumerator, ExceptDef, Expr, Field, Ident,
    InterfaceDef, Item, ItemKind, Label, Literal, ModuleDef, Prototype, StructDef, Type, Typedef,
    UnionDef, UnionField, ValuetypeDef,
};

pub trait Visitor<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        match &item {
            Item::AnnotationValue(v) => self.visit_annotation_def(v),
            Item::ModuleValue(v) => self.visit_module(v),
            Item::StructValue(v) => self.visit_struct(v),
            Item::UnionValue(v) => self.visit_union(v),
            Item::EnumValue(v) => self.visit_enum(v),
            Item::ExceptionValue(v) => self.visit_exception(v),
            Item::BitmaskValue(v) => self.visit_bitmask(v),
            Item::BitsetValue(v) => self.visit_bitset(v),
            Item::ConstValue(v) => self.visit_const(v),
            Item::TypedefValue(v) => self.visit_typedef(v),
            Item::DeclValue(v) => self.visit_decl(v),
            Item::InterfaceValue(v) => self.visit_interface(v),
            Item::ValuetypeValue(v) => self.visit_valuetype(v),
        }
    }

    fn visit_annotation_def(&mut self, def: &'a AnnotationDef) {}

    fn visit_annotation_field(&mut self, def: &'a AnnotationField) {}

    fn visit_annotation_appl(&mut self, def: &'a AnnotationAppl) {}

    fn visit_annotation_arg(&mut self, def: &'a AnnotationArg) {}

    fn visit_module(&mut self, module: &'a ModuleDef) {
        for def in &module.definitions {
            self.visit_item(def);
        }
    }

    fn visit_struct(&mut self, def: &'a StructDef) {
        for mem in &def.members {
            self.visit_struct_field(mem);
        }
    }

    fn visit_struct_field(&mut self, def: &'a Field) {
        self.visit_type(&def.ty);
        for name in &def.names {
            // self.visit_ident(name);
        }
    }

    fn visit_union(&mut self, def: &'a UnionDef) {
        self.visit_discriminant(&def.disc);
        for var in &def.fields {
            self.visit_union_variant(var);
        }
    }

    fn visit_discriminant(&mut self, def: &'a Discriminator) {}

    fn visit_union_variant(&mut self, def: &'a UnionField) {
        for label in &def.labels {
            self.visit_union_label(label);
        }
    }

    fn visit_union_label(&mut self, def: &'a Label) {}

    fn visit_enum(&mut self, def: &'a EnumDef) {
        for var in &def.fields {
            self.visit_enum_variant(var);
        }
    }

    fn visit_enum_variant(&mut self, def: &'a Enumerator) {
        for ann in &def.annotations {
            self.visit_annotation_appl(ann);
        }
        self.visit_ident(&def.name);
    }

    fn visit_exception(&mut self, def: &'a ExceptDef) {}

    fn visit_interface(&mut self, def: &'a InterfaceDef) {
        for proto in &def.prototypes {
            self.visit_prototype(proto);
        }
    }

    fn visit_valuetype(&mut self, def: &'a ValuetypeDef) {
        for proto in &def.prototypes {
            self.visit_prototype(proto);
        }
    }

    fn visit_prototype(&mut self, def: &'a Prototype) {}

    fn visit_literal(&mut self, num: &'a Literal) {}

    fn visit_bitmask(&mut self, bitmask: &'a BitmaskDef) {
        for ann in &bitmask.annotations {
            self.visit_annotation_appl(ann);
        }
        for bit in &bitmask.bits {
            self.visit_bitmask_bit(bit);
        }
    }

    fn visit_bitmask_bit(&mut self, bit: &'a Bit) {}

    fn visit_bitset(&mut self, bitset: &'a BitsetDef) {
        for bit in &bitset.fields {
            self.visit_bitfield(bit);
        }
    }

    fn visit_bitfield(&mut self, _bitset: &'a Bitfield) {}

    fn visit_const(&mut self, def: &'a ConstDef) {}

    fn visit_typedef(&mut self, def: &'a Typedef) {}

    fn visit_decl(&mut self, decl: &'a Decl) {}

    fn visit_ident(&mut self, ident: &'a Ident) {}

    fn visit_type(&mut self, ident: &'a Type) {}

    fn visit_expr(&mut self, expr: &'a Expr) {}
}

pub trait Visit {
    fn visit<'a, V: Visitor<'a>>(self, visitor: &mut V);
}

impl<T: Visit> Visit for Option<T> {
    fn visit<'a, V: Visitor<'a>>(self, visitor: &mut V) {
        if let Some(v) = self {
            v.visit(visitor);
        }
    }
}
