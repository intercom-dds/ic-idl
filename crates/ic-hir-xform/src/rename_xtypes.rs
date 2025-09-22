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

//! Rename `DDS::XTypes` module to `DDS::xtypes`
//!
//! This transformation renames the `DDS::XTypes` module (if present) to `DDS::xtypes`
//! to match Rust naming conventions.

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind};

/// Transform HIR to rename `DDS::XTypes` module to `DDS::xtypes`
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    // Use the new helper to lookup DDS::XTypes
    let xtypes_id = hir.context.lookup_symbol("DDS::XTypes");

    if let Some(xtypes_id) = xtypes_id {
        // Verify it's actually a module
        let def = hir.context.definitions.get(xtypes_id);
        if matches!(&def.kind, DefKind::Module(_)) {
            // Get parent (DDS module) ID
            let parent_id = def.parent;

            // Rename XTypes to xtypes
            let xtypes_def = hir.context.definitions.get_mut(xtypes_id);
            xtypes_def.ident.name = "xtypes".into();

            // Update scope definitions
            // First, update the XTypes module's own scope if it exists
            if let Some(scope) = hir
                .context
                .scopes
                .scopes
                .iter_mut()
                .find(|s| s.def_id == Some(xtypes_id))
            {
                // The scope's definition list might have self-reference, update it
                if scope.definitions.remove("XTypes").is_some() {
                    scope.definitions.insert("xtypes", xtypes_id);
                }
            }

            // Update parent (DDS) scope's definition list
            if let Some(dds_id) = parent_id {
                if let Some(dds_scope) = hir
                    .context
                    .scopes
                    .scopes
                    .iter_mut()
                    .find(|s| s.def_id == Some(dds_id))
                {
                    // Update the name in the parent scope's definitions
                    if dds_scope.definitions.remove("XTypes").is_some() {
                        dds_scope.definitions.insert("xtypes", xtypes_id);
                    }
                }
            }
        }
    }

    hir
}
