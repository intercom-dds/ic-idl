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

//! Built-in type injection for HIR lowering.

use ic_syntax::Item;

use super::LoweringResult;

/// Lowers AST items to HIR with built-in types pre-injected.
///
/// This function first processes built-in definitions (like annotations)
/// and then processes the user's AST in the same context, ensuring that
/// built-in types are available for resolution.
///
/// The returned `LoweringResult` includes both built-in and user-defined types
/// in the `order` vector so they are available for ptree lowering.
pub fn lower_with_builtins<I, B>(builtins: B, user_ast: I) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
    B: IntoIterator<Item = Item>,
{
    let builtin_items: Vec<Item> = builtins.into_iter().collect();
    let user_items: Vec<Item> = user_ast.into_iter().collect();

    // First, lower built-ins alone to get their DefIds
    let builtin_result = super::lower(builtin_items.clone());
    let builtin_def_ids: std::collections::HashSet<_> = builtin_result.order.into_iter().collect();

    // Combine built-ins and user AST
    let mut all_items = builtin_items;
    all_items.extend(user_items);

    // Process everything together
    let mut result = super::lower(all_items);

    // Mark all builtin definitions with IS_BUILTIN flag
    for &def_id in &builtin_def_ids {
        result.context.definitions[def_id].flags |= crate::hir::DefFlags::IS_BUILTIN;
    }

    // Store builtin order separately
    result.builtin_order = builtin_def_ids.into_iter().collect();

    result
}

/// Lowers user AST with built-in definitions available for resolution,
/// but only includes user definitions in the output order.
///
/// This avoids duplicate built-in definitions when merging multiple HIRs.
pub fn lower_with_builtin_context<I, B>(builtins: B, user_ast: I) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
    B: IntoIterator<Item = Item>,
{
    let builtin_items: Vec<Item> = builtins.into_iter().collect();
    let user_items: Vec<Item> = user_ast.into_iter().collect();

    // First, lower built-ins alone to get their DefIds
    let builtin_result = super::lower(builtin_items.clone());
    let builtin_def_ids: std::collections::HashSet<_> = builtin_result.order.into_iter().collect();

    // Combine built-ins and user AST
    let mut all_items = builtin_items;
    all_items.extend(user_items);

    // Process everything together
    let mut result = super::lower(all_items);

    // Mark all builtin definitions with IS_BUILTIN flag
    for &def_id in &builtin_def_ids {
        result.context.definitions[def_id].flags |= crate::hir::DefFlags::IS_BUILTIN;
    }

    // Store builtin order separately
    result.builtin_order = builtin_def_ids.clone().into_iter().collect();

    // Filter the order vector to only include user definitions
    // Built-in definitions are still in the context but not in the output order
    result
        .order
        .retain(|def_id| !builtin_def_ids.contains(def_id));

    result
}
