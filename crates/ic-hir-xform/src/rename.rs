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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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
use ic_hir::visit::Visitor;
use ic_hir::{Context, ResolvedGraph, hir};

/// Function type for preprocessing names before case conversion
pub type NamePreprocessor = fn(&str) -> String;

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

/// Defines the naming convention to use for types of a specific kind.
///
/// If there are specific language items that should not be renamed, setting
/// the corresponding field to `None` will prevent the transformation from
/// renaming them.
#[derive(Clone)]
pub struct Target {
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

    /// Annotation parameters
    pub annotation_param: Option<Case>,

    /// Optional preprocessor function to apply to names before case conversion
    /// If None, names are used as-is
    pub name_preprocessor: Option<NamePreprocessor>,

    /// Set of keywords that should be escaped
    /// If an identifier matches a keyword, the escape function will be applied
    pub keywords: HashSet<&'static str>,

    /// Function to apply when escaping keywords
    /// Default: append underscore
    pub keyword_escape_fn: fn(&str) -> String,

    /// Set of `DefIds` that were moved by previous transformations
    /// These will have lower priority in collision resolution
    pub moved_defs: HashSet<hir::DefId>,
}

fn default_keyword_escape(name: &str) -> String {
    format!("{name}_")
}

impl Default for Target {
    fn default() -> Self {
        Self {
            struct_type: None,
            union_type: None,
            enum_type: None,
            interface: None,
            valuetype: None,
            alias: None,
            bitmask: None,
            bitset: None,
            exception: None,
            annotation: None,
            member: None,
            variant: None,
            enumerator: None,
            bit_flag: None,
            bitset_field: None,
            constant: None,
            module: None,
            operation: None,
            attribute: None,
            parameter: None,
            annotation_param: None,
            name_preprocessor: None,
            keywords: HashSet::new(),
            keyword_escape_fn: default_keyword_escape,
            moved_defs: HashSet::new(),
        }
    }
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

struct Renamer {
    target: Target,
    renamed_idents: HashMap<String, String>,
}

/// Apply case conversion and keyword escaping to a name
fn apply_rename(name: &str, case: Option<Case>, target: &Target) -> String {
    let mut new_name = name.to_string();

    // First apply case conversion if specified
    if let Some(case) = case {
        new_name = case::convert(&new_name, case);
    }

    // Then check if the result is a keyword and escape it
    if target.keywords.contains(new_name.as_str()) {
        new_name = (target.keyword_escape_fn)(&new_name);
    }

    new_name
}

impl Renamer {
    fn new(target: Target) -> Self {
        Self {
            target,
            renamed_idents: HashMap::new(),
        }
    }

    fn rename_ident(&mut self, ident: &mut hir::Ident, case: Option<Case>) {
        let old_name = ident.name.clone();
        let new_name = apply_rename(&old_name, case, &self.target);

        if old_name != new_name {
            self.renamed_idents.insert(old_name, new_name.clone());
            ident.name = new_name;
        }
    }
}

/// Transform HIR to use the specified naming conventions with collision handling
#[must_use]
pub fn transform(mut hir: ResolvedGraph, target: &Target) -> ResolvedGraph {
    // Process top-level definitions first
    let top_level_ids: Vec<_> = hir
        .order
        .iter()
        .chain(hir.builtin_order.iter())
        .copied()
        .collect();

    rename_breadth(&mut hir, &top_level_ids, None, target);

    // Then recursively process each module's contents
    process_module_contents(&mut hir, target);

    // Process enum constants separately
    process_enum_constants(&mut hir, target);

    // Finally, rename members, variants, and other nested identifiers using Fold
    let mut renamer = Renamer::new(target.clone());
    let all_def_ids: Vec<_> = hir.context.definitions.iter().map(|(id, _)| id).collect();
    for def_id in all_def_ids {
        let def = hir.context.definitions.get_mut(def_id);
        match &mut def.kind {
            DefKind::Struct(s) => {
                for member in &mut s.members {
                    renamer.rename_ident(&mut member.ident, target.member);
                }
            }
            DefKind::Union(u) => {
                for variant in &mut u.variants {
                    renamer.rename_ident(&mut variant.ident, target.variant);
                }
            }
            DefKind::Except(e) => {
                for member in &mut e.members {
                    renamer.rename_ident(&mut member.ident, target.member);
                }
            }
            DefKind::Valuetype(v) => {
                for member in &mut v.members {
                    renamer.rename_ident(&mut member.ident, target.member);
                }
                for proto in &mut v.prototypes {
                    renamer.rename_ident(&mut proto.ident, target.operation);
                    for param in &mut proto.params {
                        renamer.rename_ident(&mut param.ident, target.parameter);
                    }
                }
                for attr in &mut v.attributes {
                    renamer.rename_ident(&mut attr.ident, target.attribute);
                }
            }
            DefKind::Interface(i) => {
                for proto in &mut i.prototypes {
                    renamer.rename_ident(&mut proto.ident, target.operation);
                    for param in &mut proto.params {
                        renamer.rename_ident(&mut param.ident, target.parameter);
                    }
                }
                for attr in &mut i.attributes {
                    renamer.rename_ident(&mut attr.ident, target.attribute);
                }
            }
            DefKind::Bitset(b) => {
                for field in &mut b.fields {
                    renamer.rename_ident(&mut field.ident, target.bitset_field);
                }
            }
            DefKind::Annotation(a) => {
                for param in &mut a.params {
                    renamer.rename_ident(&mut param.ident, target.annotation_param);
                }
            }
            _ => {}
        }
    }

    hir
}

/// Recursively process all modules and their contents
fn process_module_contents(hir: &mut ResolvedGraph, target: &Target) {
    // Collect all module IDs to process
    let module_ids: Vec<_> = hir
        .context
        .definitions
        .iter()
        .filter_map(|(id, def)| {
            if let DefKind::Module(m) = &def.kind {
                Some((id, m.definitions.clone()))
            } else {
                None
            }
        })
        .collect();

    // Process each module's children
    for (module_id, child_ids) in module_ids {
        if !child_ids.is_empty() {
            rename_breadth(hir, &child_ids, Some(module_id), target);
        }
    }
}

/// Process enum constants
fn process_enum_constants(hir: &mut ResolvedGraph, target: &Target) {
    // Only process if we have a target for enumerators
    if let Some(case) = target.enumerator {
        // Collect all enum constants
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

        // Process each enum's constants
        for (enum_id, const_ids) in enum_constants {
            if !const_ids.is_empty() {
                rename_breadth(hir, &const_ids, Some(enum_id), target);
            }
        }
    }
}

/// Rename all definitions at the current breadth level with collision handling
/// Check if a constant is an enum constant by checking all enums
fn is_enum_constant(hir: &ResolvedGraph, const_id: hir::DefId) -> bool {
    for (_, def) in &hir.context.definitions {
        if let DefKind::Enum(enum_ty) = &def.kind {
            if enum_ty.fields.contains(&const_id) {
                return true;
            }
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
        if let DefKind::Module(_) = &def.kind {
            if let Some(group) = module_groups.get(&def.ident.name) {
                if group[0] != id {
                    continue; // Skip non-representative modules
                }
            }
        }

        // Determine the appropriate case for this definition
        let case = if matches!(def.kind, hir::DefKind::Const(_)) {
            // Check if this is an enum constant
            if is_enum_constant(hir, id) {
                target.enumerator
            } else {
                target.constant
            }
        } else {
            match &def.kind {
                DefKind::Module(_) => target.module,
                DefKind::Const(_) => target.constant,
                DefKind::Struct(_) => target.struct_type,
                DefKind::Union(_) => target.union_type,
                DefKind::Enum(_) => target.enum_type,
                DefKind::Interface(_) => target.interface,
                DefKind::Valuetype(_) => target.valuetype,
                DefKind::Alias(_) => target.alias,
                DefKind::Bitmask(_) => target.bitmask,
                DefKind::Bitset(_) => target.bitset,
                DefKind::Except(_) => target.exception,
                DefKind::Annotation(_) => target.annotation,
                DefKind::Decl(_) => None,
            }
        };

        if let Some(case) = case {
            let original = def.ident.name.clone();

            // Apply preprocessor before case conversion if provided
            let preprocessed = if let Some(preprocessor) = target.name_preprocessor {
                preprocessor(&original)
            } else {
                original.clone()
            };
            let desired = case::convert(&preprocessed, case);

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
fn rename_items<T, F>(items: &mut [T], case: Option<Case>, mut get_ident: F, target: &Target)
where
    F: FnMut(&mut T) -> &mut hir::Ident,
{
    if let Some(case) = case {
        // Collect existing names for collision detection
        let mut occupied: HashSet<String> = items
            .iter_mut()
            .map(|item| get_ident(item).name.clone())
            .collect();

        rename_items_with_occupied(items, Some(case), get_ident, &mut occupied, target);
    }
}

/// Helper to rename items using an existing occupied set (for shared namespaces)
fn rename_items_with_occupied<T, F>(
    items: &mut [T],
    case: Option<Case>,
    mut get_ident: F,
    occupied: &mut HashSet<String>,
    target: &Target,
) where
    F: FnMut(&mut T) -> &mut hir::Ident,
{
    if let Some(case) = case {
        for item in items {
            let ident = get_ident(item);
            let original = ident.name.clone();

            // Apply case conversion and keyword escaping
            let mut desired = apply_rename(&original, Some(case), target);

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
}

/// Rename members, variants, parameters, etc. within a definition
fn rename_members(target: &Target, mut def: hir::Def) -> hir::Def {
    match &mut def.kind {
        DefKind::Struct(s) => {
            rename_items(&mut s.members, target.member, |m| &mut m.ident, target);
        }
        DefKind::Except(e) => {
            rename_items(&mut e.members, target.member, |m| &mut m.ident, target);
        }
        DefKind::Union(u) => {
            rename_items(&mut u.variants, target.variant, |v| &mut v.ident, target);
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
                target.operation,
                |p| &mut p.ident,
                &mut occupied,
                target,
            );

            // Rename attributes
            rename_items_with_occupied(
                &mut i.attributes,
                target.attribute,
                |a| &mut a.ident,
                &mut occupied,
                target,
            );

            // Rename parameters (no collision detection needed)
            for proto in &mut i.prototypes {
                rename_items(
                    &mut proto.params,
                    target.parameter,
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
                target.member,
                |m| &mut m.ident,
                &mut occupied,
                target,
            );

            // Rename operations
            rename_items_with_occupied(
                &mut v.prototypes,
                target.operation,
                |p| &mut p.ident,
                &mut occupied,
                target,
            );

            // Rename attributes
            rename_items_with_occupied(
                &mut v.attributes,
                target.attribute,
                |a| &mut a.ident,
                &mut occupied,
                target,
            );

            // Rename parameters (no collision detection needed)
            for proto in &mut v.prototypes {
                rename_items(
                    &mut proto.params,
                    target.parameter,
                    |p| &mut p.ident,
                    target,
                );
            }
        }
        DefKind::Bitset(b) => {
            rename_items(&mut b.fields, target.bitset_field, |f| &mut f.ident, target);
        }
        DefKind::Annotation(a) => {
            rename_items(
                &mut a.params,
                target.annotation_param,
                |p| &mut p.ident,
                target,
            );
        }
        _ => {
            // Bitmask flags are DefIds - handled separately
            // Other def kinds don't have renameable members
        }
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
