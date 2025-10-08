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

//! Synthesize explicit type aliases for array types.
//!
//! Some target languages (like Ada) require fixed-size arrays to be declared
//! as explicit type aliases before they can be used as member types,
//! parameters, or return values. This transformation extracts all inline array
//! types like `long a[10]` and converts them into typedef-style aliases.
//!
//! # Transformation
//!
//! Input IDL:
//! ```idl
//! struct Example {
//!     long data[10];
//!     short values[10];  // Same length, different type
//! };
//! ```
//!
//! Output HIR (conceptually):
//! ```idl
//! typedef long Long_10_Array[10];
//! typedef short Short_10_Array[10];
//!
//! struct Example {
//!     Long_10_Array data;
//!     Short_10_Array values;
//! };
//! ```
//!
//! # Deduplication
//!
//! Array types with identical element types and lengths are deduplicated.
//! The transformation creates a unique `ArrayKey` for each distinct array type
//! that considers:
//! - Element type (primitive, ADT, string, sequence, map, etc.)
//! - Array length
//! - For complex types: their inner structure (e.g., sequence bounds, map key/value types)
//!
//! # Scoping
//!
//! Synthesized array aliases are placed in the same scope as the first usage:
//! - Top-level arrays → synthesized alias added to `hir.order`
//! - Module-nested arrays → synthesized alias added to module's `definitions`
//! - Interface-nested arrays → synthesized alias added to interface's `definitions`
//!
//! # Coverage
//!
//! The transformation handles arrays in:
//! - Struct members
//! - Union variants
//! - Exception members
//! - Valuetype members
//! - Interface operation parameters and return types
//! - Interface attributes
//! - Type aliases (typedefs)

use std::collections::HashMap;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    self, AliasTy, DefId, DefKind, Ident, InterfaceTy, ModuleTy, PrimitiveTy, Span, Ty, TyKind,
    ValueTy,
};

/// Key for deduplicating array types.
///
/// Two arrays are considered identical if their element type and length match.
/// For complex element types (sequences, maps, etc.), the key recursively
/// captures the full type structure to prevent collisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ArrayKey {
    Primitive {
        name: String,
        len: usize,
    },
    Adt {
        def_id: DefId,
        len: usize,
    },
    String {
        wide: bool,
        bound: Option<usize>,
        len: usize,
    },
    Sequence {
        elem: Box<ArrayKey>,
        bound: Option<usize>,
        len: usize,
    },
    Array {
        elem: Box<ArrayKey>,
        elem_len: usize,
        len: usize,
    },
    Map {
        key: Box<ArrayKey>,
        value: Box<ArrayKey>,
        bound: Option<usize>,
        len: usize,
    },
    Other {
        name: String,
        len: usize,
    },
}

/// Represents an array type that needs to be synthesized into a type alias.
struct ArrayReplacement {
    /// The element type of the array
    elem_ty: Box<Ty>,

    /// The length of the array
    len: usize,

    /// Source span for the array type
    span: Span,

    /// Parent definition containing this array (None for top-level)
    parent: Option<DefId>,

    /// The definition that uses this array type
    used_by: DefId,

    /// Order in which this array was encountered (for stable ordering)
    encounter_order: usize,
}

fn insert_top_level_array(hir: &mut ResolvedGraph, array_def_id: DefId, used_by_id: DefId) {
    if let Some(pos) = hir.order.iter().position(|&id| id == used_by_id) {
        hir.order.insert(pos, array_def_id);
    } else {
        hir.order.insert(0, array_def_id);
    }
}

fn insert_nested_array(
    hir: &mut ResolvedGraph,
    array_def_id: DefId,
    parent_id: DefId,
    used_by_id: DefId,
) {
    let parent_def = hir.context.definitions.get(parent_id);
    if !matches!(
        parent_def.kind,
        DefKind::Module(_) | DefKind::Interface(_) | DefKind::Valuetype(_)
    ) {
        return;
    }

    hir.context.definitions.fold(parent_id, |mut parent_def| {
        let insert_pos = match &parent_def.kind {
            DefKind::Module(module_ty) => module_ty
                .definitions
                .iter()
                .position(|&id| id == used_by_id),
            DefKind::Interface(interface_ty) => interface_ty
                .definitions
                .iter()
                .position(|&id| id == used_by_id),
            DefKind::Valuetype(value_ty) => {
                value_ty.definitions.iter().position(|&id| id == used_by_id)
            }
            _ => None,
        };

        if let Some(pos) = insert_pos {
            match &mut parent_def.kind {
                DefKind::Module(module_ty) => {
                    module_ty.definitions.insert(pos, array_def_id);
                }
                DefKind::Interface(interface_ty) => {
                    interface_ty.definitions.insert(pos, array_def_id);
                }
                DefKind::Valuetype(value_ty) => {
                    value_ty.definitions.insert(pos, array_def_id);
                }
                _ => {}
            }
        }
        parent_def
    });
}

/// Transform array types into explicit type alias definitions.
///
/// This is a three-pass transformation:
/// 1. Collect all array types used anywhere in the HIR
/// 2. Create synthetic type alias definitions for each unique array type
/// 3. Replace inline array types with references to the synthesized aliases
///
/// Array aliases are deduplicated: if `long[10]` appears multiple times, only
/// one `Long_10_Array` typedef is created.
#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    // First pass: collect all array types that need to be synthesized
    let mut arrays_to_synthesize: Vec<ArrayReplacement> = Vec::new();
    let mut encounter_order = 0;

    let def_ids: Vec<DefId> = hir.order.clone();
    for &def_id in &def_ids {
        collect_arrays(
            &hir,
            def_id,
            &mut arrays_to_synthesize,
            &mut encounter_order,
        );
    }

    // Second pass: create synthetic type aliases and track where to insert them
    let mut array_types: HashMap<ArrayKey, DefId> = HashMap::new();
    let mut arrays_to_insert: Vec<(DefId, Option<DefId>, DefId, usize)> = Vec::new();

    for replacement in arrays_to_synthesize {
        let elem_name = type_name(&hir, &replacement.elem_ty);
        let key = make_array_key(&replacement.elem_ty, replacement.len);

        if let std::collections::hash_map::Entry::Vacant(e) = array_types.entry(key) {
            let type_name = format!("{}_{}_Array", elem_name, replacement.len);

            let array_def_id = hir.context.definitions.alloc_with_id(|id| hir::Def {
                id,
                ident: Ident {
                    name: type_name,
                    span: replacement.span,
                },
                parent: replacement.parent,
                annotations: Vec::new(),
                span: replacement.span,
                kind: DefKind::Alias(AliasTy {
                    ty: Ty {
                        span: replacement.span,
                        kind: TyKind::Array {
                            ty: replacement.elem_ty.clone(),
                            len: replacement.len,
                            len_span: replacement.span,
                        },
                    },
                }),
                flags: hir::DefFlags::nil(),
            });

            arrays_to_insert.push((
                array_def_id,
                replacement.parent,
                replacement.used_by,
                replacement.encounter_order,
            ));
            e.insert(array_def_id);
        }
    }

    // Sort arrays by encounter order (stable insertion order)
    arrays_to_insert.sort_by_key(|(_, _, _, enc)| *enc);

    // Insert arrays in encounter order, each right before its used_by definition
    for (array_def_id, parent_id, used_by_id, _) in arrays_to_insert {
        match parent_id {
            None => insert_top_level_array(&mut hir, array_def_id, used_by_id),
            Some(parent_id) => insert_nested_array(&mut hir, array_def_id, parent_id, used_by_id),
        }
    }

    // Third pass: replace array types with references to the aliases
    let synthesized_ids: std::collections::HashSet<DefId> = array_types.values().copied().collect();

    for &def_id in &def_ids {
        replace_arrays_recursive(&mut hir, def_id, &array_types, &synthesized_ids);
    }

    hir
}

/// Recursively replace inline array types with references to synthesized
/// aliases.
///
/// This walks the definition tree and updates any `TyKind::Array` to
/// `TyKind::Adt` pointing to the corresponding synthesized alias. Skips
/// synthesized definitions themselves to avoid replacing the array in the
/// typedef.
fn replace_arrays_recursive(
    hir: &mut ResolvedGraph,
    def_id: DefId,
    array_types: &HashMap<ArrayKey, DefId>,
    synthesized_ids: &std::collections::HashSet<DefId>,
) {
    let child_ids = {
        let def = hir.context.definitions.get(def_id);
        match &def.kind {
            DefKind::Module(module_ty) => module_ty.definitions.clone(),
            DefKind::Interface(interface_ty) => interface_ty.definitions.clone(),
            DefKind::Valuetype(value_ty) => value_ty.definitions.clone(),
            _ => Vec::new(),
        }
    };

    for child_id in child_ids {
        replace_arrays_recursive(hir, child_id, array_types, synthesized_ids);
    }

    if synthesized_ids.contains(&def_id) {
        return;
    }

    hir.context.definitions.fold(def_id, |mut def| {
        match &mut def.kind {
            DefKind::Struct(struct_ty) => {
                for member in &mut struct_ty.members {
                    replace_type_if_array(&mut member.ty, array_types);
                }
            }
            DefKind::Union(union_ty) => {
                replace_type_if_array(&mut union_ty.disc.ty, array_types);
                for variant in &mut union_ty.variants {
                    replace_type_if_array(&mut variant.ty, array_types);
                }
            }
            DefKind::Except(except_ty) => {
                for member in &mut except_ty.members {
                    replace_type_if_array(&mut member.ty, array_types);
                }
            }
            DefKind::Valuetype(value_ty) => {
                for member in &mut value_ty.members {
                    replace_type_if_array(&mut member.ty, array_types);
                }
            }
            DefKind::Alias(alias_ty) => {
                replace_type_if_array(&mut alias_ty.ty, array_types);
            }
            DefKind::Interface(interface_ty) => {
                for proto in &mut interface_ty.prototypes {
                    replace_type_if_array(&mut proto.ty, array_types);
                    for param in &mut proto.params {
                        replace_type_if_array(&mut param.ty, array_types);
                    }
                }
                for attr in &mut interface_ty.attributes {
                    replace_type_if_array(&mut attr.ty, array_types);
                }
            }
            _ => {}
        }
        def
    });
}

/// Collect all inline array types that need to be synthesized.
///
/// Recursively walks definitions and finds any `TyKind::Array` used in:
/// - Struct/union/exception/valuetype members
/// - Interface operation parameters and return types
/// - Interface attributes
/// - Type aliases
#[allow(clippy::too_many_lines)]
fn collect_arrays(
    hir: &ResolvedGraph,
    def_id: DefId,
    arrays: &mut Vec<ArrayReplacement>,
    encounter_order: &mut usize,
) {
    let def = hir.context.definitions.get(def_id);
    let parent = def.parent;

    match &def.kind {
        DefKind::Module(module_ty) => {
            for &child_id in &module_ty.definitions {
                collect_arrays(hir, child_id, arrays, encounter_order);
            }
        }
        DefKind::Struct(struct_ty) => {
            for member in &struct_ty.members {
                if let TyKind::Array { ty, len, .. } = &member.ty.kind {
                    arrays.push(ArrayReplacement {
                        elem_ty: ty.clone(),
                        len: *len,
                        span: member.ty.span,
                        parent,
                        used_by: def_id,
                        encounter_order: *encounter_order,
                    });
                    *encounter_order += 1;
                }
            }
        }
        DefKind::Union(union_ty) => {
            for variant in &union_ty.variants {
                if let TyKind::Array { ty, len, .. } = &variant.ty.kind {
                    arrays.push(ArrayReplacement {
                        elem_ty: ty.clone(),
                        len: *len,
                        span: variant.ty.span,
                        parent,
                        used_by: def_id,
                        encounter_order: *encounter_order,
                    });
                    *encounter_order += 1;
                }
            }
        }
        DefKind::Except(except_ty) => {
            for member in &except_ty.members {
                if let TyKind::Array { ty, len, .. } = &member.ty.kind {
                    arrays.push(ArrayReplacement {
                        elem_ty: ty.clone(),
                        len: *len,
                        span: member.ty.span,
                        parent,
                        used_by: def_id,
                        encounter_order: *encounter_order,
                    });
                    *encounter_order += 1;
                }
            }
        }
        DefKind::Interface(interface_ty) => {
            for proto in &interface_ty.prototypes {
                if let TyKind::Array { ty, len, .. } = &proto.ty.kind {
                    arrays.push(ArrayReplacement {
                        elem_ty: ty.clone(),
                        len: *len,
                        span: proto.ty.span,
                        parent,
                        used_by: def_id,
                        encounter_order: *encounter_order,
                    });
                    *encounter_order += 1;
                }
                for param in &proto.params {
                    if let TyKind::Array { ty, len, .. } = &param.ty.kind {
                        arrays.push(ArrayReplacement {
                            elem_ty: ty.clone(),
                            len: *len,
                            span: param.ty.span,
                            parent,
                            used_by: def_id,
                            encounter_order: *encounter_order,
                        });
                        *encounter_order += 1;
                    }
                }
            }
            for attr in &interface_ty.attributes {
                if let TyKind::Array { ty, len, .. } = &attr.ty.kind {
                    arrays.push(ArrayReplacement {
                        elem_ty: ty.clone(),
                        len: *len,
                        span: attr.ty.span,
                        parent,
                        used_by: def_id,
                        encounter_order: *encounter_order,
                    });
                    *encounter_order += 1;
                }
            }
            for &child_id in &interface_ty.definitions {
                collect_arrays(hir, child_id, arrays, encounter_order);
            }
        }
        DefKind::Valuetype(value_ty) => {
            for member in &value_ty.members {
                if let TyKind::Array { ty, len, .. } = &member.ty.kind {
                    arrays.push(ArrayReplacement {
                        elem_ty: ty.clone(),
                        len: *len,
                        span: member.ty.span,
                        parent,
                        used_by: def_id,
                        encounter_order: *encounter_order,
                    });
                    *encounter_order += 1;
                }
            }
            for &child_id in &value_ty.definitions {
                collect_arrays(hir, child_id, arrays, encounter_order);
            }
        }
        _ => {}
    }
}

/// Replace a single type with a reference to a synthesized array alias if it's
/// an array.
fn replace_type_if_array(ty: &mut Ty, array_types: &HashMap<ArrayKey, DefId>) {
    if let TyKind::Array {
        ty: elem_ty, len, ..
    } = &ty.kind
    {
        let key = make_array_key(elem_ty, *len);

        if let Some(&array_def_id) = array_types.get(&key) {
            let old_span = ty.span;
            *ty = Ty {
                span: old_span,
                kind: TyKind::Adt(array_def_id),
            };
        }
    }
}

/// Generate a canonical name for a primitive type for use in synthesized type
/// names.
///
/// Maps IDL primitive types to Ada-style names (e.g., `Int32` → `"Long"`).
fn primitive_name(prim: PrimitiveTy) -> String {
    match prim {
        PrimitiveTy::Int8 => "Int8",
        PrimitiveTy::Int16 => "Short",
        PrimitiveTy::Int32 => "Long",
        PrimitiveTy::Int64 => "Long_Long",
        PrimitiveTy::UInt8 => "Octet",
        PrimitiveTy::UInt16 => "Unsigned_Short",
        PrimitiveTy::UInt32 => "Unsigned_Long",
        PrimitiveTy::UInt64 => "Unsigned_Long_Long",
        _ => return format!("{prim:?}"),
    }
    .to_string()
}

/// Create a deduplication key for an array type.
///
/// The key captures the full structure of the element type to ensure that
/// different types with the same length don't collide. For nested types
/// (sequences, maps), the key recursively captures their structure.
fn make_array_key(ty: &Ty, len: usize) -> ArrayKey {
    match &ty.kind {
        TyKind::Primitive(prim) => ArrayKey::Primitive {
            name: primitive_name(*prim),
            len,
        },
        TyKind::Adt(def_id) => ArrayKey::Adt {
            def_id: *def_id,
            len,
        },
        TyKind::String { wide, bound, .. } => ArrayKey::String {
            wide: *wide,
            bound: *bound,
            len,
        },
        TyKind::Sequence {
            ty: elem_ty, bound, ..
        } => ArrayKey::Sequence {
            elem: Box::new(make_array_key(elem_ty, 0)),
            bound: *bound,
            len,
        },
        TyKind::Array {
            ty: elem_ty,
            len: elem_len,
            ..
        } => ArrayKey::Array {
            elem: Box::new(make_array_key(elem_ty, 0)),
            elem_len: *elem_len,
            len,
        },
        TyKind::Map {
            key, elem, bound, ..
        } => ArrayKey::Map {
            key: Box::new(make_array_key(key, 0)),
            value: Box::new(make_array_key(elem, 0)),
            bound: *bound,
            len,
        },
        TyKind::Any => ArrayKey::Other {
            name: "Any".to_string(),
            len,
        },
        TyKind::Fixed => ArrayKey::Other {
            name: "Fixed".to_string(),
            len,
        },
        TyKind::Null => ArrayKey::Other {
            name: "Null".to_string(),
            len,
        },
    }
}

/// Generate a human-readable name for a type for use in synthesized alias
/// names.
///
/// Produces names like `"Long"`, `"Foo"`, `"Bounded_String_10"`,
/// `"Sequence_Long"`, or `"Map_String_Long"` that will be used in the
/// synthesized typedef name.
fn type_name(hir: &ResolvedGraph, ty: &Ty) -> String {
    match &ty.kind {
        TyKind::Primitive(prim) => primitive_name(*prim),
        TyKind::Adt(def_id) => hir.context.definitions.get(*def_id).ident.name.clone(),
        TyKind::String { wide, bound, .. } => {
            if *wide {
                if let Some(bound) = bound {
                    format!("Bounded_Wide_String_{bound}")
                } else {
                    "Wide_String".to_string()
                }
            } else if let Some(bound) = bound {
                format!("Bounded_String_{bound}")
            } else {
                "String".to_string()
            }
        }
        TyKind::Sequence {
            ty: elem_ty, bound, ..
        } => {
            let elem_name = type_name(hir, elem_ty);
            if let Some(bound) = bound {
                format!("Bounded_Sequence_{elem_name}_{bound}")
            } else {
                format!("Sequence_{elem_name}")
            }
        }
        TyKind::Array {
            ty: elem_ty,
            len: elem_len,
            ..
        } => {
            let elem_name = type_name(hir, elem_ty);
            format!("{elem_name}_{elem_len}_Array")
        }
        TyKind::Map {
            key, elem, bound, ..
        } => {
            let key_name = type_name(hir, key);
            let value_name = type_name(hir, elem);
            if let Some(bound) = bound {
                format!("Bounded_Map_{key_name}_{value_name}_{bound}")
            } else {
                format!("Map_{key_name}_{value_name}")
            }
        }
        TyKind::Any => "Any".to_string(),
        TyKind::Fixed => "Fixed".to_string(),
        TyKind::Null => "Null".to_string(),
    }
}
