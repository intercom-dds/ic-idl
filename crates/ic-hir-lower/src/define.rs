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

use ic_hir::hir::{Ann, Decl, Def, DefFlags, DefId, DefKind};
use ic_hir::scope::ScopeId;
use ic_syntax::{Annotation, Ident, Span};

use crate::LoweringContext;
use crate::annotation::convert_annotations;
use crate::registry::DefKindTag;

pub fn define(
    ctx: &mut LoweringContext,
    scope: ScopeId,
    ident: &Ident,
    span: Span,
    ast_annotations: &[Annotation],
    kind_tag: DefKindTag,
    build_kind: impl FnOnce(DefId) -> DefKind,
) -> DefId {
    let annotations = convert_annotations(ctx, ast_annotations, scope);
    let def_id = ctx.context.definitions.alloc_with_id(|id| Def {
        id,
        ident: ident.clone(),
        parent: ctx.context.scopes.get_scope(scope).def_id,
        annotations,
        span,
        kind: build_kind(id),
        flags: DefFlags::nil(),
    });

    let registered = ctx.registry.register_definition(
        scope,
        ident,
        kind_tag,
        def_id,
        &mut ctx.diagnostics,
        &ctx.context,
    );
    if registered == Some(def_id) {
        ctx.context
            .scopes
            .add_definition(scope, ident.name.clone(), def_id);
    }

    def_id
}

#[allow(clippy::too_many_arguments)]
pub fn define_scoped_const(
    ctx: &mut LoweringContext,
    scope: ScopeId,
    extra_scope: ScopeId,
    ident: &Ident,
    span: Span,
    annotations: Vec<Ann>,
    flags: DefFlags,
    build_kind: impl FnOnce(DefId) -> DefKind,
) -> DefId {
    let def_id = ctx.context.definitions.alloc_with_id(|id| Def {
        id,
        ident: ident.clone(),
        parent: ctx.context.scopes.get_scope(extra_scope).def_id,
        annotations,
        span,
        kind: build_kind(id),
        flags,
    });

    let registered = ctx.registry.register_definition(
        scope,
        ident,
        DefKindTag::Const,
        def_id,
        &mut ctx.diagnostics,
        &ctx.context,
    );
    if registered == Some(def_id) {
        ctx.context
            .scopes
            .add_definition(scope, ident.name.clone(), def_id);
        ctx.context
            .scopes
            .add_definition(extra_scope, ident.name.clone(), def_id);
    }

    def_id
}

pub fn declare_forward(
    ctx: &mut LoweringContext,
    scope: ScopeId,
    ident: &Ident,
    decl_kind: Decl,
) -> DefId {
    let def_id = ctx.context.definitions.alloc_with_id(|id| Def {
        id,
        ident: ident.clone(),
        parent: ctx.context.scopes.get_scope(scope).def_id,
        annotations: Vec::new(),
        span: ident.span,
        kind: DefKind::Decl(decl_kind),
        flags: DefFlags::IS_INCOMPLETE,
    });

    if let Some(existing_id) = ctx.registry.register_forward_decl(
        scope,
        ident,
        decl_kind,
        def_id,
        &mut ctx.diagnostics,
        &ctx.context,
    ) && existing_id != def_id
    {
        return existing_id;
    }

    ctx.context
        .scopes
        .add_definition(scope, ident.name.clone(), def_id);
    def_id
}

pub fn define_annotation(
    ctx: &mut LoweringContext,
    scope: ScopeId,
    ident: &Ident,
    def_id: DefId,
    is_consistent: impl FnOnce(&Def, &Def, &ic_hir::Context) -> bool,
) {
    let ann_key = format!("@{}", ident.name);
    let existing = ctx
        .context
        .scopes
        .get_scope(scope)
        .definitions
        .get(&ann_key)
        .and_then(|ids| ids.last().copied());

    if let Some(existing_id) = existing {
        let existing_def = ctx.context.definitions.get(existing_id);
        let new_def = ctx.context.definitions.get(def_id);
        if !is_consistent(existing_def, new_def, &ctx.context) {
            use ic_diagnostic::{Label, error_span};
            ctx.diagnostics.errors.push(
                error_span(
                    format!("inconsistent redefinition of annotation `@{}`", ident.name),
                    Label::new(existing_def.ident.span).message("originally defined here"),
                )
                .label(Label::new(ident.span).message("redefined inconsistently here"))
                .note(
                    "annotation redefinitions must have the same parameters, types, and defaults",
                ),
            );
        }
    }

    ctx.context
        .scopes
        .add_annotation(scope, &ident.name, def_id);
}
