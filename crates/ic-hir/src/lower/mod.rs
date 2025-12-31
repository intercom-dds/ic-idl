// Copyright 2025 KONGSBERG
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

//! New lowering implementation from AST to HIR.
//!
//! This module implements a simplified 2-phase lowering process:
//! 1. Build & Resolve - Process items in order, creating definitions and resolving types
//! 2. Validation - Semantic validation on the complete HIR
//!
//! A small intermediate pass updates forward declaration references to point to definitions.

use std::collections::HashMap;

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::Diag;
use ic_syntax::{Item, Span};
use tracing::{debug, debug_span, info_span};

use crate::Context;
use crate::hir::{Decl, DefFlags, DefId, DefKind, Ty, TyKind};
use crate::scope::ScopeId;

mod annotation_common;
mod builder;
mod eval;
mod initializers;
mod registry;
mod type_items;
mod type_resolver;
mod utils;
mod value_items;

pub use registry::DefinitionRegistry;

/// Result of the lowering process.
pub struct LoweringResult {
    /// The constructed HIR context with all definitions.
    pub context: Context,

    /// Top-level type IDs in order of appearance.
    pub order: Vec<DefId>,

    /// Built-in type IDs in order of definition.
    pub builtin_order: Vec<DefId>,

    /// Errors collected during all phases.
    pub errors: Vec<Diag>,

    /// Warnings collected during all phases.
    pub warnings: Vec<Diag>,
}

/// Lowers AST items to HIR through the new 2-phase process.
pub fn lower<I>(ast: I) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    lower_internal(None, ast, false)
}

/// Lowers AST items with builtins.
pub fn lower_with_builtins<I>(builtins: I, user: I, include_in_output: bool) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    lower_internal(Some(builtins), user, include_in_output)
}

fn lower_internal<I>(builtins: Option<I>, user: I, include_in_output: bool) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    let _span = info_span!("hir_lowering").entered();

    let user_items: Vec<Item> = user.into_iter().collect();

    let mut context = LoweringContext::new();

    // Process builtins if provided
    let builtin_order = if let Some(builtins) = builtins {
        let _builtin_span = debug_span!("builtins").entered();
        let builtin_items: Vec<Item> = builtins.into_iter().collect();
        debug!(
            item_count = builtin_items.len(),
            "lowering builtin definitions"
        );

        let mut builder = builder::HirBuilder::new(&mut context);
        builder.build(&builtin_items);

        // Save builtin def IDs from order
        let builtin_ids = context.order.clone();

        // Mark all builtins with IS_BUILTIN flag
        for &def_id in &builtin_ids {
            context.context.definitions.get_mut(def_id).flags |= DefFlags::IS_BUILTIN;
        }

        if !include_in_output {
            context.order.clear();
        }

        builtin_ids
    } else {
        Vec::new()
    };

    // Process user items
    {
        let _user_span = debug_span!("user").entered();
        debug!(item_count = user_items.len(), "lowering user definitions");
        let mut builder = builder::HirBuilder::new(&mut context);
        builder.build(&user_items);
    }

    // Intermediate pass: Update forward references
    {
        let _fwd_span = debug_span!("forward_refs").entered();
        debug!("updating forward references");
        update_forward_references(&mut context);
    }

    // Check for undefined forward declarations
    check_undefined_forward_decls(&mut context);

    // Extract results
    let LoweringContext {
        context,
        order,
        diagnostics,
        ..
    } = context;

    LoweringResult {
        context,
        order,
        builtin_order,
        errors: diagnostics.errors,
        warnings: diagnostics.warnings,
    }
}

/// Core context for the lowering process.
pub(crate) struct LoweringContext {
    /// The HIR context being built.
    pub context: Context,

    /// Central registry for declarations and definitions.
    pub registry: DefinitionRegistry,

    /// Diagnostics collected during lowering.
    pub diagnostics: Diagnostics,

    /// Top-level type IDs in order.
    pub order: Vec<DefId>,

    /// Module reopening tracking during lowering.
    /// Maps from `parent_scope` to a `CaseMap` of module names to `(scope_id, original_span)`.
    pub module_scopes: HashMap<ScopeId, CaseMap<(ScopeId, Span)>>,
}

impl LoweringContext {
    fn new() -> Self {
        Self {
            context: Context::new(),
            registry: DefinitionRegistry::new(),
            diagnostics: Diagnostics::new(),
            module_scopes: HashMap::new(),
            order: Vec::new(),
        }
    }
}

/// Diagnostics collection during lowering.
pub struct Diagnostics {
    pub errors: Vec<Diag>,
    pub warnings: Vec<Diag>,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn error(&mut self, message: String, label: ic_diagnostic::Label) {
        use ic_diagnostic::error_span;
        self.errors.push(error_span(message, label));
    }
}

/// Updates forward declaration references to point to definitions.
fn update_forward_references(
    ctx: &mut LoweringContext,
) -> std::collections::HashMap<DefId, Vec<DefId>> {
    // Build mapping from forward decl DefIds to definition DefIds
    let forward_to_def = ctx.registry.get_forward_to_def_mapping();

    if !forward_to_def.is_empty() {
        // Update all definitions to replace forward references
        let all_defs: Vec<DefId> = ctx.context.definitions.iter().map(|(id, _)| id).collect();

        for def_id in all_defs {
            update_def_references(&mut ctx.context, def_id, &forward_to_def);
        }
    }

    // Build the inverse mapping: from definition to forward declarations
    let mut def_to_forwards = std::collections::HashMap::new();
    for (forward_id, def_id) in forward_to_def {
        def_to_forwards
            .entry(def_id)
            .or_insert_with(Vec::new)
            .push(forward_id);
    }

    def_to_forwards
}

/// Update references in a single definition.
fn update_def_references(
    ctx: &mut crate::Context,
    def_id: DefId,
    mapping: &std::collections::HashMap<DefId, DefId>,
) {
    let def = ctx.definitions.get_mut(def_id);

    match &mut def.kind {
        DefKind::Struct(s) => {
            if let Some(parent) = &mut s.parent
                && let Some(new_id) = mapping.get(parent)
            {
                *parent = *new_id;
            }

            for member in &mut s.members {
                update_type_references(&mut member.ty, mapping);
            }
        }
        DefKind::Union(u) => {
            update_type_references(&mut u.disc.ty, mapping);

            for variant in &mut u.variants {
                update_type_references(&mut variant.ty, mapping);
            }
        }
        DefKind::Interface(i) => {
            for parent in &mut i.parents {
                if let Some(new_id) = mapping.get(parent) {
                    *parent = *new_id;
                }
            }

            for proto in &mut i.prototypes {
                update_type_references(&mut proto.ty, mapping);
                for param in &mut proto.params {
                    update_type_references(&mut param.ty, mapping);
                }
            }

            for attr in &mut i.attributes {
                update_type_references(&mut attr.ty, mapping);
            }
        }
        DefKind::Valuetype(v) => {
            if let Some(parent) = &mut v.parent
                && let Some(new_id) = mapping.get(parent)
            {
                *parent = *new_id;
            }

            if let Some(supports) = &mut v.supports
                && let Some(new_id) = mapping.get(supports)
            {
                *supports = *new_id;
            }

            for member in &mut v.members {
                update_type_references(&mut member.ty, mapping);
            }

            for proto in &mut v.prototypes {
                update_type_references(&mut proto.ty, mapping);
                for param in &mut proto.params {
                    update_type_references(&mut param.ty, mapping);
                }
            }

            for attr in &mut v.attributes {
                update_type_references(&mut attr.ty, mapping);
            }
        }
        DefKind::Alias(a) => {
            update_type_references(&mut a.ty, mapping);
        }
        DefKind::Const(c) => {
            update_type_references(&mut c.ty, mapping);
            update_numeric_references(&mut c.value, mapping);
        }
        DefKind::Except(e) => {
            for member in &mut e.members {
                update_type_references(&mut member.ty, mapping);
            }
        }
        _ => {}
    }
}

/// Update type references to replace forward decl `DefIds` with definition `DefIds`.
fn update_type_references(ty: &mut Ty, mapping: &std::collections::HashMap<DefId, DefId>) {
    match &mut ty.kind {
        TyKind::Adt(def_id) => {
            if let Some(new_id) = mapping.get(def_id) {
                *def_id = *new_id;
            }
        }
        TyKind::Array { ty, .. } | TyKind::Sequence { ty, .. } => {
            update_type_references(ty, mapping);
        }
        TyKind::Map { key, elem, .. } => {
            update_type_references(key, mapping);
            update_type_references(elem, mapping);
        }
        _ => {}
    }
}

/// Update numeric references to replace forward decl `DefIds` with definition `DefIds`.
fn update_numeric_references(
    numeric: &mut crate::hir::Numeric,
    mapping: &std::collections::HashMap<DefId, DefId>,
) {
    use crate::hir::Numeric;

    match numeric {
        Numeric::Const(def_id) => {
            if let Some(new_id) = mapping.get(def_id) {
                *def_id = *new_id;
            }
        }
        Numeric::Array { ty, values } | Numeric::Sequence { ty, values } => {
            update_type_references(ty, mapping);
            for value in values.iter_mut() {
                update_numeric_references(value, mapping);
            }
        }
        Numeric::Map {
            key,
            value,
            entries,
        } => {
            update_type_references(key, mapping);
            update_type_references(value, mapping);
            for (k, v) in entries.iter_mut() {
                update_numeric_references(k, mapping);
                update_numeric_references(v, mapping);
            }
        }
        Numeric::Struct { ty, fields } => {
            if let Some(new_id) = mapping.get(ty) {
                *ty = *new_id;
            }
            for field_value in fields.iter_mut() {
                update_numeric_references(field_value, mapping);
            }
        }
        Numeric::Union {
            ty,
            discriminant,
            field_index: _,
            value,
        } => {
            if let Some(new_id) = mapping.get(ty) {
                *ty = *new_id;
            }
            update_numeric_references(discriminant, mapping);
            update_numeric_references(value, mapping);
        }
        _ => {}
    }
}

/// Check for undefined forward declarations.
fn check_undefined_forward_decls(ctx: &mut LoweringContext) {
    use ic_diagnostic::{Label, error_span};

    // Get the mapping to see which forward declarations have definitions
    let mapping = ctx.registry.get_forward_to_def_mapping();

    // Check ALL forward declarations in the context, not just top-level ones
    // The registry tracks all forward declarations, including nested ones
    for (def_id, def) in &ctx.context.definitions {
        if let DefKind::Decl(decl_kind) = &def.kind {
            // Native declarations are meant to stay as declarations - skip them
            if matches!(decl_kind, Decl::Native) {
                continue;
            }

            // This is a forward declaration - check if it has a matching definition
            if !mapping.contains_key(&def_id) {
                ctx.diagnostics.errors.push(error_span(
                    format!("type `{}` is declared but not defined", def.ident.name),
                    Label::new(def.ident.span).message("declared here"),
                ));
            }
        }
    }
}
