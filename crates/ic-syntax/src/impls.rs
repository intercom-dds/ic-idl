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

use std::ops::Range;

use crate::{
    AliasDef, AnnotationDef, AnnotationField, Bit, Bitfield, BitmaskDef, BitsetDef, ConstDef, Decl,
    DeclKind, Declarator, Discriminator, EnumDef, Enumerator, ExceptDef, Expr, Field, Ident,
    InterfaceDef, InterfaceMember, Item, ModuleDef, Path, Span, StructDef, Type, UnionDef,
    UnionField, ValueMember, ValuetypeDef,
};

impl Item {
    #[must_use]
    pub fn def_module(ident: Ident, defs: Vec<Item>, span: Span) -> Self {
        Self::ModuleValue(ModuleDef {
            ident,
            span,
            annotations: vec![],
            definitions: defs,
        })
    }

    #[must_use]
    pub fn def_struct(ident: Ident, members: Vec<Field>, parent: Option<Path>, span: Span) -> Self {
        Self::StructValue(StructDef {
            ident,
            span,
            members,
            parent,
            annotations: vec![],
        })
    }

    #[must_use]
    pub fn def_exception(ident: Ident, members: Vec<Field>, span: Span) -> Self {
        Self::ExceptionValue(ExceptDef {
            ident,
            span,
            members,
            annotations: vec![],
        })
    }

    #[must_use]
    pub fn def_union(
        ident: Ident,
        disc: Discriminator,
        fields: Vec<UnionField>,
        span: Span,
    ) -> Self {
        Self::UnionValue(UnionDef {
            ident,
            span,
            annotations: vec![],
            disc,
            fields,
        })
    }

    #[must_use]
    pub fn def_enum(ident: Ident, fields: Vec<Enumerator>, span: Span) -> Self {
        Self::EnumValue(EnumDef {
            ident,
            span,
            annotations: vec![],
            fields,
        })
    }

    #[must_use]
    pub fn def_const(decl: Declarator, ty: Type, value: Expr, span: Span) -> Self {
        Self::ConstValue(ConstDef {
            // TODO(idarcar): remove ident
            ident: Ident::default(),
            decl,
            span,
            value,
            ty,
            annotations: vec![],
        })
    }

    #[must_use]
    pub fn def_annotation(ident: Ident, params: Vec<AnnotationField>, span: Span) -> Self {
        Self::AnnotationValue(AnnotationDef {
            ident,
            span,
            annotations: vec![],
            params,
        })
    }

    #[must_use]
    pub fn interface(
        ident: Ident,
        local: Option<Span>,
        inherits: Vec<Path>,
        members: Vec<InterfaceMember>,
        span: Span,
    ) -> Self {
        Self::InterfaceValue(InterfaceDef {
            ident,
            span,
            annotations: vec![],
            members,
            inherits,
            local,
        })
    }

    #[must_use]
    pub fn valuetype(
        ident: Ident,
        members: Vec<ValueMember>,
        inherits: Option<Path>,
        supports: Option<Path>,
        span: Span,
    ) -> Self {
        Self::ValuetypeValue(ValuetypeDef {
            ident,
            span,
            annotations: vec![],
            prototypes: vec![],
            inherits,
            members,
            supports,
        })
    }

    #[must_use]
    pub fn bitmask(ident: Ident, flags: Vec<Bit>, span: Span) -> Self {
        Self::BitmaskValue(BitmaskDef {
            ident,
            span,
            annotations: vec![],
            bits: flags,
        })
    }

    #[must_use]
    pub fn bitset(
        ident: Ident,
        parent: Option<Path>,
        bitfields: Vec<Bitfield>,
        span: Span,
    ) -> Self {
        Self::BitsetValue(BitsetDef {
            ident,
            span,
            annotations: vec![],
            parent,
            fields: bitfields,
        })
    }

    #[must_use]
    pub fn typedef(decl: Vec<Declarator>, ty: Type, span: Span) -> Self {
        Self::AliasValue(AliasDef {
            // TODO(idarcar): remove ident
            ident: Ident::default(),
            decl,
            span,
            annotations: vec![],
            ty,
        })
    }

    #[must_use]
    pub fn decl(ident: Ident, kind: DeclKind, span: Span) -> Self {
        Self::DeclValue(Decl {
            ident,
            span,
            annotations: vec![],
            kind,
        })
    }
}

// This doesn't really belong here, but since we can't implement the trait in
// `ic-parse`, we have to do it here instead.
impl chumsky::Span for Span {
    type Context = u32;
    type Offset = u32;

    fn new(file_id: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Self {
            start: range.start,
            end: range.end,
            file_id,
        }
    }

    fn context(&self) -> Self::Context {
        self.file_id
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

impl From<Span> for Range<usize> {
    fn from(val: Span) -> Self {
        Self {
            start: val.start as usize,
            end: val.end as usize,
        }
    }
}
