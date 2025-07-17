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

//! Transforms `@position` annotations on bitmask flags into direct bit position values.
//!
//! This transformation:
//! 1. Finds bitmask flags with `@position` annotations
//! 2. Extracts the numeric position from the annotation
//! 3. Sets the bitmask flag's value to 1 << position
//! 4. Removes the `@position` annotation

use ic_hir::fold::Fold;
use ic_hir::hir::{Ann, BitFlag, BitmaskTy, Def, DefId, DefKind, Numeric};
use ic_hir::{Context, ResolvedGraph};

/// Transformer that converts @position annotations to direct bitmask values.
pub struct PositionAnnotationTransform {
    /// Name of the @position annotation definition
    position_ann_name: String,
}

impl PositionAnnotationTransform {
    /// Creates a new position annotation transformer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            position_ann_name: "position".to_string(),
        }
    }
}

impl Default for PositionAnnotationTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl Fold for PositionAnnotationTransform {
    fn fold_def(&mut self, mut def: Def) -> Def {
        // Only process bitmask definitions
        if let DefKind::Bitmask(ref mut bitmask_ty) = def.kind {
            // Process each bitmask flag
            bitmask_ty.flags = bitmask_ty
                .flags
                .drain(..)
                .map(|mut flag| {
                    // Look for @position annotation
                    let mut position_found = None;
                    let mut new_annotations = Vec::new();

                    for ann in flag.annotations {
                        if ann.ident.name == self.position_ann_name {
                            // Extract the position from the annotation
                            if let Some(arg) = ann.args.first() {
                                if let Numeric::Int32(v) = &arg.value {
                                    position_found = Some(*v as usize);
                                } else if let Numeric::Int64(v) = &arg.value {
                                    // Try to convert to usize
                                    // Allow truncation - bit positions are typically small
                                    #[allow(clippy::cast_possible_truncation)]
                                    let usize_value = *v as usize;
                                    position_found = Some(usize_value);
                                }
                            }
                            // Don't add @position annotation to the new list
                        } else {
                            // Keep other annotations
                            new_annotations.push(ann);
                        }
                    }

                    // Update flag value if we found a position
                    if let Some(position) = position_found {
                        // Convert position to bit value (1 << position)
                        flag.value = 1 << position;
                    }
                    flag.annotations = new_annotations;
                    flag
                })
                .collect();
        }
        def
    }
}

/// Transforms all @position annotations in the HIR to direct bitmask values.
#[must_use]
pub fn transform(mut graph: ResolvedGraph) -> ResolvedGraph {
    let mut transformer = PositionAnnotationTransform::new();

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
    use ic_hir::hir::{Ann, AnnArg, DefFlags, Ident, PrimitiveTy, Span, Ty, TyKind};

    use super::*;

    #[test]
    fn test_position_annotation_transform() {
        let def = Def {
            id: DefId::from(0),
            parent: None,
            ident: Ident {
                name: "TestFlags".to_string(),
                span: Span::default(),
            },
            kind: DefKind::Bitmask(BitmaskTy {
                flags: vec![
                    BitFlag {
                        ident: Ident {
                            name: "FLAG_A".to_string(),
                            span: Span::default(),
                        },
                        value: 1,
                        annotations: vec![],
                    },
                    BitFlag {
                        ident: Ident {
                            name: "FLAG_B".to_string(),
                            span: Span::default(),
                        },
                        value: 2,
                        annotations: vec![Ann {
                            ident: Ident {
                                name: "position".to_string(),
                                span: Span::default(),
                            },
                            def_id: DefId::from(1),
                            args: vec![AnnArg {
                                ident: None,
                                value: Numeric::Int32(5),
                            }],
                        }],
                    },
                ],
                ty: Ty {
                    span: Span::default(),
                    kind: TyKind::Primitive(PrimitiveTy::UInt32),
                },
            }),
            flags: DefFlags::nil(),
            span: Span::default(),
            annotations: vec![],
        };

        let mut transformer = PositionAnnotationTransform::new();
        let result = transformer.fold_def(def);

        // Extract the bitmask from the result
        if let DefKind::Bitmask(bitmask_ty) = result.kind {
            // Check that FLAG_A is unchanged
            assert_eq!(bitmask_ty.flags[0].value, 1);
            assert!(bitmask_ty.flags[0].annotations.is_empty());

            // Check that FLAG_B has value 1 << 5 = 32 and no annotations
            assert_eq!(bitmask_ty.flags[1].value, 32);
            assert!(bitmask_ty.flags[1].annotations.is_empty());
        } else {
            panic!("Expected bitmask definition");
        }
    }
}
