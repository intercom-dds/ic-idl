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
use ic_hir::fold::Fold;
use ic_hir::visit::Visitor;
use ic_hir::{Context, ResolvedGraph, hir};

/// Function type for preprocessing names before case conversion
pub type NamePreprocessor = fn(&str) -> String;

/// Preprocessor that strips common suffixes like _t and _e
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
#[derive(Clone, Default)]
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

    /// Set of DefIds that were moved by previous transformations
    /// These will have lower priority in collision resolution
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

/// Check if original is a natural fallback for desired (e.g., FooBar_ for FooBar)
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

impl Renamer {
    fn new(target: Target) -> Self {
        Self {
            target,
            renamed_idents: HashMap::new(),
        }
    }

    fn rename_ident(&mut self, ident: &mut hir::Ident, case: Option<Case>) {
        if let Some(case) = case {
            let old_name = ident.name.clone();
            let new_name = case::convert(&old_name, case);
            if old_name != new_name {
                self.renamed_idents.insert(old_name, new_name.clone());
                ident.name = new_name;
            }
        }
    }

    /// Get the desired case for a definition
    fn get_def_case(&self, def: &hir::Def) -> Option<Case> {
        match &def.kind {
            hir::DefKind::Module(_) => self.target.module,
            hir::DefKind::Const(_) => {
                // For now, we'll handle enum constants separately
                // by checking the parent when we have access to the context
                self.target.constant
            }
            hir::DefKind::Struct(_) => self.target.struct_type,
            hir::DefKind::Union(_) => self.target.union_type,
            hir::DefKind::Enum(_) => self.target.enum_type,
            hir::DefKind::Interface(_) => self.target.interface,
            hir::DefKind::Valuetype(_) => self.target.valuetype,
            hir::DefKind::Alias(_) => self.target.alias,
            hir::DefKind::Bitmask(_) => self.target.bitmask,
            hir::DefKind::Bitset(_) => self.target.bitset,
            hir::DefKind::Except(_) => self.target.exception,
            hir::DefKind::Annotation(_) => self.target.annotation,
            hir::DefKind::Decl(_) => None, // Don't rename forward declarations
        }
    }
}

impl Fold for Renamer {
    fn fold_def(&mut self, def: hir::Def) -> hir::Def {
        // Don't rename the def itself here - that's handled by the collision-aware rename
        // Just fold the contents
        ic_hir::fold::fold_def(self, def)
    }

    fn fold_struct_ty(&mut self, mut s: hir::StructTy) -> hir::StructTy {
        s.members = s.members.into_iter().map(|m| self.fold_member(m)).collect();
        s
    }

    fn fold_except_ty(&mut self, mut e: hir::ExceptTy) -> hir::ExceptTy {
        e.members = e.members.into_iter().map(|m| self.fold_member(m)).collect();
        e
    }

    fn fold_union_ty(&mut self, mut u: hir::UnionTy) -> hir::UnionTy {
        u.variants = u
            .variants
            .into_iter()
            .map(|v| self.fold_variant(v))
            .collect();
        u
    }

    fn fold_enum_ty(&mut self, e: hir::EnumTy) -> hir::EnumTy {
        // Enum constants are separate definitions that will be renamed
        // when we process all definitions in the transform function
        ic_hir::fold::fold_enum_ty(self, e)
    }

    fn fold_bitmask_ty(&mut self, b: hir::BitmaskTy) -> hir::BitmaskTy {
        // Bitmask flags are now DefIds - they will be renamed
        // when we process all definitions in the transform function
        ic_hir::fold::fold_bitmask_ty(self, b)
    }

    fn fold_bitset_ty(&mut self, mut b: hir::BitsetTy) -> hir::BitsetTy {
        for field in &mut b.fields {
            self.rename_ident(&mut field.ident, self.target.bitset_field);
        }
        ic_hir::fold::fold_bitset_ty(self, b)
    }

    fn fold_interface_ty(&mut self, mut i: hir::InterfaceTy) -> hir::InterfaceTy {
        i.prototypes = i
            .prototypes
            .into_iter()
            .map(|p| self.fold_proto_ty(p))
            .collect();
        i.attributes = i
            .attributes
            .into_iter()
            .map(|a| self.fold_attribute(a))
            .collect();
        ic_hir::fold::fold_interface_ty(self, i)
    }

    fn fold_valuetype(&mut self, mut v: hir::ValueTy) -> hir::ValueTy {
        v.members = v.members.into_iter().map(|m| self.fold_member(m)).collect();
        v.prototypes = v
            .prototypes
            .into_iter()
            .map(|p| self.fold_proto_ty(p))
            .collect();
        v.attributes = v
            .attributes
            .into_iter()
            .map(|a| self.fold_attribute(a))
            .collect();
        ic_hir::fold::fold_valuetype(self, v)
    }

    fn fold_annotation_ty(&mut self, mut a: hir::AnnotationTy) -> hir::AnnotationTy {
        for param in &mut a.params {
            self.rename_ident(&mut param.ident, self.target.annotation_param);
        }
        ic_hir::fold::fold_annotation_ty(self, a)
    }

    fn fold_member(&mut self, mut m: hir::Member) -> hir::Member {
        self.rename_ident(&mut m.ident, self.target.member);
        ic_hir::fold::fold_member(self, m)
    }

    fn fold_variant(&mut self, mut v: hir::Variant) -> hir::Variant {
        self.rename_ident(&mut v.ident, self.target.variant);
        ic_hir::fold::fold_variant(self, v)
    }

    fn fold_proto_ty(&mut self, mut p: hir::ProtoTy) -> hir::ProtoTy {
        self.rename_ident(&mut p.ident, self.target.operation);
        p.params = p
            .params
            .into_iter()
            .map(|param| self.fold_parameter(param))
            .collect();
        ic_hir::fold::fold_proto_ty(self, p)
    }

    fn fold_parameter(&mut self, mut p: hir::Parameter) -> hir::Parameter {
        self.rename_ident(&mut p.ident, self.target.parameter);
        ic_hir::fold::fold_parameter(self, p)
    }

    fn fold_attribute(&mut self, mut a: hir::Attribute) -> hir::Attribute {
        self.rename_ident(&mut a.ident, self.target.attribute);
        ic_hir::fold::fold_attribute(self, a)
    }

    fn fold_const_ty(&mut self, c: hir::ConstTy) -> hir::ConstTy {
        // We need to fold the constant type to handle any references
        // to renamed identifiers in the constant value
        ic_hir::fold::fold_const_ty(self, c)
    }

    fn fold_numeric(&mut self, n: hir::Numeric) -> hir::Numeric {
        // Handle Numeric::Const references that might point to renamed definitions
        match n {
            hir::Numeric::Const(def_id) => {
                // The def_id itself doesn't change, but when the constant is
                // evaluated, it will use the renamed identifier
                hir::Numeric::Const(def_id)
            }
            _ => ic_hir::fold::fold_numeric(self, n),
        }
    }
}

/// Transform HIR to use the specified naming conventions with collision handling
#[must_use]
pub fn transform(mut hir: ResolvedGraph, target: Target) -> ResolvedGraph {
    // Process top-level definitions first
    let top_level_ids: Vec<_> = hir
        .order
        .iter()
        .chain(hir.builtin_order.iter())
        .copied()
        .collect();

    rename_breadth(&mut hir, &top_level_ids, None, &target);

    // Then recursively process each module's contents
    process_module_contents(&mut hir, &target);

    // Process enum constants separately
    process_enum_constants(&mut hir, &target);

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
            if let hir::DefKind::Module(m) = &def.kind {
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
                if let hir::DefKind::Enum(e) = &def.kind {
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
    for (_, def) in hir.context.definitions.iter() {
        if let hir::DefKind::Enum(enum_ty) = &def.kind {
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
        if let hir::DefKind::Module(_) = &def.kind {
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
                hir::DefKind::Module(_) => target.module,
                hir::DefKind::Const(_) => target.constant,
                hir::DefKind::Struct(_) => target.struct_type,
                hir::DefKind::Union(_) => target.union_type,
                hir::DefKind::Enum(_) => target.enum_type,
                hir::DefKind::Interface(_) => target.interface,
                hir::DefKind::Valuetype(_) => target.valuetype,
                hir::DefKind::Alias(_) => target.alias,
                hir::DefKind::Bitmask(_) => target.bitmask,
                hir::DefKind::Bitset(_) => target.bitset,
                hir::DefKind::Except(_) => target.exception,
                hir::DefKind::Annotation(_) => target.annotation,
                hir::DefKind::Decl(_) => None,
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
            .fold(id, |def| rename_members(&target, def));
    }
}

/// Helper to rename a list of items with collision detection
fn rename_items<T, F>(items: &mut [T], case: Option<Case>, mut get_ident: F)
where
    F: FnMut(&mut T) -> &mut hir::Ident,
{
    if let Some(case) = case {
        // Collect existing names for collision detection
        let mut occupied: HashSet<String> = items
            .iter_mut()
            .map(|item| get_ident(item).name.clone())
            .collect();

        rename_items_with_occupied(items, Some(case), get_ident, &mut occupied);
    }
}

/// Helper to rename items using an existing occupied set (for shared namespaces)
fn rename_items_with_occupied<T, F>(
    items: &mut [T],
    case: Option<Case>,
    mut get_ident: F,
    occupied: &mut HashSet<String>,
) where
    F: FnMut(&mut T) -> &mut hir::Ident,
{
    if let Some(case) = case {
        for item in items {
            let ident = get_ident(item);
            let original = ident.name.clone();
            let mut desired = case::convert(&original, case);

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
        hir::DefKind::Struct(s) => {
            rename_items(&mut s.members, target.member, |m| &mut m.ident);
        }
        hir::DefKind::Except(e) => {
            rename_items(&mut e.members, target.member, |m| &mut m.ident);
        }
        hir::DefKind::Union(u) => {
            rename_items(&mut u.variants, target.variant, |v| &mut v.ident);
        }
        hir::DefKind::Interface(i) => {
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
            );

            // Rename attributes
            rename_items_with_occupied(
                &mut i.attributes,
                target.attribute,
                |a| &mut a.ident,
                &mut occupied,
            );

            // Rename parameters (no collision detection needed)
            for proto in &mut i.prototypes {
                rename_items(&mut proto.params, target.parameter, |p| &mut p.ident);
            }
        }
        hir::DefKind::Valuetype(v) => {
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
            );

            // Rename operations
            rename_items_with_occupied(
                &mut v.prototypes,
                target.operation,
                |p| &mut p.ident,
                &mut occupied,
            );

            // Rename attributes
            rename_items_with_occupied(
                &mut v.attributes,
                target.attribute,
                |a| &mut a.ident,
                &mut occupied,
            );

            // Rename parameters (no collision detection needed)
            for proto in &mut v.prototypes {
                rename_items(&mut proto.params, target.parameter, |p| &mut p.ident);
            }
        }
        hir::DefKind::Bitmask(_) => {
            // Bitmask flags are DefIds - handled separately
        }
        hir::DefKind::Bitset(b) => {
            rename_items(&mut b.fields, target.bitset_field, |f| &mut f.ident);
        }
        hir::DefKind::Annotation(a) => {
            rename_items(&mut a.params, target.annotation_param, |p| &mut p.ident);
        }
        _ => {}
    }

    def
}

/// Apply renames to top-level definitions with collision handling
fn apply_renames_with_collision_handling(
    hir: &mut ResolvedGraph,
    renames: &[NodeRename],
    module_groups: &HashMap<String, Vec<hir::DefId>>,
) {
    // Categorize nodes by priority
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
        if occupied.contains(&rename.desired) && !occupied.contains(&rename.original) {
            if is_natural_fallback(&rename.original, &rename.desired) {
                will_keep_original.insert(rename.original.clone());
            }
        }
    }

    // Mark natural fallback names as occupied
    for name in &will_keep_original {
        occupied.insert(name.clone());
    }

    // Process priority2 nodes with chain substitution support
    let mut to_process: Vec<&NodeRename> = priority2.clone();
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
            occupied.insert(name);
            if !will_keep_original.contains(&rename.original) {
                vacated.insert(rename.original.clone());
            }

            deferred.remove(0);
        }

        to_process = deferred;
    }

    // Apply all the renames
    for (def_id, new_name) in &final_assignments {
        // Check if this is a module and apply to all instances in its group
        let def = hir.context.type_of(*def_id);
        if let hir::DefKind::Module(_) = &def.kind {
            // Find the original name to look up the group
            let original_name = renames
                .iter()
                .find(|r| r.def_id == *def_id)
                .map(|r| &r.original)
                .unwrap();

            if let Some(group_ids) = module_groups.get(original_name) {
                // Apply the same name to all modules in the group
                for &module_id in group_ids {
                    hir.context.definitions.get_mut(module_id).ident.name = new_name.clone();
                }
            }
        } else {
            hir.context.definitions.get_mut(*def_id).ident.name = new_name.clone();
        }
    }
}
