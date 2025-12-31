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

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

//! Transforms `@position` annotations on bitmask flags into direct bit position values.
//!
//! This transformation:
//! 1. Finds bitmask flag constants with `@position` annotations
//! 2. Extracts the numeric position from the annotation
//! 3. Sets the flag constant's value to 1 << position
//! 4. Removes the `@position` annotation

use ic_hir::ResolvedGraph;
use ic_hir::hir::{Ann, ConstTy, Def, DefKind, Numeric};
use tracing::{debug, debug_span};

/// Process a single definition, looking for bitmask flag constants with @position annotation
fn process_def(def: &mut Def) {
    // Check if this is a constant definition
    if let DefKind::Const(ref mut const_ty) = def.kind {
        // Look for @position annotation
        let mut position_found = None;
        let mut new_annotations = Vec::new();

        for ann in def.annotations.drain(..) {
            if ann.ident.name == "position" {
                // Extract the position from the annotation
                if let Some(arg) = ann.args.first() {
                    match &arg.value {
                        Numeric::Int32(v) => position_found = Some(*v as u32),
                        Numeric::Int64(v) => position_found = Some(*v as u32),
                        Numeric::UInt16(v) => position_found = Some(u32::from(*v)),
                        Numeric::UInt32(v) => position_found = Some(*v),
                        Numeric::UInt64(v) => position_found = Some(*v as u32),
                        _ => {}
                    }
                }
                // Don't keep the @position annotation
            } else {
                // Keep other annotations
                new_annotations.push(ann);
            }
        }

        def.annotations = new_annotations;

        // Update the constant value if we found a position
        if let Some(position) = position_found {
            const_ty.value = Numeric::UInt64(1u64 << position);
        }
    }
}

/// Transforms all @position annotations in the HIR to direct bitmask values.
#[must_use]
pub fn transform(mut graph: ResolvedGraph) -> ResolvedGraph {
    let _span = debug_span!("xform", name = "position_annotation").entered();
    debug!("applying transform");
    // Find all bitmasks and their flag constants
    let bitmask_flags: Vec<_> = graph
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if let DefKind::Bitmask(bitmask) = &def.kind {
                Some(bitmask.flags.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect();

    // Process each bitmask flag constant
    for flag_id in bitmask_flags {
        let def = graph.context.definitions.get_mut(flag_id);
        process_def(def);
    }

    graph
}
