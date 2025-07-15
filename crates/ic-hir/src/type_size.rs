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

use crate::Context;
use crate::hir::{PrimitiveTy, Ty, TyKind};

/// Calculate the size in bytes of a type.
/// Returns None for dynamically-sized types or types with unknown size.
pub fn type_size(ty: &Ty, ctx: &Context) -> Option<usize> {
    match &ty.kind {
        TyKind::Any => None,      // Unknown size
        TyKind::Fixed => Some(8), // Fixed point, assume 64-bit
        TyKind::Primitive(prim) => primitive_size(prim),
        TyKind::Array { ty: elem_ty, len } => {
            // Array size = element size * count
            type_size(elem_ty, ctx).map(|elem_size| elem_size * len)
        }
        TyKind::Sequence { .. } => None, // Dynamic size
        TyKind::String { .. } => None,   // Dynamic size
        TyKind::Map { .. } => None,      // Dynamic size
        TyKind::Adt(id) => {
            // Look up the definition and get its size
            let def = ctx.definitions.get(*id);
            match &def.kind {
                crate::hir::DefKind::Struct(struct_ty) => {
                    // Struct size = sum of member sizes (ignoring padding for now)
                    let mut total = 0;
                    for member in &struct_ty.members {
                        total += type_size(&member.ty, ctx)?;
                    }
                    Some(total)
                }
                crate::hir::DefKind::Union(union_ty) => {
                    // Union size = max of variant sizes + discriminator
                    let disc_size = type_size(&union_ty.disc, ctx)?;
                    let mut max_variant_size = 0;
                    for variant in &union_ty.variants {
                        if let Some(size) = type_size(&variant.ty, ctx) {
                            max_variant_size = max_variant_size.max(size);
                        }
                    }
                    Some(disc_size + max_variant_size)
                }
                crate::hir::DefKind::Enum(enum_ty) => {
                    // Enum size = underlying type size
                    type_size(&enum_ty.ty, ctx)
                }
                crate::hir::DefKind::Bitmask(bitmask_ty) => {
                    // Bitmask size = underlying type size
                    type_size(&bitmask_ty.ty, ctx)
                }
                crate::hir::DefKind::Bitset(bitset_ty) => {
                    // Bitset size = sum of field sizes (bits) / 8 (rounded up)
                    let mut total_bits = 0;
                    for field in &bitset_ty.fields {
                        total_bits += field.size;
                    }
                    Some((total_bits + 7) / 8)
                }
                crate::hir::DefKind::Alias(alias_ty) => {
                    // Alias size = aliased type size
                    type_size(&alias_ty.ty, ctx)
                }
                _ => None, // Other types don't have a fixed size
            }
        }
    }
}

/// Calculate the size of a primitive type in bytes
fn primitive_size(prim: &PrimitiveTy) -> Option<usize> {
    match prim {
        PrimitiveTy::Void => None, // No size
        PrimitiveTy::Bool => Some(1),
        PrimitiveTy::Char => Some(1),
        PrimitiveTy::WChar => Some(4), // Assuming UTF-32
        PrimitiveTy::Int8 => Some(1),
        PrimitiveTy::UInt8 => Some(1),
        PrimitiveTy::Int16 => Some(2),
        PrimitiveTy::UInt16 => Some(2),
        PrimitiveTy::Int32 => Some(4),
        PrimitiveTy::UInt32 => Some(4),
        PrimitiveTy::Int64 => Some(8),
        PrimitiveTy::UInt64 => Some(8),
        PrimitiveTy::Float32 => Some(4),
        PrimitiveTy::Float64 => Some(8),
        PrimitiveTy::Float128 => Some(16), // Platform-specific, but often 16
    }
}

#[cfg(test)]
mod tests {
    use ic_syntax::{Ident, Span};

    use super::*;
    use crate::hir::{
        AliasTy, BitmaskTy, BitsetField, BitsetTy, Def, DefKind, EnumTy, Member, StructTy, UnionTy,
        Variant,
    };

    fn make_primitive_type(prim: PrimitiveTy) -> Ty {
        Ty {
            span: Span::default(),
            kind: TyKind::Primitive(prim),
        }
    }

    fn make_array_type(elem_ty: Ty, len: usize) -> Ty {
        Ty {
            span: Span::default(),
            kind: TyKind::Array {
                ty: Box::new(elem_ty),
                len,
            },
        }
    }

    fn make_string_type(wide: bool, bound: Option<usize>) -> Ty {
        Ty {
            span: Span::default(),
            kind: TyKind::String { wide, bound },
        }
    }

    fn make_sequence_type(elem_ty: Ty, bound: Option<usize>) -> Ty {
        Ty {
            span: Span::default(),
            kind: TyKind::Sequence {
                ty: Box::new(elem_ty),
                bound,
            },
        }
    }

    #[test]
    fn test_primitive_sizes() {
        let ctx = Context::new();

        // 1-byte types
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Bool), &ctx),
            Some(1)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Char), &ctx),
            Some(1)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Int8), &ctx),
            Some(1)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::UInt8), &ctx),
            Some(1)
        );

        // 2-byte types
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Int16), &ctx),
            Some(2)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::UInt16), &ctx),
            Some(2)
        );

        // 4-byte types
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::WChar), &ctx),
            Some(4)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Int32), &ctx),
            Some(4)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::UInt32), &ctx),
            Some(4)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Float32), &ctx),
            Some(4)
        );

        // 8-byte types
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Int64), &ctx),
            Some(8)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::UInt64), &ctx),
            Some(8)
        );
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Float64), &ctx),
            Some(8)
        );

        // 16-byte types
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Float128), &ctx),
            Some(16)
        );

        // No size
        assert_eq!(
            type_size(&make_primitive_type(PrimitiveTy::Void), &ctx),
            None
        );
    }

    #[test]
    fn test_array_sizes() {
        let ctx = Context::new();

        // Array of 10 int32s = 10 * 4 = 40 bytes
        let array_type = make_array_type(make_primitive_type(PrimitiveTy::Int32), 10);
        assert_eq!(type_size(&array_type, &ctx), Some(40));

        // Array of 100 bytes = 100 * 1 = 100 bytes
        let array_type = make_array_type(make_primitive_type(PrimitiveTy::Int8), 100);
        assert_eq!(type_size(&array_type, &ctx), Some(100));

        // Array of 50 doubles = 50 * 8 = 400 bytes
        let array_type = make_array_type(make_primitive_type(PrimitiveTy::Float64), 50);
        assert_eq!(type_size(&array_type, &ctx), Some(400));

        // Multi-dimensional array: int32[5][10] = 5 * 10 * 4 = 200 bytes
        let inner_array = make_array_type(make_primitive_type(PrimitiveTy::Int32), 10);
        let outer_array = make_array_type(inner_array, 5);
        assert_eq!(type_size(&outer_array, &ctx), Some(200));
    }

    #[test]
    fn test_dynamic_types() {
        let ctx = Context::new();

        // Unbounded string has no fixed size
        let string_type = make_string_type(false, None);
        assert_eq!(type_size(&string_type, &ctx), None);

        // Bounded string still has no fixed size (it's a max, not actual)
        let string_type = make_string_type(false, Some(100));
        assert_eq!(type_size(&string_type, &ctx), None);

        // Unbounded sequence has no fixed size
        let seq_type = make_sequence_type(make_primitive_type(PrimitiveTy::Int32), None);
        assert_eq!(type_size(&seq_type, &ctx), None);

        // Bounded sequence still has no fixed size
        let seq_type = make_sequence_type(make_primitive_type(PrimitiveTy::Int32), Some(100));
        assert_eq!(type_size(&seq_type, &ctx), None);
    }

    #[test]
    fn test_special_types() {
        let ctx = Context::new();

        // Any type has unknown size
        let any_type = Ty {
            span: Span::default(),
            kind: TyKind::Any,
        };
        assert_eq!(type_size(&any_type, &ctx), None);

        // Fixed type assumed to be 64-bit
        let fixed_type = Ty {
            span: Span::default(),
            kind: TyKind::Fixed,
        };
        assert_eq!(type_size(&fixed_type, &ctx), Some(8));
    }

    #[test]
    fn test_struct_size() {
        let mut ctx = Context::new();

        // Create a struct with:
        // - int32 field (4 bytes)
        // - int64 field (8 bytes)
        // - bool field (1 byte)
        // Total = 13 bytes (ignoring padding)
        let struct_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "TestStruct".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Struct(StructTy {
                parent: None,
                members: vec![
                    Member {
                        ident: Ident {
                            name: "field1".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Int32),
                        annotations: vec![],
                    },
                    Member {
                        ident: Ident {
                            name: "field2".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Int64),
                        annotations: vec![],
                    },
                    Member {
                        ident: Ident {
                            name: "field3".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Bool),
                        annotations: vec![],
                    },
                ],
            }),
        });

        let struct_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(struct_id),
        };

        assert_eq!(type_size(&struct_type, &ctx), Some(13));
    }

    #[test]
    fn test_alias_size() {
        let mut ctx = Context::new();

        // Create an alias to int32[100]
        let alias_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "IntArray".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Alias(AliasTy {
                ty: make_array_type(make_primitive_type(PrimitiveTy::Int32), 100),
            }),
        });

        let alias_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(alias_id),
        };

        assert_eq!(type_size(&alias_type, &ctx), Some(400)); // 100 * 4
    }

    #[test]
    fn test_union_size() {
        let mut ctx = Context::new();

        // Create a union with:
        // - int32 discriminator (4 bytes)
        // - variant1: int8 (1 byte)
        // - variant2: int64 (8 bytes)
        // - variant3: float64 (8 bytes)
        // Total = 4 + max(1, 8, 8) = 12 bytes
        let union_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "TestUnion".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Union(UnionTy {
                disc: make_primitive_type(PrimitiveTy::Int32),
                variants: vec![
                    Variant {
                        annotations: vec![],
                        ident: Ident {
                            name: "variant1".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Int8),
                        labels: vec![],
                        is_default: false,
                    },
                    Variant {
                        annotations: vec![],
                        ident: Ident {
                            name: "variant2".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Int64),
                        labels: vec![],
                        is_default: false,
                    },
                    Variant {
                        annotations: vec![],
                        ident: Ident {
                            name: "variant3".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Float64),
                        labels: vec![],
                        is_default: false,
                    },
                ],
            }),
        });

        let union_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(union_id),
        };

        assert_eq!(type_size(&union_type, &ctx), Some(12));
    }

    #[test]
    fn test_enum_size() {
        let mut ctx = Context::new();

        // Create an enum with int32 underlying type
        let enum_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "TestEnum".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Enum(EnumTy {
                ty: make_primitive_type(PrimitiveTy::Int32),
                fields: vec![],
            }),
        });

        let enum_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(enum_id),
        };

        assert_eq!(type_size(&enum_type, &ctx), Some(4));
    }

    #[test]
    fn test_bitmask_size() {
        let mut ctx = Context::new();

        // Create a bitmask with uint16 underlying type
        let bitmask_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "TestBitmask".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Bitmask(BitmaskTy {
                ty: make_primitive_type(PrimitiveTy::UInt16),
                flags: vec![],
            }),
        });

        let bitmask_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(bitmask_id),
        };

        assert_eq!(type_size(&bitmask_type, &ctx), Some(2));
    }

    #[test]
    fn test_bitset_size() {
        let mut ctx = Context::new();

        // Create a bitset with 17 bits (should be 3 bytes)
        let bitset_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "TestBitset".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Bitset(BitsetTy {
                parent: None,
                fields: vec![
                    BitsetField {
                        ident: Ident {
                            name: "field1".to_string(),
                            span: Span::default(),
                        },
                        size: 8,
                        ty: make_primitive_type(PrimitiveTy::UInt8),
                        annotations: vec![],
                    },
                    BitsetField {
                        ident: Ident {
                            name: "field2".to_string(),
                            span: Span::default(),
                        },
                        size: 5,
                        ty: make_primitive_type(PrimitiveTy::UInt8),
                        annotations: vec![],
                    },
                    BitsetField {
                        ident: Ident {
                            name: "field3".to_string(),
                            span: Span::default(),
                        },
                        size: 4,
                        ty: make_primitive_type(PrimitiveTy::UInt8),
                        annotations: vec![],
                    },
                ],
            }),
        });

        let bitset_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(bitset_id),
        };

        // 17 bits = 3 bytes (rounded up)
        assert_eq!(type_size(&bitset_type, &ctx), Some(3));
    }

    #[test]
    fn test_nested_struct_size() {
        let mut ctx = Context::new();

        // Create an inner struct
        let inner_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "InnerStruct".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Struct(StructTy {
                parent: None,
                members: vec![
                    Member {
                        ident: Ident {
                            name: "x".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Int32),
                        annotations: vec![],
                    },
                    Member {
                        ident: Ident {
                            name: "y".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Int32),
                        annotations: vec![],
                    },
                ],
            }),
        });

        // Create outer struct containing the inner struct
        let outer_id = ctx.definitions.alloc_with_id(|id| Def {
            id,
            parent: None,
            ident: Ident {
                name: "OuterStruct".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
            flags: Default::default(),
            annotations: vec![],
            kind: DefKind::Struct(StructTy {
                parent: None,
                members: vec![
                    Member {
                        ident: Ident {
                            name: "flag".to_string(),
                            span: Span::default(),
                        },
                        ty: make_primitive_type(PrimitiveTy::Bool),
                        annotations: vec![],
                    },
                    Member {
                        ident: Ident {
                            name: "inner".to_string(),
                            span: Span::default(),
                        },
                        ty: Ty {
                            span: Span::default(),
                            kind: TyKind::Adt(inner_id),
                        },
                        annotations: vec![],
                    },
                ],
            }),
        });

        let outer_type = Ty {
            span: Span::default(),
            kind: TyKind::Adt(outer_id),
        };

        // 1 (bool) + 8 (inner struct: 4 + 4) = 9 bytes
        assert_eq!(type_size(&outer_type, &ctx), Some(9));
    }
}

