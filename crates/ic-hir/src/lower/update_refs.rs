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

//! Post-processing phase to update type references from forward declarations to definitions.
//!
//! This phase runs after resolution and evaluation, updating any type references that
//! point to forward declarations to instead point to the actual definitions when available.

use std::collections::HashMap;

use crate::Context;
use crate::hir::{DefFlags, DefId, DefKind, Decl, Ty, TyKind};

/// Updates type references from forward declarations to their corresponding definitions.
pub fn update_forward_references(ctx: &mut Context) {
    // Build a mapping from forward declarations to their definitions
    let forward_to_def = build_forward_to_definition_map(ctx);
    
    // Update all type references in the HIR
    update_all_type_references(ctx, &forward_to_def);
}

/// Builds a mapping from forward declaration DefIds to their corresponding definition DefIds.
fn build_forward_to_definition_map(ctx: &Context) -> HashMap<DefId, DefId> {
    let mut forward_to_def = HashMap::new();
    
    // Group definitions by name and kind
    let mut by_name: HashMap<(String, DeclKind), Vec<DefId>> = HashMap::new();
    
    for (def_id, def) in &ctx.definitions {
        let key = match &def.kind {
            DefKind::Decl(decl_kind) => {
                // This is a forward declaration
                (def.ident.name.clone(), DeclKind::from(*decl_kind))
            }
            DefKind::Struct(_) => (def.ident.name.clone(), DeclKind::Struct),
            DefKind::Union(_) => (def.ident.name.clone(), DeclKind::Union),
            DefKind::Interface(_) => (def.ident.name.clone(), DeclKind::Interface),
            DefKind::Valuetype(_) => (def.ident.name.clone(), DeclKind::Valuetype),
            _ => continue, // Skip other kinds
        };
        
        by_name.entry(key).or_default().push(def_id);
    }
    
    // For each name, find forward declarations and their corresponding definitions
    for ((_name, kind), def_ids) in by_name {
        let mut forward_decls = Vec::new();
        let mut definitions = Vec::new();
        
        for &def_id in &def_ids {
            let def = ctx.definitions.get(def_id);
            if def.flags.contains(DefFlags::IS_INCOMPLETE) {
                forward_decls.push(def_id);
            } else if matches_kind(&def.kind, kind) {
                definitions.push(def_id);
            }
        }
        
        // If we have both forward declarations and definitions, create mappings
        // We map all forward declarations to the first definition found
        if let Some(&definition_id) = definitions.first() {
            for &forward_id in &forward_decls {
                forward_to_def.insert(forward_id, definition_id);
            }
        }
    }
    
    forward_to_def
}

/// Enum to represent the kind of declaration for matching purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DeclKind {
    Struct,
    Union,
    Interface,
    Valuetype,
}

impl From<Decl> for DeclKind {
    fn from(decl: Decl) -> Self {
        match decl {
            Decl::Struct => DeclKind::Struct,
            Decl::Union => DeclKind::Union,
            Decl::Interface => DeclKind::Interface,
            Decl::Valuetype => DeclKind::Valuetype,
            Decl::Native => DeclKind::Struct, // Native is treated as struct for matching
        }
    }
}

/// Checks if a DefKind matches the expected DeclKind.
fn matches_kind(def_kind: &DefKind, decl_kind: DeclKind) -> bool {
    match (def_kind, decl_kind) {
        (DefKind::Struct(_), DeclKind::Struct) => true,
        (DefKind::Union(_), DeclKind::Union) => true,
        (DefKind::Interface(_), DeclKind::Interface) => true,
        (DefKind::Valuetype(_), DeclKind::Valuetype) => true,
        _ => false,
    }
}

/// Updates all type references in the HIR to point to definitions instead of forward declarations.
fn update_all_type_references(ctx: &mut Context, forward_to_def: &HashMap<DefId, DefId>) {
    // Clone the definition IDs to avoid borrowing issues
    let def_ids: Vec<DefId> = ctx.definitions.iter().map(|(id, _)| id).collect();
    
    for def_id in def_ids {
        update_definition_type_references(ctx, def_id, forward_to_def);
    }
}

/// Updates type references within a single definition.
fn update_definition_type_references(
    ctx: &mut Context,
    def_id: DefId,
    forward_to_def: &HashMap<DefId, DefId>,
) {
    // Get a mutable reference to the definition
    let def = ctx.definitions.get_mut(def_id);
    
    match &mut def.kind {
        DefKind::Struct(struct_ty) => {
            // Update parent reference
            if let Some(parent_id) = &mut struct_ty.parent {
                if let Some(&new_id) = forward_to_def.get(parent_id) {
                    *parent_id = new_id;
                }
            }
            
            // Update member types
            for member in &mut struct_ty.members {
                update_type(&mut member.ty, forward_to_def);
            }
        }
        DefKind::Union(union_ty) => {
            // Update discriminator type
            update_type(&mut union_ty.disc, forward_to_def);
            
            // Update variant types
            for variant in &mut union_ty.variants {
                update_type(&mut variant.ty, forward_to_def);
            }
        }
        DefKind::Interface(interface_ty) => {
            // Update parent references
            for parent_id in &mut interface_ty.parents {
                if let Some(&new_id) = forward_to_def.get(parent_id) {
                    *parent_id = new_id;
                }
            }
            
            // Update prototype types
            for proto in &mut interface_ty.prototypes {
                update_type(&mut proto.ty, forward_to_def);
                for param in &mut proto.params {
                    update_type(&mut param.ty, forward_to_def);
                }
            }
            
            // Update attribute types
            for attr in &mut interface_ty.attributes {
                update_type(&mut attr.ty, forward_to_def);
                
                // Update exception references
                for raises_id in &mut attr.getraises {
                    if let Some(&new_id) = forward_to_def.get(raises_id) {
                        *raises_id = new_id;
                    }
                }
                for raises_id in &mut attr.setraises {
                    if let Some(&new_id) = forward_to_def.get(raises_id) {
                        *raises_id = new_id;
                    }
                }
            }
        }
        DefKind::Valuetype(valuetype_ty) => {
            // Update parent reference
            if let Some(parent_id) = &mut valuetype_ty.parent {
                if let Some(&new_id) = forward_to_def.get(parent_id) {
                    *parent_id = new_id;
                }
            }
            
            // Update member types
            for member in &mut valuetype_ty.members {
                update_type(&mut member.ty, forward_to_def);
            }
            
            // Update prototype types
            for proto in &mut valuetype_ty.prototypes {
                update_type(&mut proto.ty, forward_to_def);
                for param in &mut proto.params {
                    update_type(&mut param.ty, forward_to_def);
                }
            }
            
            // Update attribute types
            for attr in &mut valuetype_ty.attributes {
                update_type(&mut attr.ty, forward_to_def);
                
                // Update exception references
                for raises_id in &mut attr.getraises {
                    if let Some(&new_id) = forward_to_def.get(raises_id) {
                        *raises_id = new_id;
                    }
                }
                for raises_id in &mut attr.setraises {
                    if let Some(&new_id) = forward_to_def.get(raises_id) {
                        *raises_id = new_id;
                    }
                }
            }
        }
        DefKind::Alias(alias_ty) => {
            update_type(&mut alias_ty.ty, forward_to_def);
        }
        DefKind::Except(except_ty) => {
            for member in &mut except_ty.members {
                update_type(&mut member.ty, forward_to_def);
            }
        }
        DefKind::Const(const_ty) => {
            update_type(&mut const_ty.ty, forward_to_def);
        }
        DefKind::Annotation(ann_ty) => {
            for param in &mut ann_ty.params {
                update_type(&mut param.ty, forward_to_def);
            }
        }
        DefKind::Bitset(bitset_ty) => {
            // Update parent reference
            if let Some(parent_id) = &mut bitset_ty.parent {
                if let Some(&new_id) = forward_to_def.get(parent_id) {
                    *parent_id = new_id;
                }
            }
            
            // Update field types
            for field in &mut bitset_ty.fields {
                update_type(&mut field.ty, forward_to_def);
            }
        }
        _ => {
            // Other kinds don't have type references to update
        }
    }
}

/// Updates type references within a type.
fn update_type(ty: &mut Ty, forward_to_def: &HashMap<DefId, DefId>) {
    match &mut ty.kind {
        TyKind::Adt(def_id) => {
            // This is the main case - update ADT references
            if let Some(&new_id) = forward_to_def.get(def_id) {
                *def_id = new_id;
            }
        }
        TyKind::Array { ty, .. } => {
            update_type(ty, forward_to_def);
        }
        TyKind::Sequence { ty, .. } => {
            update_type(ty, forward_to_def);
        }
        TyKind::Map { key, elem, .. } => {
            update_type(key, forward_to_def);
            update_type(elem, forward_to_def);
        }
        _ => {
            // Primitive types, strings, etc. don't have type references
        }
    }
}