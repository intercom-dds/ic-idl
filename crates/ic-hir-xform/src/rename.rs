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

use std::collections::{HashMap, HashSet};

use ic_emit::case::{self, Case};
use ic_hir::hir::DefKind;
use ic_hir::{ResolvedGraph, hir};

/// Function type for preprocessing names before case conversion
pub type NamePreprocessor = fn(&str) -> String;

/// The kind of identifier being renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierKind {
    /// Struct type
    Struct,

    /// Union type
    Union,

    /// Enum type
    Enum,

    /// Interface type
    Interface,

    /// Valuetype
    Valuetype,

    /// Type alias
    Alias,

    /// Bitmask type
    Bitmask,

    /// Bitset type
    Bitset,

    /// Exception type
    Exception,

    /// Annotation type
    Annotation,

    /// Member of struct, exception, or valuetype
    Member,

    /// Union variant
    Variant,

    /// Enum constant
    Enumerator,

    /// Bitmask flag
    BitFlag,

    /// Bitset field
    BitsetField,

    /// Constant
    Constant,

    /// Module/namespace
    Module,

    /// Interface or valuetype method
    Operation,

    /// Interface or valuetype attribute
    Attribute,

    /// Method parameter
    Parameter,
}

/// Context passed to the keyword escape function.
#[derive(Debug, Clone, Copy)]
pub struct RenameContext<'a> {
    /// The name to potentially escape
    pub name: &'a str,

    /// The kind of identifier
    pub kind: IdentifierKind,
}

/// Preprocessor that strips common suffixes like _t and _e
#[must_use]
pub fn strip_common_suffixes(name: &str) -> String {
    if name.len() > 2 {
        let lower = name.to_lowercase();
        if lower.ends_with("_t") || lower.ends_with("_e") {
            name[..name.len() - 2].to_string()
        } else {
            name.to_string()
        }
    } else {
        name.to_string()
    }
}

/// Naming conventions for different kinds of definitions.
///
/// If a field is `None`, the corresponding items will not have their case changed.
#[derive(Clone, Default)]
pub struct Convention {
    /// Structs
    pub struct_type: Option<Case>,

    /// Unions
    pub union_type: Option<Case>,

    /// Enums
    pub enum_type: Option<Case>,

    /// Interfaces
    pub interface: Option<Case>,

    /// Value types
    pub valuetype: Option<Case>,

    /// Type aliases
    pub alias: Option<Case>,

    /// Bitmasks
    pub bitmask: Option<Case>,

    /// Bitsets
    pub bitset: Option<Case>,

    /// Exceptions
    pub exception: Option<Case>,

    /// Annotations
    pub annotation: Option<Case>,

    /// Members of structs, exceptions, and value types
    pub member: Option<Case>,

    /// Members of unions (variants)
    pub variant: Option<Case>,

    /// Enum constants
    pub enumerator: Option<Case>,

    /// Bitmask flags
    pub bit_flag: Option<Case>,

    /// Bitset fields
    pub bitset_field: Option<Case>,

    /// Constants
    pub constant: Option<Case>,

    /// Modules
    pub module: Option<Case>,

    /// Interface methods/operations (prototypes)
    pub operation: Option<Case>,

    /// Interface attributes
    pub attribute: Option<Case>,

    /// Parameters for operations
    pub parameter: Option<Case>,

    /// Optional preprocessor function to apply to names before case conversion
    pub name_preprocessor: Option<NamePreprocessor>,

    /// Strip common prefixes from enum constants and bitmask flags.
    /// For example, if enum `Color` has constants `COLOR_RED`, `COLOR_GREEN`,
    /// they will be renamed to `RED`, `GREEN`.
    pub strip_enum_prefix: bool,
}

/// Configuration for the rename transformation.
#[derive(Clone, Default)]
pub struct Target {
    /// Case conversion settings for different definition kinds
    pub convention: Convention,

    /// Optional keyword escaper function.
    /// Receives context about the identifier being renamed.
    /// Returns `Some(escaped)` if the name needs escaping, `None` otherwise.
    pub keyword_escape: Option<fn(RenameContext) -> Option<String>>,

    /// Set of `DefIds` that were moved by previous transformations.
    /// These will have lower priority in collision resolution.
    pub moved_defs: HashSet<hir::DefId>,
}

/// Represents a node that needs renaming
#[derive(Debug)]
struct NodeRename {
    def_id: hir::DefId,
    original: String,
    desired: String,
    is_moved: bool,
}

/// Check if original is a natural fallback for desired (e.g., `FooBar`_ for `FooBar`)
fn is_natural_fallback(original: &str, desired: &str) -> bool {
    if original.len() <= desired.len() {
        return false;
    }

    if !original.starts_with(desired) {
        return false;
    }

    // Check if the remainder is all underscores
    original[desired.len()..].chars().all(|c| c == '_')
}

/// Apply case conversion and keyword escaping to a name
fn apply_rename(name: &str, case: Option<Case>, kind: IdentifierKind, target: &Target) -> String {
    let mut new_name = name.to_string();

    // First apply preprocessor if specified
    if let Some(preprocessor) = target.convention.name_preprocessor {
        new_name = preprocessor(&new_name);
    }

    // Then apply case conversion if specified
    if let Some(case) = case {
        new_name = case::convert(&new_name, case);
    }

    // Finally check if the result needs escaping
    if let Some(escaper) = target.keyword_escape {
        let ctx = RenameContext {
            name: &new_name,
            kind,
        };
        if let Some(escaped) = escaper(ctx) {
            new_name = escaped;
        }
    }

    new_name
}

/// Check if a string starts with an alphabetic character (allowing leading underscores)
fn starts_with_alpha(s: &str) -> bool {
    for c in s.chars() {
        if c.is_alphabetic() {
            return true;
        }
        if c != '_' {
            return false;
        }
    }
    false
}

/// Find the last delimiter (underscore or case change) in a string.
/// Returns the position where the prefix ends (exclusive).
fn rfind_delimiter(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut was_upper = false;

    for i in (1..chars.len()).rev() {
        let c = chars[i];
        let peek = chars[i - 1];

        if peek == '_' {
            return Some(i);
        } else if (c.is_lowercase() && peek.is_uppercase())
            || (was_upper && c.is_uppercase() && peek.is_lowercase())
        {
            return Some(i - 1);
        }
        was_upper = c.is_uppercase();
    }

    None
}

/// Find common prefix to strip from a list of names, given the parent type name.
/// Returns the prefix length to strip, or 0 if no prefix should be stripped.
fn find_prefix_to_strip(type_name: &str, names: &[String]) -> usize {
    if names.is_empty() {
        return 0;
    }

    let first_name = &names[0];
    let mut prefix = match rfind_delimiter(first_name) {
        Some(pos) => first_name[..pos].to_string(),
        None => return 0,
    };

    loop {
        // Check if all names have this prefix
        let all_have_prefix = names.iter().all(|name| {
            if name.len() > prefix.len() {
                let remainder = &name[prefix.len()..];
                let view = &name[..prefix.len()];
                starts_with_alpha(remainder) && view == prefix
            } else {
                false
            }
        });

        if all_have_prefix {
            // Convert both type name and prefix to snake_case for comparison
            let found_prefix = case::convert(&prefix, Case::Snake);
            let type_name_snake = case::convert(type_name, Case::Snake);

            // Check if the type name starts with the same prefix
            if type_name_snake.len() >= found_prefix.len()
                && type_name_snake.starts_with(&found_prefix)
            {
                return prefix.len();
            }
        }

        // Try a shorter prefix
        match rfind_delimiter(&prefix) {
            Some(pos) => prefix = prefix[..pos].to_string(),
            None => return 0,
        }
    }
}

/// Transform HIR to use the specified naming conventions with collision handling
#[must_use]
pub fn transform(mut hir: ResolvedGraph, target: &Target) -> ResolvedGraph {
    // Process top-level definitions first (only user definitions, not builtins)
    let top_level_ids: Vec<_> = hir.order.clone();

    rename_breadth(&mut hir, &top_level_ids, None, target);

    // Then recursively process each module's contents
    process_module_contents(&mut hir, target);

    // Process enum constants separately
    process_enum_constants(&mut hir, target);

    hir
}

/// Recursively process all modules, interfaces, and valuetypes and their contents
fn process_module_contents(hir: &mut ResolvedGraph, target: &Target) {
    // Collect all container IDs to process (modules, interfaces, valuetypes)
    let container_ids: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(id, def)| match &def.kind {
            DefKind::Module(m) => Some((id, m.definitions.clone())),
            DefKind::Interface(i) => Some((id, i.definitions.clone())),
            DefKind::Valuetype(v) => Some((id, v.definitions.clone())),
            _ => None,
        })
        .collect();

    // Process each container's children
    for (container_id, child_ids) in container_ids {
        if !child_ids.is_empty() {
            rename_breadth(hir, &child_ids, Some(container_id), target);
        }
    }
}

/// Process enum constants and bitmask flags
fn process_enum_constants(hir: &mut ResolvedGraph, target: &Target) {
    // Strip enum prefixes if enabled
    if target.convention.strip_enum_prefix {
        strip_enum_prefixes(hir);
        strip_bitmask_prefixes(hir);
    }

    // Process enum constants with case conversion
    if target.convention.enumerator.is_some() {
        let enum_constants: Vec<_> = hir
            .context
            .definitions
            .iter()
            .filter_map(|(id, def)| {
                if let DefKind::Enum(e) = &def.kind {
                    Some((id, e.fields.clone()))
                } else {
                    None
                }
            })
            .collect();

        for (enum_id, const_ids) in enum_constants {
            if !const_ids.is_empty() {
                rename_breadth(hir, &const_ids, Some(enum_id), target);
            }
        }
    }

    // Process bitmask flags with case conversion
    if target.convention.bit_flag.is_some() {
        let bitmask_flags: Vec<_> = hir
            .context
            .definitions
            .iter()
            .filter_map(|(id, def)| {
                if let DefKind::Bitmask(b) = &def.kind {
                    Some((id, b.flags.clone()))
                } else {
                    None
                }
            })
            .collect();

        for (bitmask_id, flag_ids) in bitmask_flags {
            if !flag_ids.is_empty() {
                rename_breadth(hir, &flag_ids, Some(bitmask_id), target);
            }
        }
    }
}

/// Strip common prefixes from all enum constants
fn strip_enum_prefixes(hir: &mut ResolvedGraph) {
    let enums: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if let DefKind::Enum(e) = &def.kind {
                Some((def.ident.name.clone(), e.fields.clone()))
            } else {
                None
            }
        })
        .collect();

    for (enum_name, field_ids) in enums {
        let names: Vec<String> = field_ids
            .iter()
            .map(|&id| hir.context.definitions.get(id).ident.name.clone())
            .collect();

        let prefix_len = find_prefix_to_strip(&enum_name, &names);
        if prefix_len > 0 {
            for &id in &field_ids {
                let def = hir.context.definitions.get_mut(id);
                def.ident.name = def.ident.name[prefix_len..].to_string();
            }
        }
    }
}

/// Strip common prefixes from all bitmask flags
fn strip_bitmask_prefixes(hir: &mut ResolvedGraph) {
    let bitmasks: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(_, def)| {
            if let DefKind::Bitmask(b) = &def.kind {
                Some((def.ident.name.clone(), b.flags.clone()))
            } else {
                None
            }
        })
        .collect();

    for (bitmask_name, flag_ids) in bitmasks {
        let names: Vec<String> = flag_ids
            .iter()
            .map(|&id| hir.context.definitions.get(id).ident.name.clone())
            .collect();

        let prefix_len = find_prefix_to_strip(&bitmask_name, &names);
        if prefix_len > 0 {
            for &id in &flag_ids {
                let def = hir.context.definitions.get_mut(id);
                def.ident.name = def.ident.name[prefix_len..].to_string();
            }
        }
    }
}

/// Rename all definitions at the current breadth level with collision handling
/// Check if a constant is an enum constant by checking all enums
fn is_enum_constant(hir: &ResolvedGraph, const_id: hir::DefId) -> bool {
    for (_, def) in &hir.context.definitions {
        if let DefKind::Enum(enum_ty) = &def.kind
            && enum_ty.fields.contains(&const_id)
        {
            return true;
        }
    }
    false
}

fn rename_breadth(
    hir: &mut ResolvedGraph,
    def_ids: &[hir::DefId],
    parent_id: Option<hir::DefId>,
    target: &Target,
) {
    let mut renames = Vec::new();
    let mut module_groups: HashMap<String, Vec<hir::DefId>> = HashMap::new();

    // First, group modules by their original name
    for &id in def_ids {
        let def = hir.context.type_of(id);
        if matches!(def.kind, hir::DefKind::Module(_)) {
            module_groups
                .entry(def.ident.name.clone())
                .or_default()
                .push(id);
        }
    }

    // Collect all definitions at this breadth level, but only one representative per module group
    for &id in def_ids {
        let def = hir.context.type_of(id);

        // Skip non-representative modules
        if let DefKind::Module(_) = &def.kind
            && let Some(group) = module_groups.get(&def.ident.name)
            && group[0] != id
        {
            continue; // Skip non-representative modules
        }

        // Determine the appropriate case and identifier kind for this definition
        let (case, kind) = if matches!(def.kind, hir::DefKind::Const(_)) {
            // Check if this is an enum constant
            if is_enum_constant(hir, id) {
                (target.convention.enumerator, IdentifierKind::Enumerator)
            } else {
                (target.convention.constant, IdentifierKind::Constant)
            }
        } else {
            match &def.kind {
                DefKind::Module(_) => (target.convention.module, IdentifierKind::Module),
                DefKind::Const(_) => (target.convention.constant, IdentifierKind::Constant),
                DefKind::Struct(_) => (target.convention.struct_type, IdentifierKind::Struct),
                DefKind::Union(_) => (target.convention.union_type, IdentifierKind::Union),
                DefKind::Enum(_) => (target.convention.enum_type, IdentifierKind::Enum),
                DefKind::Interface(_) => (target.convention.interface, IdentifierKind::Interface),
                DefKind::Valuetype(_) => (target.convention.valuetype, IdentifierKind::Valuetype),
                DefKind::Alias(_) => (target.convention.alias, IdentifierKind::Alias),
                DefKind::Bitmask(_) => (target.convention.bitmask, IdentifierKind::Bitmask),
                DefKind::Bitset(_) => (target.convention.bitset, IdentifierKind::Bitset),
                DefKind::Except(_) => (target.convention.exception, IdentifierKind::Exception),
                DefKind::Annotation(_) => {
                    (target.convention.annotation, IdentifierKind::Annotation)
                }
                DefKind::Decl(_) => (None, IdentifierKind::Struct), // Decl doesn't matter, won't rename
            }
        };

        let original = def.ident.name.clone();
        let desired = apply_rename(&original, case, kind, target);

        // Add to renames if:
        // 1. The name changed (for any reason: case, preprocessor, or keyword)
        // 2. We're doing case conversion (even if unchanged, for collision detection)
        if original != desired || case.is_some() {
            renames.push(NodeRename {
                def_id: id,
                original,
                desired,
                is_moved: target.moved_defs.contains(&id),
            });
        }
    }

    // Apply collision-aware renaming at this breadth level
    apply_renames_with_collision_handling(hir, &renames, &module_groups);

    // Rename members within each definition at this level
    for &id in def_ids {
        hir.context
            .definitions
            .fold(id, |def| rename_members(target, def));
    }
}

/// Helper to rename a list of items with collision detection
fn rename_items<T, F>(
    items: &mut [T],
    case: Option<Case>,
    kind: IdentifierKind,
    mut get_ident: F,
    target: &Target,
) where
    F: FnMut(&mut T) -> &mut hir::Ident,
{
    // Collect existing names for collision detection
    let mut occupied: HashSet<String> = items
        .iter_mut()
        .map(|item| get_ident(item).name.clone())
        .collect();

    rename_items_with_occupied(items, case, kind, get_ident, &mut occupied, target);
}

/// Helper to rename items using an existing occupied set (for shared namespaces)
fn rename_items_with_occupied<T, F>(
    items: &mut [T],
    case: Option<Case>,
    kind: IdentifierKind,
    mut get_ident: F,
    occupied: &mut HashSet<String>,
    target: &Target,
) where
    F: FnMut(&mut T) -> &mut hir::Ident,
{
    for item in items {
        let ident = get_ident(item);
        let original = ident.name.clone();

        // Apply case conversion and keyword escaping
        let mut desired = apply_rename(&original, case, kind, target);

        // Handle collisions
        while occupied.contains(&desired) && desired != original {
            desired.push('_');
        }

        if desired != original {
            occupied.remove(&original);
            occupied.insert(desired.clone());
            ident.name = desired;
        }
    }
}

/// Rename members, variants, parameters, etc. within a definition
#[allow(clippy::too_many_lines)]
fn rename_members(target: &Target, mut def: hir::Def) -> hir::Def {
    match &mut def.kind {
        DefKind::Struct(s) => {
            rename_items(
                &mut s.members,
                target.convention.member,
                IdentifierKind::Member,
                |m| &mut m.ident,
                target,
            );
        }
        DefKind::Except(e) => {
            rename_items(
                &mut e.members,
                target.convention.member,
                IdentifierKind::Member,
                |m| &mut m.ident,
                target,
            );
        }
        DefKind::Union(u) => {
            rename_items(
                &mut u.variants,
                target.convention.variant,
                IdentifierKind::Variant,
                |v| &mut v.ident,
                target,
            );
        }
        DefKind::Interface(i) => {
            // Operations and attributes share the same namespace
            let mut occupied: HashSet<String> = i
                .prototypes
                .iter()
                .map(|p| p.ident.name.clone())
                .chain(i.attributes.iter().map(|a| a.ident.name.clone()))
                .collect();

            // Rename operations
            rename_items_with_occupied(
                &mut i.prototypes,
                target.convention.operation,
                IdentifierKind::Operation,
                |p| &mut p.ident,
                &mut occupied,
                target,
            );

            // Rename attributes
            rename_items_with_occupied(
                &mut i.attributes,
                target.convention.attribute,
                IdentifierKind::Attribute,
                |a| &mut a.ident,
                &mut occupied,
                target,
            );

            // Rename parameters (no collision detection needed)
            for proto in &mut i.prototypes {
                rename_items(
                    &mut proto.params,
                    target.convention.parameter,
                    IdentifierKind::Parameter,
                    |p| &mut p.ident,
                    target,
                );
            }
        }
        DefKind::Valuetype(v) => {
            // Members, operations, and attributes share the same namespace
            let mut occupied: HashSet<String> = v
                .members
                .iter()
                .map(|m| m.ident.name.clone())
                .chain(v.prototypes.iter().map(|p| p.ident.name.clone()))
                .chain(v.attributes.iter().map(|a| a.ident.name.clone()))
                .collect();

            // Rename members
            rename_items_with_occupied(
                &mut v.members,
                target.convention.member,
                IdentifierKind::Member,
                |m| &mut m.ident,
                &mut occupied,
                target,
            );

            // Rename operations
            rename_items_with_occupied(
                &mut v.prototypes,
                target.convention.operation,
                IdentifierKind::Operation,
                |p| &mut p.ident,
                &mut occupied,
                target,
            );

            // Rename attributes
            rename_items_with_occupied(
                &mut v.attributes,
                target.convention.attribute,
                IdentifierKind::Attribute,
                |a| &mut a.ident,
                &mut occupied,
                target,
            );

            // Rename parameters (no collision detection needed)
            for proto in &mut v.prototypes {
                rename_items(
                    &mut proto.params,
                    target.convention.parameter,
                    IdentifierKind::Parameter,
                    |p| &mut p.ident,
                    target,
                );
            }
        }
        DefKind::Bitset(b) => {
            rename_items(
                &mut b.fields,
                target.convention.bitset_field,
                IdentifierKind::BitsetField,
                |f| &mut f.ident,
                target,
            );
        }
        _ => {}
    }

    def
}

/// Categorize nodes by their rename priority
fn categorize_renames(
    renames: &[NodeRename],
) -> (Vec<&NodeRename>, Vec<&NodeRename>, Vec<&NodeRename>) {
    let mut priority1 = Vec::new(); // Nodes that want to keep their original name
    let mut priority2 = Vec::new(); // Nodes that want to change their name
    let mut moved_nodes = Vec::new(); // Moved nodes (lowest priority)

    for rename in renames {
        if rename.is_moved {
            moved_nodes.push(rename);
        } else if rename.desired == rename.original {
            priority1.push(rename);
        } else {
            priority2.push(rename);
        }
    }

    (priority1, priority2, moved_nodes)
}

/// Process priority 2 renames with chain substitution
fn process_priority2_renames(
    priority2: &[&NodeRename],
    final_assignments: &mut HashMap<hir::DefId, String>,
    occupied: &mut HashSet<String>,
    will_keep_original: &HashSet<String>,
) {
    let mut to_process: Vec<&NodeRename> = priority2.to_vec();
    let mut vacated: HashSet<String> = HashSet::new();

    while !to_process.is_empty() {
        let mut deferred = Vec::new();
        let mut made_progress = false;

        for rename in to_process {
            if final_assignments.contains_key(&rename.def_id) {
                continue;
            }

            // Check if this node should keep its original name
            if will_keep_original.contains(&rename.original) {
                final_assignments.insert(rename.def_id, rename.original.clone());
                made_progress = true;
                continue;
            }

            let target = &rename.desired;

            // Try to get desired name if it's available
            if !occupied.contains(target) && !will_keep_original.contains(target) {
                final_assignments.insert(rename.def_id, target.clone());
                occupied.insert(target.clone());
                if target != &rename.original && !will_keep_original.contains(&rename.original) {
                    vacated.insert(rename.original.clone());
                }
                made_progress = true;
            }
            // Try to use a vacated name
            else if vacated.contains(target) {
                final_assignments.insert(rename.def_id, target.clone());
                occupied.insert(target.clone());
                vacated.remove(target);
                if target != &rename.original && !will_keep_original.contains(&rename.original) {
                    vacated.insert(rename.original.clone());
                }
                made_progress = true;
            } else {
                deferred.push(rename);
            }
        }

        // If no progress, escape one node to break deadlock
        if !made_progress && !deferred.is_empty() {
            let rename = deferred[0];
            let mut name = rename.desired.clone();

            // Find an available escaped name
            while occupied.contains(&name) || will_keep_original.contains(&name) {
                name.push('_');
            }

            final_assignments.insert(rename.def_id, name.clone());
            occupied.insert(name.clone());
            if !will_keep_original.contains(&rename.original) {
                vacated.insert(rename.original.clone());
            }

            deferred.remove(0);
        }

        to_process = deferred;
    }
}

/// Apply computed renames to definitions
fn apply_final_renames(
    hir: &mut ResolvedGraph,
    final_assignments: &HashMap<hir::DefId, String>,
    renames: &[NodeRename],
    module_groups: &HashMap<String, Vec<hir::DefId>>,
) {
    for (def_id, new_name) in final_assignments {
        let def = hir.context.type_of(*def_id);
        if let DefKind::Module(_) = &def.kind {
            // Find the original name to look up the group
            let original_name = renames
                .iter()
                .find(|r| r.def_id == *def_id)
                .map(|r| &r.original)
                .unwrap();

            if let Some(group_ids) = module_groups.get(original_name) {
                // Apply the same name to all modules in the group
                for &module_id in group_ids {
                    hir.context
                        .definitions
                        .get_mut(module_id)
                        .ident
                        .name
                        .clone_from(new_name);
                }
            }
        } else {
            hir.context
                .definitions
                .get_mut(*def_id)
                .ident
                .name
                .clone_from(new_name);
        }
    }
}

/// Apply renames to top-level definitions with collision handling
fn apply_renames_with_collision_handling(
    hir: &mut ResolvedGraph,
    renames: &[NodeRename],
    module_groups: &HashMap<String, Vec<hir::DefId>>,
) {
    // Categorize nodes by priority
    let (priority1, priority2, moved_nodes) = categorize_renames(renames);

    let mut final_assignments: HashMap<hir::DefId, String> = HashMap::new();
    let mut occupied: HashSet<String> = HashSet::new();

    // Process priority 1: nodes that want to keep their original name
    for rename in &priority1 {
        final_assignments.insert(rename.def_id, rename.original.clone());
        occupied.insert(rename.original.clone());
    }

    // Process moved nodes: they must get unique names
    for rename in &moved_nodes {
        let mut name = rename.desired.clone();
        while occupied.contains(&name) {
            name.push('_');
        }
        final_assignments.insert(rename.def_id, name.clone());
        occupied.insert(name);
    }

    // Determine which priority2 nodes should keep their original names
    let mut will_keep_original: HashSet<String> = HashSet::new();
    for rename in &priority2 {
        if occupied.contains(&rename.desired)
            && !occupied.contains(&rename.original)
            && is_natural_fallback(&rename.original, &rename.desired)
        {
            will_keep_original.insert(rename.original.clone());
        }
    }

    // Mark natural fallback names as occupied
    for name in &will_keep_original {
        occupied.insert(name.clone());
    }

    // Process priority2 nodes with chain substitution support
    process_priority2_renames(
        &priority2,
        &mut final_assignments,
        &mut occupied,
        &will_keep_original,
    );

    // Apply all the renames
    apply_final_renames(hir, &final_assignments, renames, module_groups);
}
