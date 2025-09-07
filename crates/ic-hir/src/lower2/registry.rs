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

//! Central registry for managing forward declarations and definitions.

use std::collections::HashMap;

use ic_diagnostic::Label;
use ic_syntax::Ident;

use crate::hir::{Decl, DefId, DefKind};
use crate::scope::ScopeId;

use super::Diagnostics;

/// Case-folded name for case-insensitive lookup.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NameKey(String);

impl NameKey {
    pub fn new(name: &str) -> Self {
        Self(name.to_ascii_lowercase())
    }
}

/// Tag to distinguish definition kinds in the registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DefKindTag {
    Struct,
    Union,
    Interface,
    Valuetype,
    Native,
    Enum,
    Bitmask,
    Const,
    Annotation,
    Module,
}

impl DefKindTag {
    /// Extract the tag from a DefKind.
    /// Returns None for forward declarations (Decl) and other non-definition kinds.
    pub fn from_def_kind(kind: &DefKind) -> Option<Self> {
        match kind {
            DefKind::Struct(_) => Some(Self::Struct),
            DefKind::Union(_) => Some(Self::Union),
            DefKind::Interface(_) => Some(Self::Interface),
            DefKind::Valuetype(_) => Some(Self::Valuetype),
            DefKind::Enum(_) => Some(Self::Enum),
            DefKind::Bitmask(_) => Some(Self::Bitmask),
            DefKind::Const(_) => Some(Self::Const),
            DefKind::Annotation(_) => Some(Self::Annotation),
            DefKind::Module(_) => Some(Self::Module),
            DefKind::Decl(_) => None, // Forward declarations don't have a DefKindTag
            DefKind::Alias(_) => None, // Aliases are handled separately
            DefKind::Except(_) => None, // Exceptions aren't tracked this way
            DefKind::Bitset(_) => None, // Bitsets aren't tracked this way
        }
    }
}

/// Registry for tracking forward declarations and definitions.
pub struct DefinitionRegistry {
    /// Map (scope, case-folded name, decl kind) → forward-decl DefId
    forward_decls: HashMap<(ScopeId, NameKey, Decl), DefId>,
    
    /// Map (scope, case-folded name, def kind) → definition DefId
    definitions: HashMap<(ScopeId, NameKey, DefKindTag), DefId>,
}

impl DefinitionRegistry {
    pub fn new() -> Self {
        Self {
            forward_decls: HashMap::new(),
            definitions: HashMap::new(),
        }
    }
    
    /// Register a forward declaration.
    /// Returns DefId if successful, None if there's a conflict.
    pub fn register_forward_decl(
        &mut self,
        scope: ScopeId,
        name: &Ident,
        kind: Decl,
        def_id: DefId,
        diagnostics: &mut Diagnostics,
    ) -> Option<DefId> {
        let key = (scope, NameKey::new(&name.name), kind);
        
        // Check if forward decl already exists
        if let Some(&existing_id) = self.forward_decls.get(&key) {
            // Multiple forward declarations of the same kind are allowed
            return Some(existing_id);
        }
        
        // Check for definition conflict
        let def_key = (scope, NameKey::new(&name.name), def_kind_tag_from_decl(kind));
        if let Some(&_existing_def_id) = self.definitions.get(&def_key) {
            diagnostics.error(
                format!("forward declaration of `{}` conflicts with existing definition", name.name),
                Label::new(name.span).message("forward declaration here"),
            );
            return None;
        }
        
        // Register the forward declaration
        self.forward_decls.insert(key, def_id);
        Some(def_id)
    }
    
    /// Register a definition.
    /// Returns DefId if successful, None if there's a conflict.
    pub fn register_definition(
        &mut self,
        scope: ScopeId,
        name: &Ident,
        kind: DefKindTag,
        def_id: DefId,
        diagnostics: &mut Diagnostics,
    ) -> Option<DefId> {
        let key = (scope, NameKey::new(&name.name), kind);
        
        // Check if definition already exists
        if let Some(&_existing_id) = self.definitions.get(&key) {
            diagnostics.error(
                format!("redefinition of `{}`", name.name),
                Label::new(name.span).message("redefined here"),
            );
            return None;
        }
        
        // Register the definition
        self.definitions.insert(key, def_id);
        Some(def_id)
    }
    
    /// Find a forward declaration for the given name and kind.
    pub fn find_forward_decl(
        &self,
        scope: ScopeId,
        name: &str,
        kind: Decl,
    ) -> Option<DefId> {
        let key = (scope, NameKey::new(name), kind);
        self.forward_decls.get(&key).copied()
    }
    
    /// Find a definition for the given name and kind.
    pub fn find_definition(
        &self,
        scope: ScopeId,
        name: &str,
        kind: DefKindTag,
    ) -> Option<DefId> {
        let key = (scope, NameKey::new(name), kind);
        self.definitions.get(&key).copied()
    }
    
    /// Get all forward declarations and their matching definitions.
    pub fn get_forward_to_def_mapping(&self) -> HashMap<DefId, DefId> {
        let mut mapping = HashMap::new();
        
        for ((scope, name, decl_kind), &decl_id) in &self.forward_decls {
            let def_kind = def_kind_tag_from_decl(*decl_kind);
            if let Some(&def_id) = self.definitions.get(&(*scope, name.clone(), def_kind)) {
                mapping.insert(decl_id, def_id);
            }
        }
        
        mapping
    }
}

/// Convert declaration kind to definition kind tag.
fn def_kind_tag_from_decl(decl: Decl) -> DefKindTag {
    match decl {
        Decl::Struct => DefKindTag::Struct,
        Decl::Union => DefKindTag::Union,
        Decl::Interface => DefKindTag::Interface,
        Decl::Valuetype => DefKindTag::Valuetype,
        Decl::Native => DefKindTag::Native,
    }
}