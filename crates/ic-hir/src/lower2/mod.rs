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

use ic_diagnostic::Diag;
use ic_syntax::Item;

use crate::Context;
use crate::hir::{DefId, DefKind, Ty, TyKind, TypeId};

mod builder;
mod builtin;
mod eval;
mod registry;
mod scope_manager;
mod type_items;
mod type_resolver;
mod utils;
mod validator;
mod value_items;

// pub use builtin::{lower_with_builtin_context, lower_with_builtins};
pub use registry::DefinitionRegistry;
pub use scope_manager::{ResolveMode, ScopeTree};

/// Result of the lowering process.
pub struct LoweringResult {
    /// The constructed HIR context with all definitions.
    pub context: Context,

    /// Top-level type IDs in order of appearance.
    pub order: Vec<TypeId>,

    /// Built-in type IDs in order of definition.
    pub builtin_order: Vec<TypeId>,

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
    let ast_items: Vec<Item> = ast.into_iter().collect();

    // Phase 1: Build & Resolve
    let mut context = LoweringContext::new();
    let mut builder = builder::HirBuilder::new(&mut context);
    builder.build(&ast_items);

    // Intermediate pass: Update forward references
    update_forward_references(&mut context);

    // Check for undefined forward declarations
    check_undefined_forward_decls(&mut context);

    // Phase 2: Validation
    let mut validator = validator::Validator::new(&context);
    validator.validate();

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
        builtin_order: Vec::new(),
        errors: diagnostics.errors,
        warnings: diagnostics.warnings,
    }
}

/// Core context for the lowering process.
pub(crate) struct LoweringContext {
    /// The HIR context being built.
    pub context: Context,

    /// Scope tree for name resolution.
    pub scopes: ScopeTree,

    /// Central registry for declarations and definitions.
    pub registry: DefinitionRegistry,

    /// Diagnostics collected during lowering.
    pub diagnostics: Diagnostics,

    /// Top-level type IDs in order.
    pub order: Vec<TypeId>,
}

impl LoweringContext {
    fn new() -> Self {
        let context = Context::new();
        let root_scope = context.scopes.root();

        Self {
            context,
            scopes: ScopeTree::new(root_scope),
            registry: DefinitionRegistry::new(),
            diagnostics: Diagnostics::new(),
            order: Vec::new(),
        }
    }

    /// Create a lowering context from existing components.
    pub fn from_existing(context: Context, errors: Vec<Diag>, warnings: Vec<Diag>) -> Self {
        let root_scope = context.scopes.root();

        // Create a new scope tree that's aware of existing scopes
        let scopes = ScopeTree::new(root_scope);

        // Create a new registry that's aware of existing definitions
        let registry = DefinitionRegistry::new();
        // The registry starts fresh for the new items

        Self {
            context,
            scopes,
            registry,
            diagnostics: Diagnostics {
                errors,
                warnings,
                has_critical_error: false,
            },
            order: Vec::new(),
        }
    }
}

/// Diagnostics collection during lowering.
pub(crate) struct Diagnostics {
    pub errors: Vec<Diag>,
    pub warnings: Vec<Diag>,
    pub has_critical_error: bool,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            has_critical_error: false,
        }
    }

    #[allow(dead_code)]
    pub fn error(&mut self, message: String, label: ic_diagnostic::Label) {
        use ic_diagnostic::error_span;
        self.errors.push(error_span(message, label));
    }

    pub fn warn(&mut self, message: String, label: ic_diagnostic::Label) {
        use ic_diagnostic::warn_span;
        self.warnings.push(warn_span(message, label));
    }

    pub fn critical_error(&mut self, message: String, label: ic_diagnostic::Label) {
        self.error(message, label);
        self.has_critical_error = true;
    }
}

/// Updates forward declaration references to point to definitions.
fn update_forward_references(ctx: &mut LoweringContext) {
    // Build mapping from forward decl DefIds to definition DefIds
    let mapping = ctx.registry.get_forward_to_def_mapping();

    if mapping.is_empty() {
        return; // No forward references to update
    }

    // Update all definitions to replace forward references
    let all_defs: Vec<DefId> = ctx.context.definitions.iter().map(|(id, _)| id).collect();

    for def_id in all_defs {
        update_def_references(&mut ctx.context, def_id, &mapping);
    }
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
            // Update parent reference if it points to a forward decl
            if let Some(parent) = &mut s.parent {
                if let Some(new_id) = mapping.get(parent) {
                    *parent = *new_id;
                }
            }

            // Update member types
            for member in &mut s.members {
                update_type_references(&mut member.ty, mapping);
            }
        }
        DefKind::Union(u) => {
            // Update discriminator type
            update_type_references(&mut u.disc, mapping);

            // Update variant types
            for variant in &mut u.variants {
                update_type_references(&mut variant.ty, mapping);
            }
        }
        DefKind::Interface(i) => {
            // Update parent interfaces
            for parent in &mut i.parents {
                if let Some(new_id) = mapping.get(parent) {
                    *parent = *new_id;
                }
            }

            // Update prototypes
            for proto in &mut i.prototypes {
                update_type_references(&mut proto.ty, mapping);
                for param in &mut proto.params {
                    update_type_references(&mut param.ty, mapping);
                }
            }

            // Update attributes
            for attr in &mut i.attributes {
                update_type_references(&mut attr.ty, mapping);
            }
        }
        DefKind::Valuetype(v) => {
            // Update parent reference
            if let Some(parent) = &mut v.parent {
                if let Some(new_id) = mapping.get(parent) {
                    *parent = *new_id;
                }
            }

            // Update supports reference
            if let Some(supports) = &mut v.supports {
                if let Some(new_id) = mapping.get(supports) {
                    *supports = *new_id;
                }
            }

            // Update members
            for member in &mut v.members {
                update_type_references(&mut member.ty, mapping);
            }

            // Update prototypes and attributes
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
        }
        DefKind::Enum(e) => {
            update_type_references(&mut e.ty, mapping);
        }
        DefKind::Bitmask(b) => {
            update_type_references(&mut b.ty, mapping);
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
        TyKind::Array { ty, .. } => {
            update_type_references(ty, mapping);
        }
        TyKind::Sequence { ty, .. } => {
            update_type_references(ty, mapping);
        }
        TyKind::Map { key, elem, .. } => {
            update_type_references(key, mapping);
            update_type_references(elem, mapping);
        }
        _ => {}
    }
}

/// Check for undefined forward declarations.
fn check_undefined_forward_decls(ctx: &mut LoweringContext) {
    use ic_diagnostic::{Label, error_span};

    // Get the mapping to see which forward declarations have definitions
    let mapping = ctx.registry.get_forward_to_def_mapping();

    // Check all forward declarations to see if they have definitions
    for def_id in &ctx.order {
        let def = ctx.context.definitions.get(*def_id);
        if let DefKind::Decl(_) = &def.kind {
            // This is a forward declaration - check if it has a matching definition
            if !mapping.contains_key(def_id) {
                ctx.diagnostics.errors.push(error_span(
                    format!("type `{}` is declared but not defined", def.ident.name),
                    Label::new(def.ident.span).message("declared here"),
                ));
            }
        }
    }
}
