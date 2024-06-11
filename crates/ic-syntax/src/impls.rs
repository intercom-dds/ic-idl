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

use crate::{
    AnnotationDef, AnnotationField, Bit, Bitfield, BitmaskDef, BitsetDef, ConstDef, Decl, DeclKind,
    Declarator, Discriminator, EnumDef, Enumerator, ExceptDef, Expr, Field, Ident, InterfaceDef,
    Item, ModuleDef, Path, Prototype, Span, StructDef, Type, Typedef, UnionDef, UnionField,
    ValueMember, ValuetypeDef,
};

pub type Range = std::ops::Range<usize>;

impl From<Range> for Span {
    fn from(value: Range) -> Self {
        Self {
            start: value.start as u32,
            end: value.end as u32,
        }
    }
}

impl Item {
    pub fn def_module(name: Ident, defs: Vec<Item>, span: Range) -> Self {
        Self::ModuleValue(ModuleDef {
            name,
            span: span.into(),
            annotations: vec![],
            definitions: defs,
        })
    }

    pub fn def_struct(name: Ident, members: Vec<Field>, parent: Option<Path>, span: Range) -> Self {
        Self::StructValue(StructDef {
            name,
            span: span.into(),
            members,
            parent,
            annotations: vec![],
        })
    }

    pub fn def_exception(name: Ident, members: Vec<Field>, span: Range) -> Self {
        Self::ExceptionValue(ExceptDef {
            name,
            span: span.into(),
            members,
            annotations: vec![],
        })
    }

    pub fn def_union(
        name: Ident,
        disc: Discriminator,
        fields: Vec<UnionField>,
        span: Range,
    ) -> Self {
        Self::UnionValue(UnionDef {
            name,
            span: span.into(),
            annotations: vec![],
            disc,
            fields,
        })
    }

    pub fn def_enum(name: Ident, fields: Vec<Enumerator>, span: Range) -> Self {
        Self::EnumValue(EnumDef {
            name,
            span: span.into(),
            annotations: vec![],
            fields,
        })
    }

    pub fn def_const(name: Ident, ty: Type, value: Expr, span: Range) -> Self {
        Self::ConstValue(ConstDef {
            name,
            span: span.into(),
            value,
            ty,
            annotations: vec![],
        })
    }

    pub fn def_annotation(name: Ident, params: Vec<AnnotationField>, span: Range) -> Self {
        Self::AnnotationValue(AnnotationDef {
            name,
            span: span.into(),
            annotations: vec![],
            params,
        })
    }

    pub fn interface(
        name: Ident,
        local: Option<Span>,
        inherits: Vec<Path>,
        prototypes: Vec<Prototype>,
        span: Range,
    ) -> Self {
        Self::InterfaceValue(InterfaceDef {
            name,
            span: span.into(),
            annotations: vec![],
            prototypes,
            inherits,
            local,
        })
    }

    pub fn valuetype(
        name: Ident,
        members: Vec<ValueMember>,
        inherits: Option<Path>,
        supports: Option<Path>,
        span: Range,
    ) -> Self {
        Self::ValuetypeValue(ValuetypeDef {
            name,
            span: span.into(),
            annotations: vec![],
            prototypes: vec![],
            inherits,
            members,
            supports,
        })
    }

    pub fn bitmask(name: Ident, flags: Vec<Bit>, span: Range) -> Self {
        Self::BitmaskValue(BitmaskDef {
            name,
            span: span.into(),
            annotations: vec![],
            bits: flags,
        })
    }

    pub fn bitset(
        name: Ident,
        parent: Option<Path>,
        bitfields: Vec<Bitfield>,
        span: Range,
    ) -> Self {
        Self::BitsetValue(BitsetDef {
            name,
            span: span.into(),
            annotations: vec![],
            parent: todo!(),
            fields: bitfields,
        })
    }

    pub fn typedef(name: Ident, ty: Type, span: Range) -> Self {
        Self::TypedefValue(Typedef {
            name,
            span: span.into(),
            annotations: vec![],
            decl: vec![],
            ty,
        })
    }

    pub fn decl(name: Declarator, kind: DeclKind, span: Range) -> Self {
        Self::DeclValue(Decl {
            name: match name {
                Declarator::Simple(v) => v,
                Declarator::Array(v) => v.ident,
            },
            span: span.into(),
            annotations: vec![],
            kind,
        })
    }
}
