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

//! Transforms `@value` annotations on enum fields into direct value assignments.
//!
//! This transformation:
//! 1. Finds enum fields with `@value` annotations
//! 2. Extracts the numeric value from the annotation
//! 3. Sets the enum field's value directly
//! 4. Removes the `@value` annotation

use ic_hir::fold::Fold;
use ic_hir::hir::{Def, DefId, DefKind};
use ic_hir::{Context, ResolvedGraph};

/// Transformer that converts @value annotations to direct enum values.
pub struct ValueAnnotationTransform {
    /// Name of the @value annotation definition
    value_ann_name: String,
}

impl ValueAnnotationTransform {
    /// Creates a new value annotation transformer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            value_ann_name: "value".to_string(),
        }
    }
}

impl Default for ValueAnnotationTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl Fold for ValueAnnotationTransform {
    fn fold_def(&mut self, def: Def) -> Def {
        // Since enums now use constants, we don't need to transform enum definitions
        // The @value annotations are already handled during lowering
        def
    }
}

/// Transforms all @value annotations in the HIR to direct enum values.
#[must_use]
pub fn transform(mut graph: ResolvedGraph) -> ResolvedGraph {
    let mut transformer = ValueAnnotationTransform::new();

    // Transform each definition in place
    for (id, def) in &mut graph.context.definitions {
        let original_def = std::mem::replace(
            def,
            Def {
                id,
                parent: None,
                ident: ic_hir::hir::Ident {
                    name: String::new(),
                    span: ic_hir::hir::Span::default(),
                },
                kind: DefKind::Decl(ic_hir::hir::Decl::Struct),
                flags: ic_hir::hir::DefFlags::nil(),
                span: ic_hir::hir::Span::default(),
                annotations: vec![],
            },
        );
        *def = transformer.fold_def(original_def);
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_annotation_transform() {
        // With the new enum structure using constants, the @value annotation
        // transformation happens during lowering, not as a separate transform.
        // This test now just verifies the transform is a no-op.
        let mut transformer = ValueAnnotationTransform::new();

        // Create a simple definition to test with
        let def = Def {
            id: DefId::from(0),
            parent: None,
            ident: ic_hir::hir::Ident {
                name: "TestType".to_string(),
                span: ic_hir::hir::Span::default(),
            },
            kind: DefKind::Decl(ic_hir::hir::Decl::Struct),
            flags: ic_hir::hir::DefFlags::nil(),
            span: ic_hir::hir::Span::default(),
            annotations: vec![],
        };

        let original_id = def.id;
        let original_name = def.ident.name.clone();

        let result = transformer.fold_def(def);

        // Should be unchanged
        assert_eq!(result.id, original_id);
        assert_eq!(result.ident.name, original_name);
    }
}
