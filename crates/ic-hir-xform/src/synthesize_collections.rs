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

//! Synthesize explicit type aliases for collection types (arrays, sequences,
//! maps).
//!
//! Some target languages (like Ada) require collection types to be declared as
//! explicit type aliases before they can be used. This transformation extracts
//! all inline collection types and converts them into typedef-style aliases.
//!
//! This is specifically designed for Ada code generation, where:
//! - Arrays must be explicit named types (Ada doesn't support anonymous arrays)
//! - Sequences need CORBA.Sequences generic instantiation
//! - Maps need Ada.Containers generic instantiation
//!
//! # Transformation
//!
//! Input IDL:
//! ```idl
//! module A {
//!   struct Example {
//!     long data[10];
//!     sequence<string, 3> names;
//!     map<long, string> mapping;
//!   };
//! };
//! ```
//!
//! Output HIR (conceptually):
//! ```idl
//! module A {
//!   typedef long A_Long_10_Array[10];
//!   typedef sequence<string, 3> A_IDL_BOUNDED_SEQUENCE_String_3;
//!   typedef map<long, string> A_IDL_MAP_Long_String;
//!
//!   struct Example {
//!     A_Long_10_Array data;
//!     A_IDL_BOUNDED_SEQUENCE_String_3 names;
//!     A_IDL_MAP_Long_String mapping;
//!   };
//! };
//! ```
//!
//! # Deduplication
//!
//! Collection types with identical structure are deduplicated.
//! The transformation creates unique keys for each distinct collection type:
//! - **Arrays**: Element type + length
//! - **Sequences**: Element type + bound (if any)
//! - **Maps**: Key type + element type + bound (if any)
//!
//! For complex nested types, the key recursively captures the full structure
//!
//! # Scoping
//!
//! Synthesized aliases are placed in the same scope as the first usage. Type
//! names include the full ancestor path (e.g., `A_B_IDL_SEQUENCE_String` for a
//! sequence in module B nested in module A).
//!
//! # Coverage
//!
//! The transformation handles collection types in:
//! - Struct members
//! - Union variants
//! - Exception members
//! - Valuetype members
//! - Interface operation parameters and return types
//! - Interface attributes
//! - Type aliases (typedefs)
//! - Constants

use std::collections::HashMap;

use ic_hir::ResolvedGraph;
use ic_hir::hir::{
    self, AliasTy, DefId, DefKind, Ident, InterfaceTy, ModuleTy, PrimitiveTy, Span, Ty, TyKind,
    ValueTy,
};

const ARRAY_SUFFIX: &str = "_Array";
const SEQUENCE_PREFIX: &str = "IDL_SEQUENCE_";
const BOUNDED_SEQUENCE_PREFIX: &str = "IDL_BOUNDED_SEQUENCE_";
const MAP_PREFIX: &str = "IDL_MAP_";
const BOUNDED_MAP_PREFIX: &str = "IDL_BOUNDED_MAP_";

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

/// Key for deduplicating sequence types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SequenceKey {
    Primitive {
        prim: String,
        bound: Option<usize>,
    },
    String {
        wide: bool,
        str_bound: Option<usize>,
        seq_bound: Option<usize>,
    },
    Adt {
        def_id: DefId,
        bound: Option<usize>,
    },
    Sequence {
        inner: Box<SequenceKey>,
        bound: Option<usize>,
    },
    Map {
        inner: Box<MapKey>,
        bound: Option<usize>,
    },
    Array {
        inner: Box<ArrayKey>,
        bound: Option<usize>,
    },
}

/// Key for deduplicating map types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MapKey {
    Simple {
        key: Box<SequenceKey>,
        elem: Box<SequenceKey>,
        bound: Option<usize>,
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

/// Represents a sequence type that needs to be synthesized into a type alias.
struct SequenceReplacement {
    /// The element type of the sequence
    elem_ty: Box<Ty>,

    /// The bound of the sequence
    bound: Option<usize>,

    /// Source span for the sequence type
    span: Span,

    /// Parent definition containing this sequence (None for top-level)
    parent: Option<DefId>,

    /// The definition that uses this sequence type
    used_by: DefId,

    /// Order in which this sequence was encountered (for stable ordering)
    encounter_order: usize,
}

/// Represents a map type that needs to be synthesized into a type alias.
struct MapReplacement {
    /// The key type of the map
    key_ty: Box<Ty>,

    /// The element type of the map
    elem_ty: Box<Ty>,

    /// The bound of the map
    bound: Option<usize>,

    /// Source span for the map type
    span: Span,

    /// Parent definition containing this map (None for top-level)
    parent: Option<DefId>,

    /// The definition that uses this map type
    used_by: DefId,

    /// Order in which this map was encountered (for stable ordering)
    encounter_order: usize,
}

fn insert_top_level_def(hir: &mut ResolvedGraph, def_id: DefId, used_by_id: DefId) {
    if let Some(pos) = hir.order.iter().position(|&id| id == used_by_id) {
        hir.order.insert(pos, def_id);
    } else {
        hir.order.insert(0, def_id);
    }
}

fn insert_nested_def(hir: &mut ResolvedGraph, def_id: DefId, parent_id: DefId, used_by_id: DefId) {
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
                    module_ty.definitions.insert(pos, def_id);
                }
                DefKind::Interface(interface_ty) => {
                    interface_ty.definitions.insert(pos, def_id);
                }
                DefKind::Valuetype(value_ty) => {
                    value_ty.definitions.insert(pos, def_id);
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
    // First pass: collect all collection types that need to be synthesized
    let mut arrays_to_synthesize: Vec<ArrayReplacement> = Vec::new();
    let mut sequences_to_synthesize: Vec<SequenceReplacement> = Vec::new();
    let mut maps_to_synthesize: Vec<MapReplacement> = Vec::new();
    let mut encounter_order = 0;

    let def_ids: Vec<DefId> = hir.order.clone();
    {
        let mut ctx = CollectionContext {
            arrays: &mut arrays_to_synthesize,
            sequences: &mut sequences_to_synthesize,
            maps: &mut maps_to_synthesize,
            encounter_order: &mut encounter_order,
        };
        for &def_id in &def_ids {
            collect_collections(&hir, def_id, &mut ctx);
        }
    }

    // Second pass: create synthetic type aliases and track where to insert them
    let mut array_types: HashMap<ArrayKey, DefId> = HashMap::new();
    let mut sequence_types: HashMap<SequenceKey, DefId> = HashMap::new();
    let mut map_types: HashMap<MapKey, DefId> = HashMap::new();
    let mut defs_to_insert: Vec<(DefId, Option<DefId>, DefId, usize)> = Vec::new();

    synthesize_arrays(
        &mut hir,
        arrays_to_synthesize,
        &mut array_types,
        &mut defs_to_insert,
    );
    synthesize_sequences(
        &mut hir,
        sequences_to_synthesize,
        &mut sequence_types,
        &mut defs_to_insert,
    );
    synthesize_maps(
        &mut hir,
        maps_to_synthesize,
        &mut map_types,
        &mut defs_to_insert,
    );

    // Sort by collection depth (to ensure dependencies come first), then by encounter order
    // Depth = 0 for typedefs of primitives/ADTs (e.g., sequence<long>)
    // Depth = 1 for typedefs containing one level of collection (e.g., map<string, sequence<long>>)
    // Depth = 2 for typedefs containing nested collections, etc.
    let mut defs_with_depth: Vec<(DefId, Option<DefId>, DefId, usize, usize)> = defs_to_insert
        .into_iter()
        .map(|(def_id, parent_id, used_by_id, encounter_order)| {
            let depth = compute_collection_depth(&hir, def_id);
            (def_id, parent_id, used_by_id, encounter_order, depth)
        })
        .collect();

    defs_with_depth.sort_by_key(|(_, _, _, enc, depth)| (*depth, *enc));

    for (def_id, parent_id, used_by_id, _, _) in defs_with_depth {
        match parent_id {
            None => insert_top_level_def(&mut hir, def_id, used_by_id),
            Some(parent_id) => insert_nested_def(&mut hir, def_id, parent_id, used_by_id),
        }
    }

    // Third pass: replace collection types with references to the aliases
    let synthesized_ids: std::collections::HashSet<DefId> = array_types
        .values()
        .chain(sequence_types.values())
        .chain(map_types.values())
        .copied()
        .collect();

    let def_ids_after_synthesis: Vec<DefId> = hir.order.clone();
    for &def_id in &def_ids_after_synthesis {
        replace_collections_recursive(
            &mut hir,
            def_id,
            &array_types,
            &sequence_types,
            &map_types,
            &synthesized_ids,
        );
    }

    hir
}

fn compute_collection_depth(hir: &ResolvedGraph, typedef_id: DefId) -> usize {
    let def = hir.context.definitions.get(typedef_id);
    if let DefKind::Alias(alias_ty) = &def.kind {
        collection_nesting_depth(&alias_ty.ty)
    } else {
        0
    }
}

fn collection_nesting_depth(ty: &Ty) -> usize {
    match &ty.kind {
        TyKind::Primitive(_)
        | TyKind::String { .. }
        | TyKind::Any
        | TyKind::Fixed
        | TyKind::Null
        | TyKind::Adt(_) => 0,
        TyKind::Array { ty: elem, .. } => {
            let inner_depth = collection_nesting_depth(elem);
            if matches!(
                elem.kind,
                TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. }
            ) {
                1 + inner_depth
            } else {
                0
            }
        }
        TyKind::Sequence { ty: elem, .. } => {
            let inner_depth = collection_nesting_depth(elem);
            if matches!(
                elem.kind,
                TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. }
            ) {
                1 + inner_depth
            } else {
                0
            }
        }
        TyKind::Map { key, elem, .. } => {
            let key_depth = collection_nesting_depth(key);
            let elem_depth = collection_nesting_depth(elem);
            let max_inner = key_depth.max(elem_depth);
            if matches!(
                key.kind,
                TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. }
            ) || matches!(
                elem.kind,
                TyKind::Array { .. } | TyKind::Sequence { .. } | TyKind::Map { .. }
            ) {
                1 + max_inner
            } else {
                0
            }
        }
    }
}

struct DefInsertion {
    def_id: DefId,
    parent_id: Option<DefId>,
    insert_before: DefId,
    encounter_order: usize,
}

fn create_typedef_def(
    hir: &mut ResolvedGraph,
    name: String,
    span: Span,
    parent: Option<DefId>,
    ty_kind: TyKind,
) -> DefId {
    hir.context.definitions.alloc_with_id(|id| hir::Def {
        id,
        ident: Ident { name, span },
        parent,
        annotations: Vec::new(),
        span,
        kind: DefKind::Alias(AliasTy {
            ty: Ty {
                span,
                kind: ty_kind,
            },
        }),
        flags: hir::DefFlags::nil(),
    })
}

fn synthesize_arrays(
    hir: &mut ResolvedGraph,
    arrays_to_synthesize: Vec<ArrayReplacement>,
    array_types: &mut HashMap<ArrayKey, DefId>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
) {
    for replacement in arrays_to_synthesize {
        let elem_name = type_name(hir, &replacement.elem_ty);
        let key = make_array_key(&replacement.elem_ty, replacement.len);

        if let std::collections::hash_map::Entry::Vacant(e) = array_types.entry(key) {
            let type_name = if let Some(parent_id) = replacement.parent {
                let prefix = ancestor_path(hir, parent_id);
                format!("{prefix}_{elem_name}_{}{ARRAY_SUFFIX}", replacement.len)
            } else {
                format!("{elem_name}_{}{ARRAY_SUFFIX}", replacement.len)
            };

            let ty_kind = TyKind::Array {
                ty: replacement.elem_ty.clone(),
                len: replacement.len,
                len_span: replacement.span,
            };

            let array_def_id = create_typedef_def(
                hir,
                type_name,
                replacement.span,
                replacement.parent,
                ty_kind,
            );

            defs_to_insert.push((
                array_def_id,
                replacement.parent,
                replacement.used_by,
                replacement.encounter_order,
            ));
            e.insert(array_def_id);
        }
    }
}

fn synthesize_sequences(
    hir: &mut ResolvedGraph,
    sequences_to_synthesize: Vec<SequenceReplacement>,
    sequence_types: &mut HashMap<SequenceKey, DefId>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
) {
    for replacement in sequences_to_synthesize {
        let elem_name = type_name(hir, &replacement.elem_ty);
        let elem_name = strip_prefix(&elem_name);
        let key = make_sequence_key(&replacement.elem_ty, replacement.bound);

        if let std::collections::hash_map::Entry::Vacant(e) = sequence_types.entry(key) {
            let type_name = if let Some(parent_id) = replacement.parent {
                let prefix = ancestor_path(hir, parent_id);
                if let Some(bound) = replacement.bound {
                    format!("{prefix}_{BOUNDED_SEQUENCE_PREFIX}{elem_name}_{bound}")
                } else {
                    format!("{prefix}_{SEQUENCE_PREFIX}{elem_name}")
                }
            } else if let Some(bound) = replacement.bound {
                format!("{BOUNDED_SEQUENCE_PREFIX}{elem_name}_{bound}")
            } else {
                format!("{SEQUENCE_PREFIX}{elem_name}")
            };

            let ty_kind = TyKind::Sequence {
                ty: replacement.elem_ty.clone(),
                bound: replacement.bound,
                bound_span: None,
            };

            let seq_def_id = create_typedef_def(
                hir,
                type_name,
                replacement.span,
                replacement.parent,
                ty_kind,
            );

            defs_to_insert.push((
                seq_def_id,
                replacement.parent,
                replacement.used_by,
                replacement.encounter_order,
            ));
            e.insert(seq_def_id);
        }
    }
}

fn synthesize_maps(
    hir: &mut ResolvedGraph,
    maps_to_synthesize: Vec<MapReplacement>,
    map_types: &mut HashMap<MapKey, DefId>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
) {
    for replacement in maps_to_synthesize {
        let key_name = type_name(hir, &replacement.key_ty);
        let key_name = strip_prefix(&key_name);
        let elem_name = type_name(hir, &replacement.elem_ty);
        let elem_name = strip_prefix(&elem_name);
        let key = make_map_key(&replacement.key_ty, &replacement.elem_ty, replacement.bound);

        if let std::collections::hash_map::Entry::Vacant(e) = map_types.entry(key) {
            let type_name = if let Some(parent_id) = replacement.parent {
                let prefix = ancestor_path(hir, parent_id);
                if let Some(bound) = replacement.bound {
                    format!("{prefix}_{BOUNDED_MAP_PREFIX}{key_name}_{elem_name}_{bound}")
                } else {
                    format!("{prefix}_{MAP_PREFIX}{key_name}_{elem_name}")
                }
            } else if let Some(bound) = replacement.bound {
                format!("{BOUNDED_MAP_PREFIX}{key_name}_{elem_name}_{bound}")
            } else {
                format!("{MAP_PREFIX}{key_name}_{elem_name}")
            };

            let ty_kind = TyKind::Map {
                key: replacement.key_ty.clone(),
                elem: replacement.elem_ty.clone(),
                bound: replacement.bound,
                bound_span: None,
            };

            let map_def_id = create_typedef_def(
                hir,
                type_name,
                replacement.span,
                replacement.parent,
                ty_kind,
            );

            defs_to_insert.push((
                map_def_id,
                replacement.parent,
                replacement.used_by,
                replacement.encounter_order,
            ));
            e.insert(map_def_id);
        }
    }
}

/// Get the full ancestor path for a definition (e.g., "`A_B_C`" for module C nested in B in A)
fn ancestor_path(hir: &ResolvedGraph, def_id: DefId) -> String {
    let mut parent_names = Vec::new();
    let mut current_id = def_id;
    loop {
        let def = hir.context.definitions.get(current_id);
        parent_names.push(def.ident.name.clone());
        if let Some(parent) = def.parent {
            current_id = parent;
        } else {
            break;
        }
    }
    parent_names.reverse();
    parent_names.join("_")
}

fn replace_in_def(
    def: &mut hir::Def,
    array_types: &HashMap<ArrayKey, DefId>,
    sequence_types: &HashMap<SequenceKey, DefId>,
    map_types: &HashMap<MapKey, DefId>,
) {
    match &mut def.kind {
        DefKind::Struct(struct_ty) => {
            for member in &mut struct_ty.members {
                replace_type_if_collection(&mut member.ty, array_types, sequence_types, map_types);
            }
        }
        DefKind::Union(union_ty) => {
            for variant in &mut union_ty.variants {
                replace_type_if_collection(&mut variant.ty, array_types, sequence_types, map_types);
            }
        }
        DefKind::Except(except_ty) => {
            for member in &mut except_ty.members {
                replace_type_if_collection(&mut member.ty, array_types, sequence_types, map_types);
            }
        }
        DefKind::Valuetype(value_ty) => {
            for member in &mut value_ty.members {
                replace_type_if_collection(&mut member.ty, array_types, sequence_types, map_types);
            }
            for proto in &mut value_ty.prototypes {
                replace_type_if_collection(&mut proto.ty, array_types, sequence_types, map_types);
                for param in &mut proto.params {
                    replace_type_if_collection(
                        &mut param.ty,
                        array_types,
                        sequence_types,
                        map_types,
                    );
                }
            }
            for attr in &mut value_ty.attributes {
                replace_type_if_collection(&mut attr.ty, array_types, sequence_types, map_types);
            }
        }
        DefKind::Alias(alias_ty) => {
            replace_type_if_collection(&mut alias_ty.ty, array_types, sequence_types, map_types);
        }
        DefKind::Interface(interface_ty) => {
            for proto in &mut interface_ty.prototypes {
                replace_type_if_collection(&mut proto.ty, array_types, sequence_types, map_types);
                for param in &mut proto.params {
                    replace_type_if_collection(
                        &mut param.ty,
                        array_types,
                        sequence_types,
                        map_types,
                    );
                }
            }
            for attr in &mut interface_ty.attributes {
                replace_type_if_collection(&mut attr.ty, array_types, sequence_types, map_types);
            }
        }
        DefKind::Const(const_ty) => {
            replace_type_if_collection(&mut const_ty.ty, array_types, sequence_types, map_types);
        }
        _ => {}
    }
}

/// Recursively replace inline collection types with references to synthesized
/// aliases.
///
/// This walks the definition tree and updates any `TyKind::Array`, `TyKind::Sequence`,
/// or `TyKind::Map` to `TyKind::Adt` pointing to the corresponding synthesized alias.
/// Skips synthesized typedefs to avoid self-references.
fn replace_collections_recursive(
    hir: &mut ResolvedGraph,
    def_id: DefId,
    array_types: &HashMap<ArrayKey, DefId>,
    sequence_types: &HashMap<SequenceKey, DefId>,
    map_types: &HashMap<MapKey, DefId>,
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
        replace_collections_recursive(
            hir,
            child_id,
            array_types,
            sequence_types,
            map_types,
            synthesized_ids,
        );
    }

    hir.context.definitions.fold(def_id, |mut def| {
        if synthesized_ids.contains(&def_id) {
            // For synthesized typedefs, only replace types in nested element types,
            // not the top-level collection type itself
            if let DefKind::Alias(alias_ty) = &mut def.kind {
                match &mut alias_ty.ty.kind {
                    TyKind::Array { ty: elem_ty, .. } | TyKind::Sequence { ty: elem_ty, .. } => {
                        replace_type_if_collection(elem_ty, array_types, sequence_types, map_types);
                    }
                    TyKind::Map { key, elem, .. } => {
                        replace_type_if_collection(key, array_types, sequence_types, map_types);
                        replace_type_if_collection(elem, array_types, sequence_types, map_types);
                    }
                    _ => {}
                }
            }
        } else {
            replace_in_def(&mut def, array_types, sequence_types, map_types);
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
struct CollectionContext<'a> {
    arrays: &'a mut Vec<ArrayReplacement>,
    sequences: &'a mut Vec<SequenceReplacement>,
    maps: &'a mut Vec<MapReplacement>,
    encounter_order: &'a mut usize,
}

/// Recursively collect arrays, sequences, and maps from a type
fn collect_from_ty(
    ty: &Ty,
    span: Span,
    parent: Option<DefId>,
    used_by: DefId,
    ctx: &mut CollectionContext,
) {
    match &ty.kind {
        TyKind::Array {
            ty: elem_ty, len, ..
        } => {
            ctx.arrays.push(ArrayReplacement {
                elem_ty: elem_ty.clone(),
                len: *len,
                span,
                parent,
                used_by,
                encounter_order: *ctx.encounter_order,
            });
            *ctx.encounter_order += 1;
            collect_from_ty(elem_ty, span, parent, used_by, ctx);
        }
        TyKind::Sequence {
            ty: elem_ty, bound, ..
        } => {
            ctx.sequences.push(SequenceReplacement {
                elem_ty: elem_ty.clone(),
                bound: *bound,
                span,
                parent,
                used_by,
                encounter_order: *ctx.encounter_order,
            });
            *ctx.encounter_order += 1;
            collect_from_ty(elem_ty, span, parent, used_by, ctx);
        }
        TyKind::Map {
            key, elem, bound, ..
        } => {
            ctx.maps.push(MapReplacement {
                key_ty: key.clone(),
                elem_ty: elem.clone(),
                bound: *bound,
                span,
                parent,
                used_by,
                encounter_order: *ctx.encounter_order,
            });
            *ctx.encounter_order += 1;
            collect_from_ty(key, span, parent, used_by, ctx);
            collect_from_ty(elem, span, parent, used_by, ctx);
        }
        _ => {}
    }
}

fn collect_collections(hir: &ResolvedGraph, def_id: DefId, ctx: &mut CollectionContext) {
    let def = hir.context.definitions.get(def_id);
    let parent = def.parent;

    match &def.kind {
        DefKind::Module(module_ty) => {
            for &child_id in &module_ty.definitions {
                collect_collections(hir, child_id, ctx);
            }
        }
        DefKind::Struct(struct_ty) => {
            for member in &struct_ty.members {
                collect_from_ty(&member.ty, member.ty.span, parent, def_id, ctx);
            }
        }
        DefKind::Union(union_ty) => {
            for variant in &union_ty.variants {
                collect_from_ty(&variant.ty, variant.ty.span, parent, def_id, ctx);
            }
        }
        DefKind::Except(except_ty) => {
            for member in &except_ty.members {
                collect_from_ty(&member.ty, member.ty.span, parent, def_id, ctx);
            }
        }
        DefKind::Interface(interface_ty) => {
            for proto in &interface_ty.prototypes {
                collect_from_ty(&proto.ty, proto.ty.span, parent, def_id, ctx);
                for param in &proto.params {
                    collect_from_ty(&param.ty, param.ty.span, parent, def_id, ctx);
                }
            }
            for attr in &interface_ty.attributes {
                collect_from_ty(&attr.ty, attr.ty.span, parent, def_id, ctx);
            }
            for &child_id in &interface_ty.definitions {
                collect_collections(hir, child_id, ctx);
            }
        }
        DefKind::Valuetype(value_ty) => {
            for member in &value_ty.members {
                collect_from_ty(&member.ty, member.ty.span, parent, def_id, ctx);
            }
            for proto in &value_ty.prototypes {
                collect_from_ty(&proto.ty, proto.ty.span, parent, def_id, ctx);
                for param in &proto.params {
                    collect_from_ty(&param.ty, param.ty.span, parent, def_id, ctx);
                }
            }
            for attr in &value_ty.attributes {
                collect_from_ty(&attr.ty, attr.ty.span, parent, def_id, ctx);
            }
            for &child_id in &value_ty.definitions {
                collect_collections(hir, child_id, ctx);
            }
        }
        DefKind::Const(const_ty) => {
            collect_from_ty(&const_ty.ty, const_ty.ty.span, parent, def_id, ctx);
        }
        DefKind::Alias(alias_ty) => {
            collect_from_ty(&alias_ty.ty, alias_ty.ty.span, parent, def_id, ctx);
        }
        _ => {}
    }
}

/// Replace a single type with a reference to a synthesized collection alias if it's
/// an array, sequence, or map.
fn replace_type_if_collection(
    ty: &mut Ty,
    array_types: &HashMap<ArrayKey, DefId>,
    sequence_types: &HashMap<SequenceKey, DefId>,
    map_types: &HashMap<MapKey, DefId>,
) {
    match &ty.kind {
        TyKind::Array {
            ty: elem_ty, len, ..
        } => {
            let key = make_array_key(elem_ty, *len);
            if let Some(&array_def_id) = array_types.get(&key) {
                let old_span = ty.span;
                *ty = Ty {
                    span: old_span,
                    kind: TyKind::Adt(array_def_id),
                };
            }
        }
        TyKind::Sequence {
            ty: elem_ty, bound, ..
        } => {
            let key = make_sequence_key(elem_ty, *bound);
            if let Some(&seq_def_id) = sequence_types.get(&key) {
                let old_span = ty.span;
                *ty = Ty {
                    span: old_span,
                    kind: TyKind::Adt(seq_def_id),
                };
            }
        }
        TyKind::Map {
            key, elem, bound, ..
        } => {
            let map_key = make_map_key(key, elem, *bound);
            if let Some(&map_def_id) = map_types.get(&map_key) {
                let old_span = ty.span;
                *ty = Ty {
                    span: old_span,
                    kind: TyKind::Adt(map_def_id),
                };
            }
        }
        _ => {}
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

/// Strip package prefix from type names (e.g., "CORBA.String" -> "String")
fn strip_prefix(type_name: &str) -> &str {
    type_name.rsplit('.').next().unwrap_or(type_name)
}

/// Create a deduplication key for a sequence type.
fn make_sequence_key(ty: &Ty, bound: Option<usize>) -> SequenceKey {
    match &ty.kind {
        TyKind::Primitive(prim) => SequenceKey::Primitive {
            prim: format!("{prim:?}"),
            bound,
        },
        TyKind::String {
            wide,
            bound: str_bound,
            ..
        } => SequenceKey::String {
            wide: *wide,
            str_bound: *str_bound,
            seq_bound: bound,
        },
        TyKind::Adt(def_id) => SequenceKey::Adt {
            def_id: *def_id,
            bound,
        },
        TyKind::Sequence {
            ty: elem_ty,
            bound: inner_bound,
            ..
        } => {
            let inner = make_sequence_key(elem_ty, *inner_bound);
            SequenceKey::Sequence {
                inner: Box::new(inner),
                bound,
            }
        }
        TyKind::Map {
            key,
            elem,
            bound: inner_bound,
            ..
        } => {
            let inner = make_map_key(key, elem, *inner_bound);
            SequenceKey::Map {
                inner: Box::new(inner),
                bound,
            }
        }
        TyKind::Array {
            ty: elem_ty,
            len: arr_len,
            ..
        } => {
            let inner = make_array_key(elem_ty, *arr_len);
            SequenceKey::Array {
                inner: Box::new(inner),
                bound,
            }
        }
        TyKind::Any | TyKind::Fixed | TyKind::Null => SequenceKey::Primitive {
            prim: format!("{:?}", ty.kind),
            bound,
        },
    }
}

/// Create a deduplication key for a map type.
fn make_map_key(key_ty: &Ty, elem_ty: &Ty, bound: Option<usize>) -> MapKey {
    MapKey::Simple {
        key: Box::new(make_sequence_key(key_ty, None)),
        elem: Box::new(make_sequence_key(elem_ty, None)),
        bound,
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
                format!("IDL_BOUNDED_SEQUENCE_{elem_name}_{bound}")
            } else {
                format!("IDL_SEQUENCE_{elem_name}")
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
                format!("IDL_BOUNDED_MAP_{key_name}_{value_name}_{bound}")
            } else {
                format!("IDL_MAP_{key_name}_{value_name}")
            }
        }
        TyKind::Any => "Any".to_string(),
        TyKind::Fixed => "Fixed".to_string(),
        TyKind::Null => "Null".to_string(),
    }
}
