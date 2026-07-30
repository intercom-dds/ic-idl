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

use crate::{Declarator, Item, Op, Path, Span, Type};

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
        Type::String(_) => "string".to_string(),
        Type::Map(_) => "map".to_string(),
        Type::Fixed(_) => "fixed".to_string(),
        Type::Sequence(seq) => format!("sequence<{}>", type_name(&seq.element)),
        Type::Named(ty) => path_name(ty),
    }
}

#[must_use]
pub fn element_type(path: &Type) -> String {
    match path {
        Type::Map(v) => element_type(&v.value),
        Type::Sequence(seq) => element_type(&seq.element),
        _ => type_name(path),
    }
}

#[must_use]
pub fn path_span(path: &Path) -> Span {
    let start = path.leading_colons.map_or_else(
        || {
            path.segments
                .first()
                .expect("path has a segment")
                .span
                .start
        },
        |v| v.start,
    );

    let end = path.segments.last().expect("path has a segment").span.end;
    Span { start, end }
}

#[must_use]
pub fn ty_span(ty: &Type) -> Span {
    match ty {
        Type::Sequence(v) => v.span,
        Type::String(v) => v.span,
        Type::Map(v) => v.span,
        Type::Fixed(v) => v.span,
        Type::Named(v) => path_span(v),
    }
}

#[must_use]
pub fn decl_name(decl: &Declarator) -> &str {
    match decl {
        Declarator::Name(v) => &v.name,
        Declarator::Array(v) => &v.name.name,
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
        Item::Annotation(v) => v.meta.span,
        Item::Module(v) => v.meta.span,
        Item::Struct(v) => v.meta.span,
        Item::Union(v) => v.meta.span,
        Item::Enum(v) => v.meta.span,
        Item::Exception(v) => v.meta.span,
        Item::Bitmask(v) => v.meta.span,
        Item::Bitset(v) => v.meta.span,
        Item::Const(v) => v.meta.span,
        Item::Alias(v) => v.meta.span,
        Item::Interface(v) => v.meta.span,
        Item::Valuetype(v) => v.meta.span,
        Item::Decl(v) => v.meta.span,
    }
}

#[must_use]
pub fn item_ident_span(item: &Item) -> Span {
    match item {
        Item::Annotation(v) => v.name.span,
        Item::Module(v) => v.name.span,
        Item::Struct(v) => v.name.span,
        Item::Union(v) => v.name.span,
        Item::Enum(v) => v.name.span,
        Item::Exception(v) => v.name.span,
        Item::Bitmask(v) => v.name.span,
        Item::Bitset(v) => v.name.span,
        Item::Const(v) => decl_span(&v.declarator),
        Item::Interface(v) => v.name.span,
        Item::Valuetype(v) => v.name.span,
        Item::Decl(v) => v.name.span,
        Item::Alias(v) => {
            if let (Some(first), Some(last)) = (
                v.declarators.first().map(decl_span),
                v.declarators.last().map(decl_span),
            ) {
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
        Declarator::Name(v) => v.span,
        Declarator::Array(v) => v.name.span,
    }
}

#[must_use]
pub fn item_variant_name(item: &Item) -> &'static str {
    match item {
        Item::Annotation(_) => "annotation",
        Item::Module(_) => "module",
        Item::Struct(_) => "struct",
        Item::Union(_) => "union",
        Item::Enum(_) => "enum",
        Item::Exception(_) => "exception",
        Item::Bitmask(_) => "bitmask",
        Item::Bitset(_) => "bitset",
        Item::Const(_) => "const",
        Item::Alias(_) => "alias",
        Item::Interface(_) => "interface",
        Item::Valuetype(_) => "valuetype",
        Item::Decl(_) => "forward declaration",
    }
}

/// Get a human-readable name for an operator
#[must_use]
pub fn op_name(op: Op) -> &'static str {
    match op {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Multiply => "*",
        Op::Divide => "/",
        Op::Modulo => "%",
        Op::And => "&",
        Op::Or => "|",
        Op::Xor => "^",
        Op::LShift => "<<",
        Op::RShift => ">>",
        Op::Not => "~",
    }
}

/// Get the identifier name from an Item, if it has one
#[must_use]
pub fn item_ident_name(item: &Item) -> Option<&str> {
    match item {
        Item::Module(v) => Some(&v.name.name),
        Item::Struct(v) => Some(&v.name.name),
        Item::Union(v) => Some(&v.name.name),
        Item::Enum(v) => Some(&v.name.name),
        Item::Interface(v) => Some(&v.name.name),
        Item::Valuetype(v) => Some(&v.name.name),
        Item::Exception(v) => Some(&v.name.name),
        Item::Bitmask(v) => Some(&v.name.name),
        Item::Bitset(v) => Some(&v.name.name),
        Item::Annotation(v) => Some(&v.name.name),
        Item::Decl(v) => Some(&v.name.name),
        Item::Const(v) => Some(decl_name(&v.declarator)),
        Item::Alias(v) => v.declarators.first().map(decl_name),
    }
}
