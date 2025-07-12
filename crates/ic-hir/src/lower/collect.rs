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

//! Phase 1: Name collection and definition discovery.
//!
//! This phase walks the AST and:
//! - Creates placeholder HIR definitions for all types
//! - Builds parent-child relationships
//! - Constructs a name resolution map
//! - Does NOT resolve type references or evaluate expressions

use std::collections::HashMap;

use ic_diagnostic::{Diag, Label, error_span};
use ic_syntax::{Ident, Item, ItemKind, Path, Span};

use crate::{Context, hir::*};

/// Maps fully-qualified names to their DefIds.
pub type NameMap = HashMap<String, DefId>;

/// Tracks the current scope during collection.
#[derive(Debug)]
struct ScopeStack {
    /// Stack of (name, DefId) pairs representing the current scope hierarchy.
    scopes: Vec<(String, Option<DefId>)>,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            scopes: vec![("<global>".to_string(), None)],
        }
    }
    
    fn push(&mut self, name: String, id: DefId) {
        self.scopes.push((name, Some(id)));
    }
    
    fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    
    fn current_parent(&self) -> Option<DefId> {
        self.scopes.last()?.1
    }
    
    fn qualified_name(&self, name: &str) -> String {
        let path = self.scopes[1..]  // Skip <global>
            .iter()
            .map(|(n, _)| n.as_str())
            .chain(std::iter::once(name))
            .collect::<Vec<_>>()
            .join("::");
        
        if path.is_empty() {
            name.to_string()
        } else {
            path
        }
    }
}

/// Collects all definitions from the AST.
pub struct NameCollector<'a> {
    ctx: &'a mut Context,
    name_map: NameMap,
    scope_stack: ScopeStack,
    order: Vec<DefId>,
    errors: Vec<Diag>,
}

impl<'a> NameCollector<'a> {
    fn new(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            name_map: HashMap::new(),
            scope_stack: ScopeStack::new(),
            order: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    /// Allocates a placeholder definition with proper parent tracking.
    fn alloc_definition(
        &mut self,
        ident: Ident,
        kind: DefKind,
        span: Span,
    ) -> DefId {
        let parent = self.scope_stack.current_parent();
        let qualified_name = self.scope_stack.qualified_name(&ident.name);
        
        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent,
            annotations: Vec::new(),  // Will be filled in resolution phase
            span,
            kind,
            flags: DefFlags::IS_INCOMPLETE,  // Mark as incomplete until resolved
        });
        
        // Check for duplicate names
        if let Some(&existing_id) = self.name_map.get(&qualified_name) {
            let existing = self.ctx.definitions.get(existing_id);
            self.errors.push(error_span(
                format!("duplicate definition of `{}`", ident.name),
                Label::new(span).message("redefined here"),
            ).label(
                Label::new(existing.span).message("first defined here")
            ));
        } else {
            self.name_map.insert(qualified_name, id);
        }
        
        id
    }
    
    /// Creates a definition that introduces a new scope.
    fn alloc_scoped_definition(
        &mut self,
        ident: Ident,
        kind: DefKind,
        span: Span,
    ) -> DefId {
        let id = self.alloc_definition(ident.clone(), kind, span);
        self.scope_stack.push(ident.name, id);
        id
    }
    
    fn collect_module(&mut self, def: &ic_syntax::ModuleDef) -> DefId {
        let id = self.alloc_scoped_definition(
            def.ident.clone(),
            DefKind::Module(ModuleTy { definitions: Vec::new() }),
            def.span,
        );
        
        // Collect nested definitions
        let mut child_ids = Vec::new();
        for item in &def.definitions {
            child_ids.extend(self.collect_item(item));
        }
        
        // Update module with children
        if let Def { kind: DefKind::Module(module), .. } = self.ctx.definitions.get_mut(id) {
            module.definitions = child_ids;
        }
        
        self.scope_stack.pop();
        id
    }
    
    fn collect_interface(&mut self, def: &ic_syntax::InterfaceDef) -> DefId {
        let id = self.alloc_scoped_definition(
            def.ident.clone(),
            DefKind::Interface(InterfaceTy::default()),
            def.span,
        );
        
        // Collect nested type definitions
        let mut child_ids = Vec::new();
        for member in &def.members {
            if let ic_syntax::InterfaceMember::Item(item) = member {
                child_ids.extend(self.collect_item(item));
            }
        }
        
        // Update interface with children
        if let Def { kind: DefKind::Interface(interface), .. } = self.ctx.definitions.get_mut(id) {
            interface.definitions = child_ids;
            interface.is_local = def.local.is_some();
        }
        
        self.scope_stack.pop();
        id
    }
    
    fn collect_annotation(&mut self, def: &ic_syntax::AnnotationDef) -> DefId {
        let id = self.alloc_scoped_definition(
            def.ident.clone(),
            DefKind::Annotation(AnnotationTy { 
                members: Vec::new(),
                types: Vec::new(),
            }),
            def.span,
        );
        
        // Collect nested type definitions
        let mut child_ids = Vec::new();
        for field in &def.params {
            if let ic_syntax::AnnotationField::Item(item) = field {
                child_ids.extend(self.collect_item(item));
            }
        }
        
        // Update annotation with children
        if let Def { kind: DefKind::Annotation(ann), .. } = self.ctx.definitions.get_mut(id) {
            ann.types = child_ids;
        }
        
        self.scope_stack.pop();
        id
    }
    
    fn collect_simple_definition(
        &mut self,
        ident: Ident,
        kind: DefKind,
        span: Span,
    ) -> DefId {
        let id = self.alloc_definition(ident, kind, span);
        id
    }
    
    fn collect_item(&mut self, item: &Item) -> Vec<DefId> {
        match item {
            Item::ModuleValue(v) => vec![self.collect_module(v)],
            Item::InterfaceValue(v) => vec![self.collect_interface(v)],
            Item::AnnotationValue(v) => vec![self.collect_annotation(v)],
            
            // Simple types without nested scopes
            Item::StructValue(v) => vec![self.collect_simple_definition(
                v.ident.clone(),
                DefKind::Struct(StructTy { parent: None, members: Vec::new() }),
                v.span,
            )],
            Item::UnionValue(v) => vec![self.collect_simple_definition(
                v.ident.clone(),
                DefKind::Union(UnionTy { 
                    disc: placeholder_type(v.span),
                    variants: Vec::new(),
                }),
                v.span,
            )],
            Item::EnumValue(v) => vec![self.collect_simple_definition(
                v.ident.clone(),
                DefKind::Enum(EnumTy { 
                    fields: Vec::new(),
                    ty: placeholder_type(v.span),
                }),
                v.span,
            )],
            Item::ExceptionValue(v) => vec![self.collect_simple_definition(
                v.ident.clone(),
                DefKind::Except(ExceptTy { members: Vec::new() }),
                v.span,
            )],
            Item::BitmaskValue(v) => vec![self.collect_simple_definition(
                v.ident.clone(),
                DefKind::Bitmask(BitmaskTy { 
                    flags: Vec::new(),
                    ty: placeholder_type(v.span),
                }),
                v.span,
            )],
            Item::ConstValue(v) => {
                // Constants might have array declarators
                vec![self.collect_simple_definition(
                    extract_declarator_name(&v.decl),
                    DefKind::Const(ConstTy { 
                        value: Numeric::Null,  // Placeholder
                        ty: placeholder_type(v.span),
                    }),
                    v.span,
                )]
            },
            Item::AliasValue(v) => {
                // Expand multiple declarators into separate definitions
                v.decl.iter().map(|decl| {
                    self.collect_simple_definition(
                        extract_declarator_name(decl),
                        DefKind::Alias(AliasTy { ty: placeholder_type(v.span) }),
                        v.span,
                    )
                }).collect()
            },
            Item::DeclValue(v) => vec![self.collect_simple_definition(
                v.ident.clone(),
                DefKind::Decl(match v.kind {
                    ic_syntax::DeclKind::Struct => Decl::Struct,
                    ic_syntax::DeclKind::Union => Decl::Union,
                    ic_syntax::DeclKind::Native => Decl::Native,
                    ic_syntax::DeclKind::Interface => Decl::Interface,
                    ic_syntax::DeclKind::Valuetype => Decl::Valuetype,
                }),
                v.span,
            )],
            
            // Skip for now
            Item::ValuetypeValue(_) | Item::BitsetValue(_) => Vec::new(),
        }
    }
}

/// Placeholder type used during collection phase.
fn placeholder_type(span: Span) -> Ty {
    Ty {
        kind: TyKind::Any,
        span,
    }
}

/// Extracts the identifier from a declarator.
fn extract_declarator_name(decl: &ic_syntax::Declarator) -> Ident {
    match decl {
        ic_syntax::Declarator::Simple(name) => Ident {
            name: name.clone(),
            span: Span::default(),  // TODO: Get proper span
        },
        ic_syntax::Declarator::Array(arr) => arr.ident.clone(),
    }
}

/// Collects all definitions from AST items.
pub fn collect_definitions(items: &[Item]) -> (Context, NameMap, Vec<DefId>, Vec<Diag>) {
    let mut context = Context::new();
    let mut collector = NameCollector::new(&mut context);
    
    // Collect all top-level items
    for item in items {
        let ids = collector.collect_item(item);
        collector.order.extend(ids);
    }
    
    let NameCollector { name_map, order, errors, .. } = collector;
    
    (context, name_map, order, errors)
}