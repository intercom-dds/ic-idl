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

//! Multi-phase lowering from AST to HIR.
//!
//! The lowering process is split into distinct phases to improve maintainability,
//! testability, and clarity. Each phase has a single responsibility and produces
//! well-defined outputs.
//!
//! ## Phases
//!
//! 1. **Collection**: Discovers all definitions and creates placeholder HIR nodes
//! 2. **Resolution**: Resolves type references and builds the type graph
//! 3. **Evaluation**: Evaluates constant expressions and enum values
//! 4. **Type Checking**: Validates that values match their declared types
//! 5. **Validation**: Performs semantic validation and consistency checks
//!
//! ## Design Principles
//!
//! - Each phase is independent and can be tested in isolation
//! - Errors are collected, not thrown, to provide comprehensive diagnostics
//! - The HIR is incrementally built and refined through each phase
//! - Parent relationships and cross-references are explicitly maintained

use ic_diagnostic::Diag;
use ic_syntax::Item;

use crate::Context;
use crate::hir::TypeId;

mod collect;
mod evaluate;
mod resolve;
mod typecheck;
mod validate;

/// Converts AST annotations to HIR annotations
pub(crate) fn convert_annotations(ast_anns: &[ic_syntax::AnnotationAppl]) -> Vec<crate::hir::Ann> {
    ast_anns
        .iter()
        .map(|ann| {
            crate::hir::Ann {
                path: ann.ident.clone(),
                ty: None, // Annotations don't have types in the AST
                args: ann
                    .args
                    .iter()
                    .map(|arg| crate::hir::AnnArg {
                        ident: arg.ident.clone(),
                        value: convert_annotation_value(&arg.value),
                    })
                    .collect(),
            }
        })
        .collect()
}

/// Converts an annotation argument value (expression) to a Numeric value
fn convert_annotation_value(expr: &ic_syntax::Expr) -> crate::hir::Numeric {
    // For now, only handle literal expressions
    // TODO: Full expression evaluation should happen in the evaluate phase
    match expr {
        ic_syntax::Expr::Literal(lit) => match &lit.value {
            ic_syntax::LiteralValue::Bool(b) => crate::hir::Numeric::Bool(*b),
            ic_syntax::LiteralValue::Int(i) => crate::hir::Numeric::Int32(*i as i32),
            ic_syntax::LiteralValue::Float(f) => crate::hir::Numeric::Double(*f),
            ic_syntax::LiteralValue::String(s) => crate::hir::Numeric::String(s.clone()),
            _ => crate::hir::Numeric::Null,
        },
        _ => crate::hir::Numeric::Null, // Non-literal expressions need evaluation
    }
}

/// Result of the lowering process.
pub struct LoweringResult {
    /// The constructed HIR context with all definitions.
    pub context: Context,

    /// Top-level type IDs in order of appearance.
    pub order: Vec<TypeId>,

    /// Errors collected during all phases.
    pub errors: Vec<Diag>,
}

/// Lowers AST items to HIR through multiple phases.
pub fn lower<I>(ast: I) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    let ast_items: Vec<Item> = ast.into_iter().collect();
    let mut errors = Vec::new();

    // Phase 1: Collect all names and create placeholder definitions
    let (mut context, name_map, order, mut phase_errors) = collect::collect_definitions(&ast_items);
    errors.append(&mut phase_errors);

    // Phase 2: Resolve type references
    let mut phase_errors = resolve::resolve_types(&mut context, &name_map, &ast_items);
    errors.append(&mut phase_errors);

    // Phase 3: Evaluate constant expressions
    let mut phase_errors = evaluate::evaluate_expressions(&mut context, &name_map, &ast_items);
    errors.append(&mut phase_errors);

    // Phase 4: Type check values against their declared types
    let mut phase_errors = typecheck::typecheck_hir(&context, &order);
    errors.append(&mut phase_errors);

    // Phase 5: Validate the HIR
    let mut phase_errors = validate::validate_hir(&context, &order);
    errors.append(&mut phase_errors);

    LoweringResult {
        context,
        order,
        errors,
    }
}
