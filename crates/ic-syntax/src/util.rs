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

use ic_vfs::Location;

use crate::ast::Item;
use crate::{Declarator, Expr, OpKind, Path, Span, Type};

#[must_use]
pub fn path_name(path: &Path) -> String {
    let segments = path
        .segments
        .iter()
        .map(|v| v.name.as_str())
        .collect::<Vec<_>>()
        .join("::");

    if path.leading_colons.is_some() {
        format!("::{segments}")
    } else {
        segments
    }
}

#[must_use]
pub fn type_name(path: &Type) -> String {
    match path {
        Type::String(..) => "string".to_string(),
        Type::Map(..) => "map".to_string(),
        Type::Fixed(..) => "fixed".to_string(),
        Type::Sequence(seq) => format!("sequence<{}>", type_name(seq.ty.as_ref())),
        Type::Path(ty) => path_name(ty),
    }
}

#[must_use]
pub fn element_type(path: &Type) -> String {
    match path {
        Type::Map(v) => element_type(v.value.as_ref()),
        Type::Sequence(seq) => element_type(seq.ty.as_ref()),
        _ => type_name(path),
    }
}

#[must_use]
pub fn path_span(path: &Path) -> Span {
    let start = path.leading_colons.map_or_else(
        || {
            path.segments
                .first()
                .map_or_else(Location::default, |v| v.span.start)
        },
        |v| v.start,
    );

    let end = path
        .segments
        .last()
        .map_or_else(Location::default, |v| v.span.end);

    Span { start, end }
}

#[must_use]
pub fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Literal(v) => v.span,
        Expr::Path(v) => path_span(v),
        Expr::Unary(v) => {
            let start = v.op.span.start;
            let end = expr_span(&v.expr).end;
            Span { start, end }
        }
        Expr::Binary(v) => {
            let start = expr_span(&v.lhs).start;
            let end = expr_span(&v.rhs).end;
            Span { start, end }
        }
        Expr::InitList(v) => v.span,
        Expr::Group(v) => v.span,
    }
}

#[must_use]
pub fn ty_span(ty: &Type) -> Span {
    match ty {
        Type::Sequence(v) => v.span,
        Type::String(v) => v.span,
        Type::Map(v) => v.span,
        Type::Fixed(v) => v.span,
        Type::Path(v) => path_span(v),
    }
}

#[must_use]
pub fn decl_name(decl: &Declarator) -> &str {
    match decl {
        Declarator::Simple(v) => &v.name,
        Declarator::Array(v) => &v.ident.name,
    }
}

impl Expr {
    #[must_use]
    pub fn span(&self) -> Span {
        expr_span(self)
    }
}

pub trait ItemTraits {
    fn item_name() -> &'static str;
}

macro_rules! named_item {
    ($($type:ty: $name:expr $(,)?)+) => {
        $(
            impl ItemTraits for $type {
                fn item_name() -> &'static str {
                    $name
                }
            }
        )*
    };
}

named_item! {
    crate::AnnotationDef: "annotation",
    crate::ModuleDef: "module",
    crate::StructDef: "struct",
    crate::UnionDef: "union",
    crate::EnumDef: "enum",
    crate::ValuetypeDef: "valuetype",
    crate::ExceptDef: "exception",
    crate::BitmaskDef: "bitmask",
    crate::BitsetDef: "bitset",
    crate::AliasDef: "typedef",
    crate::ConstDef: "const",
}

#[must_use]
pub fn item_name<T: ItemTraits>(_: &T) -> &'static str {
    T::item_name()
}

#[must_use]
pub fn item_span(item: &Item) -> Span {
    match item {
        Item::AnnotationValue(v) => v.span,
        Item::ModuleValue(v) => v.span,
        Item::StructValue(v) => v.span,
        Item::UnionValue(v) => v.span,
        Item::EnumValue(v) => v.span,
        Item::ExceptionValue(v) => v.span,
        Item::BitmaskValue(v) => v.span,
        Item::BitsetValue(v) => v.span,
        Item::ConstValue(v) => v.span,
        Item::AliasValue(v) => v.span,
        Item::InterfaceValue(v) => v.span,
        Item::ValuetypeValue(v) => v.span,
        Item::DeclValue(v) => v.span,
    }
}

#[must_use]
pub fn item_ident_span(item: &Item) -> Span {
    match item {
        Item::AnnotationValue(v) => v.ident.span,
        Item::ModuleValue(v) => v.ident.span,
        Item::StructValue(v) => v.ident.span,
        Item::UnionValue(v) => v.ident.span,
        Item::EnumValue(v) => v.ident.span,
        Item::ExceptionValue(v) => v.ident.span,
        Item::BitmaskValue(v) => v.ident.span,
        Item::BitsetValue(v) => v.ident.span,
        Item::ConstValue(v) => decl_span(&v.decl),
        Item::InterfaceValue(v) => v.ident.span,
        Item::ValuetypeValue(v) => v.ident.span,
        Item::DeclValue(v) => v.ident.span,
        Item::AliasValue(v) => {
            if let (Some(first), Some(last)) =
                (v.decl.first().map(decl_span), v.decl.last().map(decl_span))
            {
                Span {
                    start: first.start,
                    end: last.end,
                }
            } else {
                // Fall back to using the type span. This should never happen
                // for well-constructed ASTs, but we need to return something
                // and this avoids panicking.
                ty_span(&v.ty)
            }
        }
    }
}

#[must_use]
pub fn decl_span(decl: &Declarator) -> Span {
    match decl {
        Declarator::Simple(v) => v.span,
        Declarator::Array(v) => v.ident.span,
    }
}

#[must_use]
pub fn item_variant_name(item: &Item) -> &'static str {
    match item {
        Item::AnnotationValue(_) => "annotation",
        Item::ModuleValue(_) => "module",
        Item::StructValue(_) => "struct",
        Item::UnionValue(_) => "union",
        Item::EnumValue(_) => "enum",
        Item::ExceptionValue(_) => "exception",
        Item::BitmaskValue(_) => "bitmask",
        Item::BitsetValue(_) => "bitset",
        Item::ConstValue(_) => "const",
        Item::AliasValue(_) => "alias",
        Item::InterfaceValue(_) => "interface",
        Item::ValuetypeValue(_) => "valuetype",
        Item::DeclValue(_) => "forward declaration",
    }
}

/// Get a human-readable name for an operator
#[must_use]
pub fn op_name(op: OpKind) -> &'static str {
    match op {
        OpKind::Add => "+",
        OpKind::Sub => "-",
        OpKind::Multiply => "*",
        OpKind::Divide => "/",
        OpKind::Modulo => "%",
        OpKind::And => "&",
        OpKind::Or => "|",
        OpKind::Xor => "^",
        OpKind::Lshift => "<<",
        OpKind::Rshift => ">>",
        OpKind::Not => "~",
    }
}

/// Get the identifier name from an Item, if it has one
#[must_use]
pub fn item_ident_name(item: &Item) -> Option<&str> {
    match item {
        Item::ModuleValue(v) => Some(&v.ident.name),
        Item::StructValue(v) => Some(&v.ident.name),
        Item::UnionValue(v) => Some(&v.ident.name),
        Item::EnumValue(v) => Some(&v.ident.name),
        Item::InterfaceValue(v) => Some(&v.ident.name),
        Item::ValuetypeValue(v) => Some(&v.ident.name),
        Item::ExceptionValue(v) => Some(&v.ident.name),
        Item::BitmaskValue(v) => Some(&v.ident.name),
        Item::BitsetValue(v) => Some(&v.ident.name),
        Item::AnnotationValue(v) => Some(&v.ident.name),
        Item::DeclValue(v) => Some(&v.ident.name),
        Item::ConstValue(v) => Some(decl_name(&v.decl)),
        Item::AliasValue(v) => v.decl.first().map(decl_name),
    }
}
