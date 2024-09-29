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

#![allow(unused)]

use crate::hir::{
    AliasTy, AnnotationTy, BitmaskTy, ConstTy, Decl, Def, DefKind, EnumTy, ExceptTy, InterfaceTy,
    ModuleTy, Numeric, StructTy, Ty, UnionTy, ValueTy,
};

pub trait Visitor<'a> {
    fn visit_def(&mut self, def: &'a Def) {
        walk_def(self, def);
    }

    fn visit_annotation(&mut self, def: &'a Def, data: &'a AnnotationTy) {}

    fn visit_module(&mut self, def: &'a Def, data: &'a ModuleTy) {}

    fn visit_struct(&mut self, def: &'a Def, data: &'a StructTy) {}

    fn visit_except(&mut self, def: &'a Def, data: &'a ExceptTy) {}

    fn visit_enum(&mut self, def: &'a Def, data: &'a EnumTy) {}

    fn visit_union(&mut self, def: &'a Def, data: &'a UnionTy) {}

    fn visit_alias(&mut self, def: &'a Def, data: &'a AliasTy) {}

    fn visit_bitmask(&mut self, def: &'a Def, data: &'a BitmaskTy) {}

    fn visit_const(&mut self, def: &'a Def, data: &'a ConstTy) {}

    fn visit_interface(&mut self, def: &'a Def, data: &'a InterfaceTy) {}

    fn visit_valuetype(&mut self, def: &'a Def, data: &'a ValueTy) {}

    fn visit_decl(&mut self, def: &'a Def, data: &'a Decl) {}

    fn visit_ty(&mut self, ty: &'a Ty) {}

    fn visit_numeric(&mut self, num: &'a Numeric) {}
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
    match &def.kind {
        DefKind::Annotation(v) => visitor.visit_annotation(def, v),
        DefKind::Module(v) => visitor.visit_module(def, v),
        DefKind::Struct(v) => visitor.visit_struct(def, v),
        DefKind::Except(v) => visitor.visit_except(def, v),
        DefKind::Union(v) => visitor.visit_union(def, v),
        DefKind::Enum(v) => visitor.visit_enum(def, v),
        DefKind::Const(v) => visitor.visit_const(def, v),
        DefKind::Bitmask(v) => visitor.visit_bitmask(def, v),
        DefKind::Alias(v) => visitor.visit_alias(def, v),
        DefKind::Interface(v) => visitor.visit_interface(def, v),
        DefKind::Valuetype(v) => visitor.visit_valuetype(def, v),
        DefKind::Decl(v) => visitor.visit_decl(def, v),
    }
}
