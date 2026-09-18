// Copyright 2026 KONGSBERG
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

//! Synthesize explicit type aliases for anonymous types (arrays, sequences,
//! maps, bounded strings and optional/external members).
//!
//! Some target languages (like Ada) require anonymous types to be declared as
//! explicit type aliases before they can be used. This transformation extracts
//! all inline anonymous types and converts them into typedef-style aliases.
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
use ic_hir::hir::{self, AliasTy, Ann, DefId, DefKind, Ident, PrimitiveTy, Span, Ty, TyKind};
use ic_hir_analysis::annotation::{MemberLike, is_external, is_optional};
use tracing::{debug, debug_span};

/// Conventions that control how synthesized alias identifiers are constructed for
/// anonymous types: arrays, sequences, maps, bounded strings and optional/external members.
#[derive(Default)]
pub struct Convention {
    /// Prefix/suffix for array aliases.
    pub array_prefix: &'static str,
    pub array_suffix: &'static str,

    /// Prefix/suffix for sequence aliases.
    pub sequence_prefix: &'static str,
    pub sequence_suffix: &'static str,

    /// Prefix/suffix for bounded sequence aliases.
    pub bounded_sequence_prefix: &'static str,
    pub bounded_sequence_suffix: &'static str,

    /// Prefix/suffix for map aliases.
    pub map_prefix: &'static str,
    pub map_suffix: &'static str,

    /// Prefix/suffix for bounded map aliases.
    pub bounded_map_prefix: &'static str,
    pub bounded_map_suffix: &'static str,

    /// Prefix/suffix for aliases that represent bounded trings.
    pub bounded_string_prefix: &'static str,
    pub bounded_string_suffix: &'static str,

    /// Prefix/suffix for aliases that represent bounded wide strings.
    pub bounded_wide_string_prefix: &'static str,
    pub bounded_wide_string_suffix: &'static str,

    /// Prefix/suffix used when a member is annotated `@optional` and aliased.
    pub optional_alias_prefix: &'static str,
    pub optional_alias_suffix: &'static str,

    /// Prefix/suffix used when a member is annotated `@external` and aliased.
    pub external_alias_prefix: &'static str,
    pub external_alias_suffix: &'static str,

    /// Subtype for array in synthesized identifiers.
    pub array_subtype: &'static str,

    /// Subtypes for strings in synthesized identifiers.
    pub string_subtype: &'static str,
    pub wide_string_subtype: &'static str,
    pub bounded_string_subtype: &'static str,
    pub bounded_wide_string_subtype: &'static str,

    /// Subtypes for sequences in synthesized identifiers.
    pub sequence_subtype: &'static str,
    pub bounded_sequence_subtype: &'static str,

    /// Subtypes for maps in synthesized identifiers.
    pub map_subtype: &'static str,
    pub bounded_map_subtype: &'static str,

    /// Separator used between sub types when composing an identifier.
    pub seperator: &'static str,

    /// Callback for the name of primitive types used in subtypes.
    pub primitive_name: Option<fn(PrimitiveTy) -> String>,
}

/// Which kind of annotation was found on a member
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AnnotationKind {
    Optional,
    External,
}

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

/// Key for deduplicating bounded strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BoundedStringKey {
    wide: bool,
    bound: usize,
}

/// Key for deduplicating annotations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AnnotationKey {
    name: String,
    adt_id: Option<DefId>,
    kind: AnnotationKind,
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

/// Represents a map type that needs to be synthesized into a type alias.
struct BoundedStringReplacement {
    /// The string is wide
    wide: bool,

    /// The bound of the string
    bound: usize,

    /// Source span for the string type
    span: Span,

    /// Parent definition containing this string (None for top-level)
    parent: Option<DefId>,

    /// The definition that uses this string type
    used_by: DefId,

    /// Order in which this string was encountered (for stable ordering)
    encounter_order: usize,
}

/// Represents an optional or external annotated member that needs to be synthesized to a type alias.
struct AnnotationReplacement {
    /// The type to synthesize a typedef for
    ty: Box<Ty>,

    /// Member index of field
    member_index: usize,

    /// Source span for the type
    span: Span,

    /// Parent definition containing this type
    parent: Option<DefId>,

    /// The definition holding the member
    used_by: DefId,

    /// The annotation to move onto the synthesized typedef
    ann: hir::Ann,

    /// The kind of annotation
    kind: AnnotationKind,

    /// Order in which this member was encountered (for stable ordering)
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
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn transform(mut hir: ResolvedGraph, convention: &Convention) -> ResolvedGraph {
    let _span = debug_span!("xform", name = "synthesize_collections").entered();
    debug!("applying transform");

    // First pass: collect all collection types that need to be synthesized
    let mut arrays_to_synthesize: Vec<ArrayReplacement> = Vec::new();
    let mut sequences_to_synthesize: Vec<SequenceReplacement> = Vec::new();
    let mut maps_to_synthesize: Vec<MapReplacement> = Vec::new();
    let mut bounded_strings_to_synthesize: Vec<BoundedStringReplacement> = Vec::new();
    let mut annotated_members_to_synthesize: Vec<AnnotationReplacement> = Vec::new();
    let mut encounter_order = 0;

    let def_ids: Vec<DefId> = hir.order.clone();
    {
        let mut ctx = CollectionContext {
            hir: &hir,
            arrays: &mut arrays_to_synthesize,
            sequences: &mut sequences_to_synthesize,
            maps: &mut maps_to_synthesize,
            bounded_strings: &mut bounded_strings_to_synthesize,
            annotated_members: &mut annotated_members_to_synthesize,
            encounter_order: &mut encounter_order,
        };
        for def_id in def_ids {
            collect_collections(def_id, &mut ctx);
        }
    }

    // Second pass: create synthetic type aliases and track where to insert them.
    // Deduplication is scoped to the containing definition so sibling modules can
    // synthesize equivalent collection types without reusing one another's aliases.
    let mut array_types: HashMap<Option<DefId>, HashMap<ArrayKey, DefId>> = HashMap::new();
    let mut sequence_types: HashMap<Option<DefId>, HashMap<SequenceKey, DefId>> = HashMap::new();
    let mut map_types: HashMap<Option<DefId>, HashMap<MapKey, DefId>> = HashMap::new();
    let mut bounded_string_types: HashMap<Option<DefId>, HashMap<BoundedStringKey, DefId>> =
        HashMap::new();
    let mut defs_to_insert: Vec<(DefId, Option<DefId>, DefId, usize)> = Vec::new();

    synthesize_arrays(
        &mut hir,
        arrays_to_synthesize,
        &mut array_types,
        &mut defs_to_insert,
        convention,
    );
    synthesize_sequences(
        &mut hir,
        sequences_to_synthesize,
        &mut sequence_types,
        &mut defs_to_insert,
        convention,
    );
    synthesize_maps(
        &mut hir,
        maps_to_synthesize,
        &mut map_types,
        &mut defs_to_insert,
        convention,
    );
    synthesize_bounded_strings(
        &mut hir,
        bounded_strings_to_synthesize,
        &mut bounded_string_types,
        &mut defs_to_insert,
        convention,
    );
    synthesize_annotated_members(
        &mut hir,
        annotated_members_to_synthesize,
        &mut defs_to_insert,
        convention,
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
        .flat_map(|scope_types| scope_types.values().copied())
        .chain(
            sequence_types
                .values()
                .flat_map(|scope_types| scope_types.values().copied()),
        )
        .chain(
            map_types
                .values()
                .flat_map(|scope_types| scope_types.values().copied()),
        )
        .chain(
            bounded_string_types
                .values()
                .flat_map(|scope_types| scope_types.values().copied()),
        )
        .collect();

    let def_ids_after_synthesis: Vec<DefId> = hir.order.clone();
    for &def_id in &def_ids_after_synthesis {
        replace_collections_recursive(
            &mut hir,
            def_id,
            &array_types,
            &sequence_types,
            &map_types,
            &bounded_string_types,
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

fn create_typedef_def(
    hir: &mut ResolvedGraph,
    name: String,
    span: Span,
    parent: Option<DefId>,
    ty_kind: TyKind,
    annotations: Vec<Ann>,
) -> DefId {
    hir.context.definitions.alloc_with_id(|id| hir::Def {
        id,
        ident: Ident { name, span },
        parent,
        annotations,
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
    array_types: &mut HashMap<Option<DefId>, HashMap<ArrayKey, DefId>>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
    convention: &Convention,
) {
    let Convention {
        array_prefix,
        array_suffix,
        seperator,
        ..
    } = convention;
    for replacement in arrays_to_synthesize {
        let elem_name = type_name(hir, &replacement.elem_ty, convention);
        let key = make_array_key(&replacement.elem_ty, replacement.len);
        let scope_types = array_types.entry(replacement.parent).or_default();

        if let std::collections::hash_map::Entry::Vacant(e) = scope_types.entry(key) {
            let type_name: String = format!(
                "{array_prefix}{elem_name}{seperator}{}{array_suffix}",
                replacement.len
            );

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
                Vec::new(),
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
    sequence_types: &mut HashMap<Option<DefId>, HashMap<SequenceKey, DefId>>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
    convention: &Convention,
) {
    let Convention {
        bounded_sequence_prefix,
        bounded_sequence_suffix,
        sequence_prefix,
        sequence_suffix,
        seperator,
        ..
    } = convention;
    for replacement in sequences_to_synthesize {
        let elem_name = type_name(hir, &replacement.elem_ty, convention);
        let elem_name = strip_prefix(&elem_name);
        let key = make_sequence_key(&replacement.elem_ty, replacement.bound);
        let scope_types = sequence_types.entry(replacement.parent).or_default();

        if let std::collections::hash_map::Entry::Vacant(e) = scope_types.entry(key) {
            let type_name = if let Some(bound) = replacement.bound {
                format!("{bounded_sequence_prefix}{elem_name}{seperator}{bound}{bounded_sequence_suffix}")
            } else {
                format!("{sequence_prefix}{elem_name}{sequence_suffix}")
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
                Vec::new(),
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
    map_types: &mut HashMap<Option<DefId>, HashMap<MapKey, DefId>>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
    convention: &Convention,
) {
    let Convention {
        bounded_map_prefix,
        bounded_map_suffix,
        map_prefix,
        map_suffix,
        seperator,
        ..
    } = convention;
    for replacement in maps_to_synthesize {
        let key_name = type_name(hir, &replacement.key_ty, convention);
        let key_name = strip_prefix(&key_name);
        let elem_name = type_name(hir, &replacement.elem_ty, convention);
        let elem_name = strip_prefix(&elem_name);
        let key = make_map_key(&replacement.key_ty, &replacement.elem_ty, replacement.bound);
        let scope_types = map_types.entry(replacement.parent).or_default();

        if let std::collections::hash_map::Entry::Vacant(e) = scope_types.entry(key) {
            let type_name = if let Some(bound) = replacement.bound {
                format!("{bounded_map_prefix}{key_name}{seperator}{elem_name}{seperator}{bound}{bounded_map_suffix}")
            } else {
                format!("{map_prefix}{key_name}{seperator}{elem_name}{map_suffix}")
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
                Vec::new(),
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

fn synthesize_bounded_strings(
    hir: &mut ResolvedGraph,
    bounded_strings_to_synthesize: Vec<BoundedStringReplacement>,
    bounded_strings_types: &mut HashMap<Option<DefId>, HashMap<BoundedStringKey, DefId>>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
    Convention {
        bounded_string_prefix,
        bounded_string_suffix,
        bounded_wide_string_prefix,
        bounded_wide_string_suffix,
        ..
    }: &Convention,
) {
    for replacement in bounded_strings_to_synthesize {
        let bound = replacement.bound;
        let key = make_bounded_string_key(replacement.wide, bound);
        let scope_types = bounded_strings_types.entry(replacement.parent).or_default();

        if let std::collections::hash_map::Entry::Vacant(e) = scope_types.entry(key) {
            let type_name = if replacement.wide {
                format!("{bounded_wide_string_prefix}{bound}{bounded_wide_string_suffix}")
            } else {
                format!("{bounded_string_prefix}{bound}{bounded_string_suffix}")
            };

            let ty_kind = TyKind::String {
                wide: replacement.wide,
                bound: Some(bound),
                bound_span: None,
            };

            let string_def_id = create_typedef_def(
                hir,
                type_name,
                replacement.span,
                replacement.parent,
                ty_kind,
                Vec::new(),
            );

            defs_to_insert.push((
                string_def_id,
                replacement.parent,
                replacement.used_by,
                replacement.encounter_order,
            ));
            e.insert(string_def_id);
        }
    }
}

fn synthesize_annotated_members(
    hir: &mut ResolvedGraph,
    annotated_members_to_synthesize: Vec<AnnotationReplacement>,
    defs_to_insert: &mut Vec<(DefId, Option<DefId>, DefId, usize)>,
    convention: &Convention,
) {
    let Convention {
        optional_alias_prefix,
        optional_alias_suffix,
        external_alias_prefix,
        external_alias_suffix,
        ..
    } = convention;
    let mut optional_types: HashMap<Option<DefId>, HashMap<AnnotationKey, DefId>> = HashMap::new();
    let mut optional_moves = vec![];

    for replacement in annotated_members_to_synthesize {
        let aliased_type_name = type_name(hir, &replacement.ty, convention);
        let adt_id = match replacement.ty.kind {
            TyKind::Adt(adt_id) => Some(adt_id),
            _ => None,
        };
        let key = AnnotationKey {
            name: aliased_type_name.clone(),
            adt_id,
            kind: replacement.kind,
        };
        let scope = optional_types.entry(replacement.parent).or_default();

        let typedef_id = match scope.entry(key) {
            std::collections::hash_map::Entry::Occupied(e) => *e.get(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let type_name = if replacement.kind == AnnotationKind::Optional {
                    format!("{optional_alias_prefix}{aliased_type_name}{optional_alias_suffix}")
                } else {
                    format!("{external_alias_prefix}{aliased_type_name}{external_alias_suffix}")
                };

                let typedef_id = create_typedef_def(
                    hir,
                    type_name,
                    replacement.span,
                    replacement.parent,
                    replacement.ty.kind.clone(),
                    Vec::new(),
                );

                defs_to_insert.push((
                    typedef_id,
                    replacement.parent,
                    replacement.used_by,
                    replacement.encounter_order,
                ));
                *e.insert(typedef_id)
            }
        };

        optional_moves.push((
            replacement.used_by,
            replacement.member_index,
            typedef_id,
            replacement.ann,
            replacement.kind,
        ));
    }

    for (def_id, member_index, typedef_id, ann, kind) in optional_moves {
        hir.context.definitions.fold(def_id, |mut def| {
            match &mut def.kind {
                DefKind::Struct(s) => {
                    replace_annotated_ty(&mut s.members[member_index], typedef_id);
                }
                DefKind::Union(u) => {
                    replace_annotated_ty(&mut u.variants[member_index], typedef_id);
                }
                DefKind::Except(e) => {
                    replace_annotated_ty(&mut e.members[member_index], typedef_id);
                }
                DefKind::Valuetype(v) => {
                    replace_annotated_ty(&mut v.members[member_index], typedef_id);
                }
                _ => {}
            }
            def
        });

        let typedef = hir.context.definitions.get(typedef_id);
        if !(is_external(&hir.context, typedef) && kind == AnnotationKind::External
            || is_optional(&hir.context, typedef) && kind == AnnotationKind::Optional)
        {
            continue;
        }

        // Attach annotation to alias
        hir.context
            .definitions
            .fold(typedef_id, |mut td: hir::Def| {
                td.annotations.push(ann);
                td
            });
    }
}

fn move_annotations_in_def(
    hir: &mut ResolvedGraph,
    member: &mut impl MemberKindMut,
    moved_annotations: &mut Vec<(DefId, hir::Ann)>,
) {
    if let TyKind::Adt(typedef_id) = member.ty().kind {
        // remove optional annotation from member
        if let Some(opt_ann) =
            ic_hir_analysis::annotation::optional_annotation(&hir.context, member)
        {
            let opt_def_id = opt_ann.def_id;
            if let Some(pos) = member
                .annotations()
                .iter()
                .position(|a| a.def_id == opt_def_id)
            {
                let ann = member.annotations_mut().remove(pos);
                // attach the annotation as the last step
                moved_annotations.push((typedef_id, ann));
            }
        }
        // remove external annotation from member
        if let Some(ext_ann) =
            ic_hir_analysis::annotation::external_annotation(&hir.context, member)
        {
            let opt_def_id = ext_ann.def_id;
            if let Some(pos) = member
                .annotations()
                .iter()
                .position(|a| a.def_id == opt_def_id)
            {
                let ann = member.annotations_mut().remove(pos);
                // attach the annotation as the last step
                moved_annotations.push((typedef_id, ann));
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn replace_in_def(
    def: &mut hir::Def,
    hir: &mut ResolvedGraph,
    array_types: &HashMap<Option<DefId>, HashMap<ArrayKey, DefId>>,
    sequence_types: &HashMap<Option<DefId>, HashMap<SequenceKey, DefId>>,
    map_types: &HashMap<Option<DefId>, HashMap<MapKey, DefId>>,
    bounded_string_types: &HashMap<Option<DefId>, HashMap<BoundedStringKey, DefId>>,
    scope_id: Option<DefId>,
    moved_annotations: &mut Vec<(DefId, hir::Ann)>,
) {
    match &mut def.kind {
        DefKind::Struct(struct_ty) => {
            for member in &mut struct_ty.members {
                replace_type_if_collection(
                    &mut member.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
                move_annotations_in_def(hir, member, moved_annotations);
            }
        }
        DefKind::Union(union_ty) => {
            for variant in &mut union_ty.variants {
                replace_type_if_collection(
                    &mut variant.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
                move_annotations_in_def(hir, variant, moved_annotations);
            }
        }
        DefKind::Except(except_ty) => {
            for member in &mut except_ty.members {
                replace_type_if_collection(
                    &mut member.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
                move_annotations_in_def(hir, member, moved_annotations);
            }
        }
        DefKind::Valuetype(value_ty) => {
            for member in &mut value_ty.members {
                replace_type_if_collection(
                    &mut member.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
                move_annotations_in_def(hir, member, moved_annotations);
            }
            for proto in &mut value_ty.prototypes {
                replace_type_if_collection(
                    &mut proto.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
                for param in &mut proto.params {
                    replace_type_if_collection(
                        &mut param.ty,
                        array_types,
                        sequence_types,
                        map_types,
                        bounded_string_types,
                        scope_id,
                    );
                }
            }
            for attr in &mut value_ty.attributes {
                replace_type_if_collection(
                    &mut attr.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
            }
        }
        DefKind::Interface(interface_ty) => {
            for proto in &mut interface_ty.prototypes {
                replace_type_if_collection(
                    &mut proto.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
                for param in &mut proto.params {
                    replace_type_if_collection(
                        &mut param.ty,
                        array_types,
                        sequence_types,
                        map_types,
                        bounded_string_types,
                        scope_id,
                    );
                }
            }
            for attr in &mut interface_ty.attributes {
                replace_type_if_collection(
                    &mut attr.ty,
                    array_types,
                    sequence_types,
                    map_types,
                    bounded_string_types,
                    scope_id,
                );
            }
        }
        DefKind::Const(const_ty) => {
            replace_type_if_collection(
                &mut const_ty.ty,
                array_types,
                sequence_types,
                map_types,
                bounded_string_types,
                scope_id,
            );
        }
        DefKind::Alias(alias_ty) => {
            replace_type_if_collection(
                &mut alias_ty.ty,
                array_types,
                sequence_types,
                map_types,
                bounded_string_types,
                scope_id,
            );
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
#[allow(clippy::too_many_arguments)]
fn replace_collections_recursive(
    hir: &mut ResolvedGraph,
    def_id: DefId,
    array_types: &HashMap<Option<DefId>, HashMap<ArrayKey, DefId>>,
    sequence_types: &HashMap<Option<DefId>, HashMap<SequenceKey, DefId>>,
    map_types: &HashMap<Option<DefId>, HashMap<MapKey, DefId>>,
    bounded_string_types: &HashMap<Option<DefId>, HashMap<BoundedStringKey, DefId>>,
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
            bounded_string_types,
            synthesized_ids,
        );
    }

    // record moved optional annotations here to avoid nested mutable borrows on the
    // definitions arena while iterating/updating a definition.
    let mut moved_annotations: Vec<(DefId, hir::Ann)> = Vec::new();

    // Work on a cloned copy of the definition to avoid holding a mutable borrow to
    // the definitions arena while calling functions that need to read/modify it.
    let mut def = hir.context.definitions.get(def_id).clone();
    let scope_id = def.parent;
    if synthesized_ids.contains(&def_id) {
        // For synthesized typedefs, only replace types in nested element types,
        // not the top-level collection type itself.
        if let DefKind::Alias(alias_ty) = &mut def.kind {
            match &mut alias_ty.ty.kind {
                TyKind::Array { ty: elem_ty, .. } | TyKind::Sequence { ty: elem_ty, .. } => {
                    replace_type_if_collection(
                        elem_ty,
                        array_types,
                        sequence_types,
                        map_types,
                        bounded_string_types,
                        scope_id,
                    );
                }
                TyKind::Map { key, elem, .. } => {
                    replace_type_if_collection(
                        key,
                        array_types,
                        sequence_types,
                        map_types,
                        bounded_string_types,
                        scope_id,
                    );
                    replace_type_if_collection(
                        elem,
                        array_types,
                        sequence_types,
                        map_types,
                        bounded_string_types,
                        scope_id,
                    );
                }
                _ => {}
            }
        }
    } else {
        replace_in_def(
            &mut def,
            hir,
            array_types,
            sequence_types,
            map_types,
            bounded_string_types,
            scope_id,
            &mut moved_annotations,
        );
    }

    // Write the possibly modified definition back into the arena.
    hir.context.definitions.fold(def_id, |_| def);

    // Apply recorded annotation moves now that the arena access is free.
    for (typedef_id, ann) in moved_annotations {
        hir.context.definitions.fold(typedef_id, |mut td| {
            if !td.annotations.iter().any(|a| a.def_id == ann.def_id) {
                td.annotations.push(ann.clone());
            }
            td
        });
    }
}

fn replace_annotated_ty(member: &mut impl MemberKindMut, typedef_id: DefId) {
    // replace member type with typedef
    let old_span = member.ty().span;
    *member.ty_mut() = Ty {
        span: old_span,
        kind: TyKind::Adt(typedef_id),
    };
}

/// Collect all inline array types that need to be synthesized.
///
/// Recursively walks definitions and finds any `TyKind::Array` used in:
/// - Struct/union/exception/valuetype members
/// - Interface operation parameters and return types
/// - Interface attributes
/// - Type aliases
struct CollectionContext<'a> {
    hir: &'a ResolvedGraph,
    arrays: &'a mut Vec<ArrayReplacement>,
    sequences: &'a mut Vec<SequenceReplacement>,
    maps: &'a mut Vec<MapReplacement>,
    bounded_strings: &'a mut Vec<BoundedStringReplacement>,
    annotated_members: &'a mut Vec<AnnotationReplacement>,
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
            collect_from_ty(elem_ty, span, parent, used_by, ctx);
            ctx.arrays.push(ArrayReplacement {
                elem_ty: elem_ty.clone(),
                len: *len,
                span,
                parent,
                used_by,
                encounter_order: *ctx.encounter_order,
            });
            *ctx.encounter_order += 1;
        }
        TyKind::Sequence {
            ty: elem_ty, bound, ..
        } => {
            collect_from_ty(elem_ty, span, parent, used_by, ctx);
            ctx.sequences.push(SequenceReplacement {
                elem_ty: elem_ty.clone(),
                bound: *bound,
                span,
                parent,
                used_by,
                encounter_order: *ctx.encounter_order,
            });
            *ctx.encounter_order += 1;
        }
        TyKind::Map {
            key, elem, bound, ..
        } => {
            collect_from_ty(key, span, parent, used_by, ctx);
            collect_from_ty(elem, span, parent, used_by, ctx);
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
        }
        TyKind::String {
            wide,
            bound: Some(bound),
            ..
        } => {
            ctx.bounded_strings.push(BoundedStringReplacement {
                wide: *wide,
                bound: *bound,
                span,
                parent,
                used_by,
                encounter_order: *ctx.encounter_order,
            });
            *ctx.encounter_order += 1;
        }
        _ => {}
    }
}

fn collect_from_annotations(
    member: &impl MemberKind,
    member_index: usize,
    def_id: DefId,
    parent: Option<DefId>,
    ctx: &mut CollectionContext,
) {
    if let Some(opt_ann) =
        ic_hir_analysis::annotation::optional_annotation(&ctx.hir.context, member)
    {
        ctx.annotated_members.push(AnnotationReplacement {
            ty: Box::new(member.ty().clone()),
            member_index,
            span: member.ty().span,
            parent,
            used_by: def_id,
            ann: opt_ann.clone(),
            kind: AnnotationKind::Optional,
            encounter_order: *ctx.encounter_order,
        });
        *ctx.encounter_order += 1;
    }
    if let Some(ext_ann) =
        ic_hir_analysis::annotation::external_annotation(&ctx.hir.context, member)
    {
        ctx.annotated_members.push(AnnotationReplacement {
            ty: Box::new(member.ty().clone()),
            member_index,
            span: member.ty().span,
            parent,
            used_by: def_id,
            ann: ext_ann.clone(),
            kind: AnnotationKind::External,
            encounter_order: *ctx.encounter_order,
        });
        *ctx.encounter_order += 1;
    }
}

#[allow(clippy::too_many_lines)]
fn collect_collections(def_id: DefId, ctx: &mut CollectionContext) {
    let def = ctx.hir.context.definitions.get(def_id);
    let parent = def.parent;

    match &def.kind {
        DefKind::Module(module_ty) => {
            for &child_id in &module_ty.definitions {
                collect_collections(child_id, ctx);
            }
        }
        DefKind::Struct(struct_ty) => {
            for (i, member) in struct_ty.members.iter().enumerate() {
                collect_from_ty(&member.ty, member.ty.span, parent, def_id, ctx);
                collect_from_annotations(member, i, def_id, parent, ctx);
            }
        }
        DefKind::Union(union_ty) => {
            for (i, variant) in union_ty.variants.iter().enumerate() {
                collect_from_ty(&variant.ty, variant.ty.span, parent, def_id, ctx);
                collect_from_annotations(variant, i, def_id, parent, ctx);
            }
        }
        DefKind::Except(except_ty) => {
            for (i, member) in except_ty.members.iter().enumerate() {
                collect_from_ty(&member.ty, member.ty.span, parent, def_id, ctx);
                collect_from_annotations(member, i, def_id, parent, ctx);
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
                collect_collections(child_id, ctx);
            }
        }
        DefKind::Valuetype(value_ty) => {
            for (i, member) in value_ty.members.iter().enumerate() {
                collect_from_ty(&member.ty, member.ty.span, parent, def_id, ctx);
                collect_from_annotations(member, i, def_id, parent, ctx);
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
                collect_collections(child_id, ctx);
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
    array_types: &HashMap<Option<DefId>, HashMap<ArrayKey, DefId>>,
    sequence_types: &HashMap<Option<DefId>, HashMap<SequenceKey, DefId>>,
    map_types: &HashMap<Option<DefId>, HashMap<MapKey, DefId>>,
    bounded_string_types: &HashMap<Option<DefId>, HashMap<BoundedStringKey, DefId>>,
    scope_id: Option<DefId>,
) {
    match &ty.kind {
        TyKind::Array {
            ty: elem_ty, len, ..
        } => {
            let key = make_array_key(elem_ty, *len);
            if let Some(array_def_id) = array_types
                .get(&scope_id)
                .and_then(|scope_types| scope_types.get(&key))
                .copied()
            {
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
            if let Some(seq_def_id) = sequence_types
                .get(&scope_id)
                .and_then(|scope_types| scope_types.get(&key))
                .copied()
            {
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
            if let Some(map_def_id) = map_types
                .get(&scope_id)
                .and_then(|scope_types| scope_types.get(&map_key))
                .copied()
            {
                let old_span = ty.span;
                *ty = Ty {
                    span: old_span,
                    kind: TyKind::Adt(map_def_id),
                };
            }
        }
        TyKind::String {
            wide,
            bound: Some(bound),
            ..
        } => {
            let key = make_bounded_string_key(*wide, *bound);
            if let Some(string_def_id) = bounded_string_types
                .get(&scope_id)
                .and_then(|scope_types| scope_types.get(&key))
                .copied()
            {
                let old_span = ty.span;
                *ty = Ty {
                    span: old_span,
                    kind: TyKind::Adt(string_def_id),
                };
            }
        }
        _ => {}
    }
}

/// Create a deduplication key for an array type.
///
/// The key captures the full structure of the element type to ensure that
/// different types with the same length don't collide. For nested types
/// (sequences, maps), the key recursively captures their structure.
fn make_array_key(ty: &Ty, len: usize) -> ArrayKey {
    match &ty.kind {
        TyKind::Primitive(prim) => ArrayKey::Primitive {
            name: prim.name().into(),
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

/// Create a deduplication key for a map type.
fn make_bounded_string_key(wide: bool, bound: usize) -> BoundedStringKey {
    BoundedStringKey { wide, bound }
}
/// Generate a human-readable name for a type for use in synthesized alias
/// names.
///
/// Produces names like `"Long"`, `"Foo"`, `"Bounded_String_10"`,
/// `"Sequence_Long"`, or `"Map_String_Long"` that will be used in the
/// synthesized typedef name.
#[must_use]
pub fn type_name(hir: &ResolvedGraph, ty: &Ty, convention: &Convention) -> String {
    let sep = convention.seperator;
    match &ty.kind {
        TyKind::Primitive(prim) => convention.primitive_name.map_or_else(
            || prim.name().replace(' ', convention.seperator),
            |callback| callback(*prim),
        ),
        TyKind::Adt(def_id) => {
            let def = hir.context.definitions.get(*def_id);
            let mut parts = vec![def.ident.name.clone()];
            let mut parent = def.parent;
            while let Some(parent_id) = parent {
                let parent_def = hir.context.definitions.get(parent_id);
                parts.push(parent_def.ident.name.clone());
                parent = parent_def.parent;
            }
            parts.reverse();

            parts.join(sep)
        }
        TyKind::String { wide, bound, .. } => {
            if *wide {
                if let Some(bound) = bound {
                    format!("{}{sep}{bound}", convention.bounded_wide_string_subtype)
                } else {
                    convention.wide_string_subtype.to_string()
                }
            } else if let Some(bound) = bound {
                format!("{}{sep}{bound}", convention.bounded_string_subtype)
            } else {
                convention.string_subtype.to_string()
            }
        }
        TyKind::Sequence {
            ty: elem_ty, bound, ..
        } => {
            let elem_name = type_name(hir, elem_ty, convention);
            if let Some(bound) = bound {
                format!(
                    "{}{sep}{elem_name}{sep}{bound}",
                    convention.bounded_sequence_subtype
                )
            } else {
                format!("{}{sep}{elem_name}", convention.sequence_subtype)
            }
        }
        TyKind::Array {
            ty: elem_ty,
            len: elem_len,
            ..
        } => {
            let elem_name = type_name(hir, elem_ty, convention);
            format!(
                "{}{sep}{elem_name}{sep}{elem_len}",
                convention.array_subtype
            )
        }
        TyKind::Map {
            key, elem, bound, ..
        } => {
            let key_name = type_name(hir, key, convention);
            let value_name = type_name(hir, elem, convention);
            if let Some(bound) = bound {
                format!(
                    "{}{sep}{key_name}{sep}{value_name}{sep}{bound}",
                    convention.bounded_map_subtype
                )
            } else {
                format!("{}{sep}{key_name}{sep}{value_name}", convention.map_subtype)
            }
        }
        TyKind::Any => "Any".to_string(),
        TyKind::Fixed => "Fixed".to_string(),
        TyKind::Null => "Null".to_string(),
    }
}

trait MemberKind: MemberLike {
    fn ty(&self) -> &Ty;
}

impl MemberKind for hir::Member {
    fn ty(&self) -> &Ty {
        &self.ty
    }
}

impl MemberKind for hir::Variant {
    fn ty(&self) -> &Ty {
        &self.ty
    }
}

trait MemberKindMut: MemberKind {
    fn ty_mut(&mut self) -> &mut Ty;
    fn annotations_mut(&mut self) -> &mut Vec<hir::Ann>;
}

impl MemberKindMut for hir::Member {
    fn ty_mut(&mut self) -> &mut Ty {
        &mut self.ty
    }
    fn annotations_mut(&mut self) -> &mut Vec<hir::Ann> {
        &mut self.annotations
    }
}

impl MemberKindMut for hir::Variant {
    fn ty_mut(&mut self) -> &mut Ty {
        &mut self.ty
    }
    fn annotations_mut(&mut self) -> &mut Vec<hir::Ann> {
        &mut self.annotations
    }
}
