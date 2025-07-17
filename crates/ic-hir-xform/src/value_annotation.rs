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
use ic_hir::hir::{Ann, Def, DefId, DefKind, EnumLit, EnumTy, Numeric};
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
    fn fold_def(&mut self, mut def: Def) -> Def {
        // Only process enum definitions
        if let DefKind::Enum(ref mut enum_ty) = def.kind {
            // Process each enum field
            enum_ty.fields = enum_ty
                .fields
                .drain(..)
                .map(|mut field| {
                    // Look for @value annotation
                    let mut value_found = None;
                    let mut new_annotations = Vec::new();

                    for ann in field.annotations {
                        if ann.ident.name == self.value_ann_name {
                            // Extract the value from the annotation
                            if let Some(arg) = ann.args.first() {
                                if let Numeric::Int32(v) = &arg.value {
                                    value_found = Some(*v as isize);
                                } else if let Numeric::Int64(v) = &arg.value {
                                    // Try to convert to isize
                                    // Allow truncation - enum values are typically 32-bit anyway
                                    #[allow(clippy::cast_possible_truncation)]
                                    let isize_value = *v as isize;
                                    value_found = Some(isize_value);
                                }
                            }
                            // Don't add @value annotation to the new list
                        } else {
                            // Keep other annotations
                            new_annotations.push(ann);
                        }
                    }

                    // Update field if we found a value
                    if let Some(new_value) = value_found {
                        field.value = new_value;
                    }
                    field.annotations = new_annotations;
                    field
                })
                .collect();
        }
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
    use ic_hir::hir::{Ann, AnnArg, DefFlags, Ident, PrimitiveTy, Span, Ty, TyKind};

    use super::*;

    #[test]
    fn test_value_annotation_transform() {
        let def = Def {
            id: DefId::from(0),
            parent: None,
            ident: Ident {
                name: "TestEnum".to_string(),
                span: Span::default(),
            },
            kind: DefKind::Enum(EnumTy {
                fields: vec![
                    EnumLit {
                        ident: Ident {
                            name: "FOO".to_string(),
                            span: Span::default(),
                        },
                        value: 0,
                        annotations: vec![Ann {
                            ident: Ident {
                                name: "value".to_string(),
                                span: Span::default(),
                            },
                            def_id: DefId::from(1),
                            args: vec![AnnArg {
                                ident: None,
                                value: Numeric::Int32(42),
                            }],
                        }],
                    },
                    EnumLit {
                        ident: Ident {
                            name: "BAR".to_string(),
                            span: Span::default(),
                        },
                        value: 1,
                        annotations: vec![],
                    },
                ],
                ty: Ty {
                    span: Span::default(),
                    kind: TyKind::Primitive(PrimitiveTy::Int32),
                },
            }),
            flags: DefFlags::nil(),
            span: Span::default(),
            annotations: vec![],
        };

        let mut transformer = ValueAnnotationTransform::new();
        let result = transformer.fold_def(def);

        // Extract the enum from the result
        if let DefKind::Enum(enum_ty) = result.kind {
            // Check that FOO has value 42 and no annotations
            assert_eq!(enum_ty.fields[0].value, 42);
            assert!(enum_ty.fields[0].annotations.is_empty());

            // Check that BAR is unchanged
            assert_eq!(enum_ty.fields[1].value, 1);
            assert!(enum_ty.fields[1].annotations.is_empty());
        } else {
            panic!("Expected enum definition");
        }
    }
}
