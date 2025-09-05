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

//! Support for built-in types in HIR lowering.

use ic_syntax::Item;

use super::{LoweringResult, lower};
use crate::hir::DefFlags;

/// Lower AST with built-in definitions included in the output.
pub fn lower_with_builtins<I, J>(builtins: I, user: J) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
    J: IntoIterator<Item = Item>,
{
    let builtin_items: Vec<Item> = builtins.into_iter().collect();
    let user_items: Vec<Item> = user.into_iter().collect();

    // Process builtins first
    let mut result = lower(builtin_items);

    // Mark all builtin definitions with IS_BUILTIN flag
    for &def_id in &result.order {
        let def = result.context.definitions.get_mut(def_id);
        def.flags = def.flags.union(DefFlags::IS_BUILTIN);
    }

    // Save builtin order
    let builtin_order = result.order.clone();

    // Process user items in the same context
    let user_result = lower_in_context(user_items, result);

    // Combine builtin and user orders
    let mut final_order = builtin_order.clone();
    final_order.extend(&user_result.order);

    LoweringResult {
        context: user_result.context,
        order: final_order,
        builtin_order,
        errors: user_result.errors,
        warnings: user_result.warnings,
    }
}

/// Lower AST with built-ins available in context but not in output order.
pub fn lower_with_builtin_context<I, J>(builtins: I, user: J) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
    J: IntoIterator<Item = Item>,
{
    let builtin_items: Vec<Item> = builtins.into_iter().collect();
    let user_items: Vec<Item> = user.into_iter().collect();

    // Process builtins first
    let mut result = lower(builtin_items);

    // Mark all builtin definitions with IS_BUILTIN flag
    for &def_id in &result.order {
        let def = result.context.definitions.get_mut(def_id);
        def.flags = def.flags.union(DefFlags::IS_BUILTIN);
    }

    // Save builtin order separately
    let builtin_order = result.order.clone();

    // Process user items in the same context
    let user_result = lower_in_context(user_items, result);

    // Only include user items in the output order
    LoweringResult {
        context: user_result.context,
        order: user_result.order, // Only user items
        builtin_order,
        errors: user_result.errors,
        warnings: user_result.warnings,
    }
}

/// Lower AST items in an existing context.
fn lower_in_context<I>(ast: I, result: LoweringResult) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
{
    let ast_items: Vec<Item> = ast.into_iter().collect();

    // Create a new lowering context from the existing result
    let mut ctx =
        super::LoweringContext::from_existing(result.context, result.errors, result.warnings);

    // Build HIR from AST items
    let mut builder = super::builder::HirBuilder::new(&mut ctx);
    builder.build(&ast_items);

    // Phase 2: Update forward references
    super::update_forward_references(&mut ctx);

    // Phase 3: Validation
    let mut validator = super::validator::Validator::new(&ctx);
    validator.validate();

    // Return the updated result
    LoweringResult {
        context: ctx.context,
        order: ctx.order,
        builtin_order: result.builtin_order,
        errors: ctx.diagnostics.errors,
        warnings: ctx.diagnostics.warnings,
    }
}
