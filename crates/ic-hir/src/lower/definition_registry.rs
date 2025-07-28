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

//! Common functionality for registering and managing definitions during resolution.

use ic_alloc::insensitive::CaseMap;
use ic_alloc::arena::Id;
use ic_diagnostic::{Diag, Label, error_span};

use crate::Context;
use crate::hir::{Decl, DefId, DefKind};
use crate::scope::ScopeId;
use crate::lower::definition_builder::DefBuilder;

/// Helper for registering definitions and checking for duplicates.
pub struct DefinitionRegistry<'a> {
    ctx: &'a mut Context,
    name_map: &'a mut CaseMap<DefId>,
    current_scope: ScopeId,
    scope_path: &'a [String],
    errors: &'a mut Vec<Diag>,
}

impl<'a> DefinitionRegistry<'a> {
    /// Creates a new definition registry.
    pub fn new(
        ctx: &'a mut Context,
        name_map: &'a mut CaseMap<DefId>,
        current_scope: ScopeId,
        scope_path: &'a [String],
        errors: &'a mut Vec<Diag>,
    ) -> Self {
        Self {
            ctx,
            name_map,
            current_scope,
            scope_path,
            errors,
        }
    }

    /// Generates a qualified name by joining the scope path with the given name.
    pub fn qualified_name(&self, name: &str) -> String {
        if self.scope_path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.scope_path.join("::"), name)
        }
    }

    /// Registers a definition in both the name map and current scope.
    /// Returns true if registration was successful, false if there was a conflict.
    pub fn register(&mut self, name: &str, id: DefId) -> bool {
        let qualified_name = self.qualified_name(name);
        
        // Register in name map
        self.name_map.insert(qualified_name, id);
        
        // Register in scope and check for duplicates
        self.register_in_scope(name.to_string(), id)
    }

    /// Registers a definition in the current scope, checking for duplicates.
    /// Returns true if registration was successful, false if there was a conflict.
    fn register_in_scope(&mut self, name: String, id: DefId) -> bool {
        if let Some(existing_id) =
            self.ctx
                .scopes
                .add_definition(self.current_scope, name.clone(), id)
        {
            // Found a duplicate - handle it
            self.handle_duplicate(name, existing_id, id);
            false
        } else {
            true
        }
    }

    /// Handles duplicate definitions according to IDL rules.
    fn handle_duplicate(&mut self, name: String, existing_id: DefId, new_id: DefId) {
        let existing = self.ctx.definitions.get(existing_id);
        let new_def = self.ctx.definitions.get(new_id);

        // Check if both are forward declarations
        if let (DefKind::Decl(existing_decl), DefKind::Decl(new_decl)) =
            (&existing.kind, &new_def.kind)
        {
            // If they're different types, that's an error
            if !self.are_compatible_forward_decls(existing_decl, new_decl) {
                self.errors.push(
                    error_span(
                        format!(
                            "`{}` forward declared as both {} and {}",
                            name,
                            decl_kind_name(existing_decl),
                            decl_kind_name(new_decl)
                        ),
                        Label::new(new_def.span).message(format!(
                            "forward declared as {} here",
                            decl_kind_name(new_decl)
                        )),
                    )
                    .label(Label::new(existing.span).message(format!(
                        "first forward declared as {} here",
                        decl_kind_name(existing_decl)
                    ))),
                );
                return;
            }
            // Same type forward declarations are allowed
            return;
        }

        // Check if one is a forward declaration and the other is a definition
        if self.is_valid_forward_decl_and_definition(&existing.kind, &new_def.kind) {
            return;
        }

        // Both are definitions - this is always an error
        self.errors.push(
            error_span(
                format!("duplicate definition of `{}`", name),
                Label::new(new_def.span).message("redefined here"),
            )
            .label(Label::new(existing.span).message("first defined here")),
        );
    }

    /// Checks if two forward declarations are compatible.
    fn are_compatible_forward_decls(&self, decl1: &Decl, decl2: &Decl) -> bool {
        matches!(
            (decl1, decl2),
            (Decl::Struct, Decl::Struct)
                | (Decl::Union, Decl::Union)
                | (Decl::Interface, Decl::Interface)
                | (Decl::Valuetype, Decl::Valuetype)
                | (Decl::Native, Decl::Native)
        )
    }

    /// Checks if one definition is a forward declaration and the other is a valid definition.
    fn is_valid_forward_decl_and_definition(&self, existing: &DefKind, new: &DefKind) -> bool {
        match (existing, new) {
            (DefKind::Decl(decl_type), other) | (other, DefKind::Decl(decl_type)) => {
                matches!(
                    (decl_type, other),
                    (Decl::Struct, DefKind::Struct(_))
                        | (Decl::Union, DefKind::Union(_))
                        | (Decl::Interface, DefKind::Interface(_))
                        | (Decl::Valuetype, DefKind::Struct(_))
                )
            }
            _ => false,
        }
    }

    /// Checks if a definition with the given qualified name already exists.
    /// Returns the existing DefId if found.
    pub fn check_existing(&self, qualified_name: &str) -> Option<DefId> {
        self.name_map.get(qualified_name).copied()
    }

    /// Reports a duplicate definition error for the given type.
    pub fn report_duplicate(&mut self, name: &str, kind: &str, new_span: ic_syntax::Span, existing_id: DefId) {
        let existing = self.ctx.definitions.get(existing_id);
        self.errors.push(
            error_span(
                format!("duplicate {} `{}`", kind, name),
                Label::new(new_span).message(format!("{} redefined here", kind)),
            )
            .label(Label::new(existing.span).message(format!("{} first defined here", kind))),
        );
    }
    
    /// Registers a definition builder and builds it with the allocated ID.
    pub fn register_and_build(&mut self, builder: DefBuilder) -> DefId {
        // Build a temporary definition to extract the ident
        let temp_def = builder.build_with_id(Id::_do_not_use());
        let ident = temp_def.ident.clone();
        let qualified_name = self.qualified_name(&ident.name);
        
        // Rebuild the builder with the same parameters
        let builder = DefBuilder::new(ident.clone())
            .parent(temp_def.parent)
            .annotations(temp_def.annotations)
            .span(temp_def.span)
            .kind(temp_def.kind)
            .flags(temp_def.flags);
        
        // Allocate the definition
        let id = self.ctx.definitions.alloc_with_id(|id| builder.build_with_id(id));
        
        // Register in name map
        self.name_map.insert(qualified_name, id);
        
        // Register in scope
        self.ctx.scopes.add_definition(self.current_scope, ident.name, id);
        
        id
    }
}

/// Gets a human-readable name for a forward declaration type.
fn decl_kind_name(decl: &Decl) -> &'static str {
    match decl {
        Decl::Struct => "struct",
        Decl::Union => "union",
        Decl::Interface => "interface",
        Decl::Valuetype => "valuetype",
        Decl::Native => "native",
    }
}