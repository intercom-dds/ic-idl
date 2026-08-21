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

use std::collections::HashMap;

use ic_diagnostic::{Label, error_span};
use ic_hir::hir::{Decl, DefId, DefKind};
use ic_hir::scope::ScopeId;
use ic_syntax::Ident;

use crate::Diagnostics;

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
    Bitset,
    Const,
    Module,
    Annotation,
    Alias,
}

/// Registry for tracking forward declarations and definitions.
pub struct DefinitionRegistry {
    /// Map (scope, case-folded name) -> forward declaration info.
    /// Multiple forward decls of different types are tracked here
    forward_decls: HashMap<(ScopeId, NameKey), Vec<(Decl, DefId)>>,

    /// Map (scope, case-folded name) -> definition `DefId`.
    /// Only one definition per name is allowed in a scope
    definitions: HashMap<(ScopeId, NameKey), DefId>,
}

impl Default for DefinitionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DefinitionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            forward_decls: HashMap::new(),
            definitions: HashMap::new(),
        }
    }

    /// Register a forward declaration.
    /// Returns `DefId` if successful, None if there's a conflict.
    pub fn register_forward_decl(
        &mut self,
        scope: ScopeId,
        name: &Ident,
        kind: Decl,
        def_id: DefId,
        diagnostics: &mut Diagnostics,
        context: &ic_hir::Context,
    ) -> Option<DefId> {
        let key = (scope, NameKey::new(&name.name));
        let forward_decls = self.forward_decls.entry(key.clone()).or_default();

        // Check if we already have a forward declaration of this type
        for (existing_kind, _existing_id) in forward_decls.iter() {
            if *existing_kind == kind {
                forward_decls.push((kind, def_id));
                return Some(def_id);
            }
        }

        // Check if there's a forward declaration with a different type
        if !forward_decls.is_empty() {
            let (existing_kind, existing_id) = &forward_decls[0];
            let existing_def = context.definitions.get(*existing_id);
            let existing_type_str = decl_type_str(*existing_kind);
            let new_type_str = decl_type_str(kind);

            diagnostics.errors.push(
                error_span(
                    format!(
                        "`{}` forward declared as both {} and {}",
                        name.name, existing_type_str, new_type_str
                    ),
                    Label::new(existing_def.ident.span).message(format!(
                        "first forward declared as {existing_type_str} here"
                    )),
                )
                .label(
                    Label::new(name.span)
                        .message(format!("forward declared as {new_type_str} here")),
                ),
            );
            forward_decls.push((kind, def_id));
            return None;
        }

        forward_decls.push((kind, def_id));
        Some(def_id)
    }

    /// Register a definition.
    /// Returns `DefId` if successful, None if there's a conflict.
    pub fn register_definition(
        &mut self,
        scope: ScopeId,
        name: &Ident,
        kind: DefKindTag,
        def_id: DefId,
        diagnostics: &mut Diagnostics,
        context: &ic_hir::Context,
    ) -> Option<DefId> {
        let key = (scope, NameKey::new(&name.name));

        // Check if definition already exists
        if let Some(&existing_id) = self.definitions.get(&key) {
            let existing_def = context.definitions.get(existing_id);
            let existing_kind = def_kind_from_def(existing_def);

            if kind == DefKindTag::Module && existing_kind == DefKindTag::Module {
                return Some(existing_id);
            }

            diagnostics.errors.push(
                error_span(
                    format!("duplicate definition of `{}`", name.name),
                    Label::new(existing_def.ident.span).message("originally defined here"),
                )
                .label(Label::new(name.span).message("redefined here")),
            );
            return None;
        }

        // Check if there's a forward declaration with a mismatched type
        if let Some(forward_decls) = self.forward_decls.get(&key) {
            for (decl_kind, forward_id) in forward_decls {
                let expected_def_kind = def_kind_tag_from_decl(*decl_kind);
                if expected_def_kind != kind {
                    let forward_def = context.definitions.get(*forward_id);
                    let decl_type_str = decl_type_str(*decl_kind);
                    let def_type_str = def_kind_tag_str(kind);

                    diagnostics.errors.push(
                        error_span(
                            format!(
                                "forward declaration of `{}` as {} conflicts with {} definition",
                                name.name, decl_type_str, def_type_str
                            ),
                            Label::new(forward_def.ident.span)
                                .message(format!("forward declared as {decl_type_str} here")),
                        )
                        .label(
                            Label::new(name.span)
                                .message(format!("defined as {def_type_str} here")),
                        ),
                    );
                }
            }
        }

        self.definitions.insert(key, def_id);
        Some(def_id)
    }

    /// Get all forward declarations and their matching definitions.
    #[must_use]
    pub fn get_forward_to_def_mapping(&self) -> HashMap<DefId, DefId> {
        let mut mapping = HashMap::new();

        for ((scope, name), forward_decls) in &self.forward_decls {
            if let Some(&def_id) = self.definitions.get(&(*scope, name.clone())) {
                // Map all forward declarations to the definition
                for (_, decl_id) in forward_decls {
                    mapping.insert(*decl_id, def_id);
                }
            }
        }

        mapping
    }
}

fn def_kind_from_def(def: &ic_hir::hir::Def) -> DefKindTag {
    match &def.kind {
        DefKind::Struct(_) | DefKind::Except(_) => DefKindTag::Struct,
        DefKind::Union(_) => DefKindTag::Union,
        DefKind::Interface(_) => DefKindTag::Interface,
        DefKind::Valuetype(_) => DefKindTag::Valuetype,
        DefKind::Decl(decl) => def_kind_tag_from_decl(*decl),
        DefKind::Enum(_) => DefKindTag::Enum,
        DefKind::Bitmask(_) => DefKindTag::Bitmask,
        DefKind::Bitset(_) => DefKindTag::Bitset,
        DefKind::Const(_) => DefKindTag::Const,
        DefKind::Module(_) => DefKindTag::Module,
        DefKind::Alias(_) => DefKindTag::Alias,
        DefKind::Annotation(_) => DefKindTag::Annotation,
    }
}

fn def_kind_tag_from_decl(decl: Decl) -> DefKindTag {
    match decl {
        Decl::Struct => DefKindTag::Struct,
        Decl::Union => DefKindTag::Union,
        Decl::Interface => DefKindTag::Interface,
        Decl::Valuetype => DefKindTag::Valuetype,
        Decl::Native => DefKindTag::Native,
    }
}

fn decl_type_str(decl: Decl) -> &'static str {
    match decl {
        Decl::Struct => "struct",
        Decl::Union => "union",
        Decl::Interface => "interface",
        Decl::Valuetype => "valuetype",
        Decl::Native => "native",
    }
}

fn def_kind_tag_str(kind: DefKindTag) -> &'static str {
    match kind {
        DefKindTag::Struct => "struct",
        DefKindTag::Union => "union",
        DefKindTag::Interface => "interface",
        DefKindTag::Valuetype => "valuetype",
        DefKindTag::Native => "native",
        DefKindTag::Enum => "enum",
        DefKindTag::Const => "const",
        DefKindTag::Bitmask => "bitmask",
        DefKindTag::Bitset => "bitset",
        DefKindTag::Module => "module",
        DefKindTag::Alias => "alias",
        DefKindTag::Annotation => "annotation",
    }
}
