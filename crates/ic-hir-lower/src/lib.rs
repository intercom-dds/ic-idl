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

//! Lowers `ic-syntax` AST to `ic-hir` HIR.
//!
//! This crate transforms the parse tree (AST) into a typed, resolved
//! representation suitable for semantic analysis and code generation. The HIR
//! resolves names, performs type checking, and evaluates constant expressions.
//!
//! # Example
//!
//! ```ignore
//! use ic_hir_lower::{from_ast, AstInput};
//! let ast = parse_idl_file("example.idl")?;
//! let hir = from_ast(AstInput::User(ast));
//!
//! if !hir.errors.is_empty() {
//!     // Handle compilation errors
//! }
//!
//! // Use the HIR for code generation or analysis
//! for def in hir.iter() {
//!     println!("Found type: {}", def.ident.name);
//! }
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

use std::collections::HashMap;

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::Diag;
use ic_hir::Context;
use ic_hir::hir::{DefFlags, DefId};
use ic_hir::scope::ScopeId;
use ic_syntax::{Item, Span};
use tracing::{debug, debug_span, info_span};

mod annotation;
mod builder;
mod define;
mod eval;
mod initializers;
mod registry;
mod resolve;
mod type_items;
mod utils;
mod value_items;

pub use registry::DefinitionRegistry;

/// Input for HIR lowering, supporting both user-only and user+builtins scenarios.
pub enum AstInput<I> {
    /// Just user AST, no builtins
    User(I),

    /// User AST with builtin definitions
    WithBuiltins {
        /// Built-in definitions that will be marked with `IS_BUILTIN` flag
        builtins: I,

        /// User definitions
        user: I,

        /// If true, builtins are included in the output order.
        /// If false, they're available in context but not in the output.
        include_in_output: bool,
    },
}

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

/// Convert AST to HIR with the specified input configuration.
pub fn from_ast<I>(input: AstInput<I>) -> ic_hir::ResolvedGraph
where
    I: IntoIterator<Item = Item>,
{
    let result = match input {
        AstInput::User(ast) => lower(None, ast, false),
        AstInput::WithBuiltins {
            builtins,
            user,
            include_in_output,
        } => lower(Some(builtins), user, include_in_output),
    };

    ic_hir::ResolvedGraph {
        context: result.context,
        order: result.order,
        builtin_order: result.builtin_order,
        errors: result.errors,
        warnings: result.warnings,
    }
}

fn lower<I>(builtins: Option<I>, user: I, include_in_output: bool) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    let _span = info_span!("hir_lowering").entered();
    let user_items: Vec<Item> = user.into_iter().collect();
    let mut context = LoweringContext::new();

    let builtin_order = if let Some(builtins) = builtins {
        let _builtin_span = debug_span!("builtins").entered();
        let builtin_items: Vec<Item> = builtins.into_iter().collect();
        debug!(
            item_count = builtin_items.len(),
            "lowering builtin definitions"
        );

        let mut builder = builder::HirBuilder::new(&mut context);
        builder.build(&builtin_items);

        let builtin_ids = context.order.clone();
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

    {
        let _user_span = debug_span!("user").entered();
        debug!(item_count = user_items.len(), "lowering user definitions");
        let mut builder = builder::HirBuilder::new(&mut context);
        builder.build(&user_items);
    }

    {
        let _fwd_span = debug_span!("forward_refs").entered();
        debug!("updating forward references");
        let forward_to_def = context.registry.get_forward_to_def_mapping();
        ic_hir::rewrite::replace_def_ids(&mut context.context, &forward_to_def);

        // Check for undefined forward declarations
        let errors =
            ic_hir::validate::check_undefined_forward_decls(&context.context, &forward_to_def);
        context.diagnostics.errors.extend(errors);
    }

    LoweringResult {
        context: context.context,
        order: context.order,
        builtin_order,
        errors: context.diagnostics.errors,
        warnings: context.diagnostics.warnings,
    }
}

/// Core context for the lowering process.
pub(crate) struct LoweringContext {
    /// The HIR context being built.
    pub context: Context,

    /// Central registry for declarations and definitions.
    pub registry: DefinitionRegistry,

    /// Diagnostics collected during lowering.
    pub diagnostics: ic_hir::diagnostics::Diagnostics,

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
            diagnostics: ic_hir::diagnostics::Diagnostics::new(),
            module_scopes: HashMap::new(),
            order: Vec::new(),
        }
    }
}
