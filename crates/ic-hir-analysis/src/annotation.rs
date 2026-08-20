// Copyright 2026 KONGSBERG
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

use ic_hir::Context;
use ic_hir::hir::{Ann, AnnParam, Def, DefFlags, DefKind, Disc, Member, Numeric, Variant};

pub trait MemberLike {
    fn annotations(&self) -> &[Ann];
}

impl MemberLike for Member {
    fn annotations(&self) -> &[Ann] {
        &self.annotations
    }
}

impl MemberLike for Disc {
    fn annotations(&self) -> &[Ann] {
        &self.annotations
    }
}

impl MemberLike for Variant {
    fn annotations(&self) -> &[Ann] {
        &self.annotations
    }
}

pub trait ExternalTarget {
    fn annotations(&self) -> &[Ann];
}

impl<T: MemberLike> ExternalTarget for T {
    fn annotations(&self) -> &[Ann] {
        MemberLike::annotations(self)
    }
}

impl ExternalTarget for Def {
    fn annotations(&self) -> &[Ann] {
        &self.annotations
    }
}

pub trait DefaultTarget {
    fn annotations(&self) -> &[Ann];
}

impl<T: MemberLike> DefaultTarget for T {
    fn annotations(&self) -> &[Ann] {
        MemberLike::annotations(self)
    }
}

impl DefaultTarget for Def {
    fn annotations(&self) -> &[Ann] {
        &self.annotations
    }
}

impl DefaultTarget for AnnParam {
    fn annotations(&self) -> &[Ann] {
        &self.annotations
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Extensibility {
    Final,
    #[default]
    Appendable,
    Mutable,
}

#[must_use]
pub fn extensibility(ctx: &Context, def: &Def) -> Extensibility {
    if builtin_annotation(ctx, &def.annotations, "final").is_some() {
        return Extensibility::Final;
    }

    if builtin_annotation(ctx, &def.annotations, "mutable").is_some() {
        return Extensibility::Mutable;
    }

    if builtin_annotation(ctx, &def.annotations, "appendable").is_some() {
        return Extensibility::Appendable;
    }

    let Some(annotation) = builtin_annotation(ctx, &def.annotations, "extensibility") else {
        return Extensibility::Appendable;
    };

    let Some(value) = annotation.args.first() else {
        return Extensibility::Appendable;
    };

    match ctx.unsigned_value(&value.value) {
        0 => Extensibility::Final,
        2 => Extensibility::Mutable,
        _ => Extensibility::Appendable,
    }
}

pub fn key_annotation<'a>(ctx: &Context, target: &'a impl MemberLike) -> Option<&'a Ann> {
    enabled_bool_annotation(ctx, target.annotations(), "key")
}

pub fn is_key(ctx: &Context, target: &impl MemberLike) -> bool {
    key_annotation(ctx, target).is_some()
}

pub fn optional_annotation<'a>(ctx: &Context, target: &'a impl MemberLike) -> Option<&'a Ann> {
    enabled_bool_annotation(ctx, target.annotations(), "optional")
}

pub fn is_optional(ctx: &Context, target: &impl MemberLike) -> bool {
    optional_annotation(ctx, target).is_some()
}

pub fn default_annotation<'a>(ctx: &Context, target: &'a impl DefaultTarget) -> Option<&'a Ann> {
    builtin_annotation(ctx, target.annotations(), "default")
}

pub fn default_value<'a>(ctx: &Context, target: &'a impl DefaultTarget) -> Option<&'a Numeric> {
    default_annotation(ctx, target)
        .and_then(|annotation| annotation.args.first())
        .map(|argument| &argument.value)
}

#[must_use]
pub fn bit_bound_annotation<'a>(ctx: &Context, def: &'a Def) -> Option<&'a Ann> {
    builtin_annotation(ctx, &def.annotations, "bit_bound")
}

#[must_use]
pub fn bit_bound<'a>(ctx: &Context, def: &'a Def) -> Option<&'a Numeric> {
    bit_bound_annotation(ctx, def)
        .and_then(|annotation| annotation.args.first())
        .map(|argument| &argument.value)
}

#[must_use]
pub fn is_default_literal(ctx: &Context, def: &Def) -> bool {
    builtin_annotation(ctx, &def.annotations, "default_literal").is_some()
}

#[must_use]
pub fn doc(ctx: &Context, annotation: &Ann) -> Option<String> {
    if builtin_annotation_name(ctx, annotation) != Some("doc") {
        return None;
    }

    annotation
        .args
        .first()
        .and_then(|argument| ctx.string_value(&argument.value))
}

#[must_use]
pub fn is_extensibility_annotation(ctx: &Context, annotation: &Ann) -> bool {
    builtin_annotation_name(ctx, annotation)
        .is_some_and(|name| matches!(name, "final" | "mutable" | "appendable" | "extensibility"))
}

#[must_use]
pub fn is_external(ctx: &Context, target: &impl ExternalTarget) -> bool {
    bool_annotation(ctx, target.annotations(), "external").unwrap_or(false)
        || bool_annotation(ctx, target.annotations(), "shared").unwrap_or(false)
}

#[must_use]
pub fn is_newtype(ctx: &Context, def: &Def) -> bool {
    bool_annotation(ctx, &def.annotations, "newtype").unwrap_or(false)
}

pub fn is_must_understand(ctx: &Context, target: &impl MemberLike) -> bool {
    bool_annotation(ctx, target.annotations(), "must_understand").unwrap_or(false)
}

#[must_use]
pub fn is_non_serialized(ctx: &Context, target: &impl MemberLike) -> bool {
    bool_annotation(ctx, target.annotations(), "non_serialized").unwrap_or(false)
}

#[must_use]
pub fn is_nested(ctx: &Context, def: &Def) -> bool {
    if let Some(nested) = bool_annotation(ctx, &def.annotations, "nested") {
        return nested;
    }

    let mut parent = def.parent;
    while let Some(parent_id) = parent {
        let parent_def = ctx.type_of(parent_id);
        if matches!(parent_def.kind, DefKind::Module(_))
            && let Some(nested) = bool_annotation(ctx, &parent_def.annotations, "default_nested")
        {
            return nested;
        }
        parent = parent_def.parent;
    }

    false
}

fn bool_annotation(ctx: &Context, annotations: &[Ann], name: &str) -> Option<bool> {
    let annotation = builtin_annotation(ctx, annotations, name)?;
    Some(annotation_enabled(ctx, annotation))
}

fn enabled_bool_annotation<'a>(
    ctx: &Context,
    annotations: &'a [Ann],
    name: &str,
) -> Option<&'a Ann> {
    let annotation = builtin_annotation(ctx, annotations, name)?;
    annotation_enabled(ctx, annotation).then_some(annotation)
}

fn annotation_enabled(ctx: &Context, annotation: &Ann) -> bool {
    annotation
        .args
        .first()
        .is_none_or(|argument| ctx.unsigned_value(&argument.value) != 0)
}

#[must_use]
pub fn builtin_annotation<'a>(
    ctx: &Context,
    annotations: &'a [Ann],
    name: &str,
) -> Option<&'a Ann> {
    annotations
        .iter()
        .find(|annotation| is_builtin_annotation(ctx, annotation, name))
}

#[must_use]
pub fn builtin_annotation_def<'a>(ctx: &'a Context, annotation: &Ann) -> Option<&'a Def> {
    let def = ctx.base_def_of(annotation.def_id?);
    def.flags.contains(DefFlags::IS_BUILTIN).then_some(def)
}

#[must_use]
pub fn is_builtin_annotation(ctx: &Context, annotation: &Ann, name: &str) -> bool {
    builtin_annotation_def(ctx, annotation).is_some_and(|def| def.ident.name == name)
}

fn builtin_annotation_name<'a>(ctx: &'a Context, annotation: &Ann) -> Option<&'a str> {
    builtin_annotation_def(ctx, annotation).map(|def| def.ident.name.as_str())
}
