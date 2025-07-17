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
/// The returned LoweringResult will only include user-defined types in the
/// `order` vector, while built-in types remain in the context for resolution.
pub fn lower_with_builtins<I, B>(builtins: B, user_ast: I) -> LoweringResult
where
    I: IntoIterator<Item = Item>,
    B: IntoIterator<Item = Item>,
{
    // Combine built-ins and user AST
    let mut all_items: Vec<Item> = builtins.into_iter().collect();
    all_items.extend(user_ast);
    
    // Process everything together
    let mut result = super::lower(all_items);
    
    // Filter the order to exclude built-in definitions
    // Built-in annotations are in the intercom module
    result.order.retain(|&def_id| {
        let def = result.context.definitions.get(def_id);
        // Keep only non-intercom definitions (user code)
        !def.ident.name.starts_with("intercom")
    });
    
    result
}