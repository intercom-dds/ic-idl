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

//! Lowering from AST to HIR.
//!
//! The lowering process uses a single-pass approach for resolution,
//! followed by separate phases for evaluation, type checking, and validation.
//!
//! ## Phases
//!
//! 1. **Resolution**: Processes items in order, creating HIR nodes and
//!    resolving type references as they appear
//! 2. **Evaluation**: Evaluates constant expressions and enum values
//! 3. **Type Checking**: Validates that values match their declared types
//! 4. **Validation**: Performs semantic validation and consistency checks
//!
//! ## Design Principles
//!
//! - The resolver processes definitions in order, allowing forward references
//! - Errors are collected, not thrown, to provide comprehensive diagnostics
//! - Each phase after collection/resolution can be tested independently
//! - Parent relationships and cross-references are maintained during resolution

use ic_diagnostic::Diag;
use ic_syntax::Item;

use crate::Context;
use crate::hir::{DefId, TypeId};

mod builtin;
mod evaluate;
mod resolve;
mod typecheck;
mod validate;

pub use builtin::{lower_with_builtin_context, lower_with_builtins};

/// Converts an annotation argument value (expression) to a Numeric value
fn convert_annotation_value(
    expr: &ic_syntax::Expr,
    resolver: &mut resolve::Resolver,
    annotation_def_id: Option<DefId>,
) -> crate::hir::Numeric {
    match expr {
        ic_syntax::Expr::Literal(lit) => match &lit.value {
            ic_syntax::LiteralValue::Bool(b) => crate::hir::Numeric::Bool(*b),
            ic_syntax::LiteralValue::Int(i) =>
            {
                #[allow(clippy::cast_possible_truncation)]
                crate::hir::Numeric::Int32(*i as i32)
            }
            ic_syntax::LiteralValue::Float(f) => crate::hir::Numeric::Double(*f),
            ic_syntax::LiteralValue::String(s) => crate::hir::Numeric::String(s.clone()),
            _ => crate::hir::Numeric::Null,
        },
        ic_syntax::Expr::Path(path) => {
            // For annotation arguments, first try to resolve in the annotation's own scope
            if let Some(ann_def_id) = annotation_def_id {
                if let Some(def_id) = resolver.resolve_path_in_annotation_scope(path, ann_def_id) {
                    return crate::hir::Numeric::Const(def_id);
                }
            }

            // Then try to resolve in the current scope
            if let Some(def_id) = resolver.resolve_path(path) {
                // Return a reference to the constant - the actual value will be resolved
                // during expression evaluation phase
                crate::hir::Numeric::Const(def_id)
            } else {
                // Handle enum-qualified paths like foo::Status::NOT_FOUND
                // where foo::Status is the enum type and NOT_FOUND is the enumerator
                if path.segments.len() >= 2 {
                    // Try to resolve all but the last segment as the enum type
                    let enum_path = ic_syntax::Path {
                        leading_colons: path.leading_colons,
                        segments: path.segments[..path.segments.len() - 1].to_vec(),
                    };

                    if let Some(enum_def_id) = resolver.resolve_path(&enum_path) {
                        let enum_def = resolver.get_definition(enum_def_id);
                        if let crate::hir::DefKind::Enum(enum_ty) = &enum_def.kind {
                            // Look for the enumerator in this enum
                            let enumerator_name = &path.segments.last().unwrap().name;
                            for &field_id in &enum_ty.fields {
                                let field_def = resolver.get_definition(field_id);
                                if field_def.ident.name == *enumerator_name {
                                    return crate::hir::Numeric::Const(field_id);
                                }
                            }
                        }
                    }
                }
                crate::hir::Numeric::Null
            }
        }
        _ => crate::hir::Numeric::Null, // Other expressions need evaluation
    }
}

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

/// Lowers AST items to HIR through multiple phases.
pub fn lower<I>(ast: I) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    let ast_items: Vec<Item> = ast.into_iter().collect();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Resolution phase
    let mut context = Context::new();
    let lowerer = resolve::Resolver::new(&mut context);
    let (order, mut phase_errors, mut phase_warnings) = lowerer.process(&ast_items);
    errors.append(&mut phase_errors);
    warnings.append(&mut phase_warnings);

    // Evaluate constant expressions
    let mut phase_errors = evaluate::evaluate_expressions(&mut context, &ast_items);
    errors.append(&mut phase_errors);

    // Type check values against their declared types
    let mut phase_errors = typecheck::typecheck_hir(&context, &order);
    errors.append(&mut phase_errors);

    // Validate the HIR
    let mut phase_errors = validate::validate_hir(&context, &order);
    errors.append(&mut phase_errors);

    LoweringResult {
        context,
        order,
        builtin_order: Vec::new(),
        errors,
        warnings,
    }
}
