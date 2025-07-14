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

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::{Diag, Label, error_span};
use ic_syntax::{Ident, Item, Span};

use crate::Context;
use crate::hir::*;
use crate::scope::ScopeId;

/// Maps fully-qualified names to their DefIds.
pub type NameMap = CaseMap<DefId>;

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
        let path = self.scopes[1..] // Skip <global>
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
    current_scope: ScopeId,
    order: Vec<DefId>,
    errors: Vec<Diag>,
}

impl<'a> NameCollector<'a> {
    fn new(ctx: &'a mut Context) -> Self {
        let root_scope = ctx.scopes.root();
        Self {
            ctx,
            name_map: CaseMap::new(),
            scope_stack: ScopeStack::new(),
            current_scope: root_scope,
            order: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Allocates a definition with annotations
    fn alloc_definition_with_annotations(
        &mut self,
        ident: Ident,
        kind: DefKind,
        span: Span,
        annotations: &[ic_syntax::AnnotationAppl],
    ) -> DefId {
        let id = self.alloc_definition(ident, kind, span);

        // Convert and set annotations
        let hir_annotations = super::convert_annotations(annotations);
        let def = self.ctx.definitions.get_mut(id);
        def.annotations = hir_annotations;

        id
    }

    /// Allocates a placeholder definition with proper parent tracking.
    fn alloc_definition(&mut self, ident: Ident, kind: DefKind, span: Span) -> DefId {
        let parent = self.scope_stack.current_parent();
        let qualified_name = self.scope_stack.qualified_name(&ident.name);

        // Check if this is a forward declaration or module before moving kind
        let new_is_decl = matches!(&kind, DefKind::Decl(_));
        let new_is_module = matches!(&kind, DefKind::Module(_));

        // Determine which types are complete immediately vs need resolution
        let flags = match &kind {
            // These are complete immediately
            DefKind::Const(_) => DefFlags::default(),
            DefKind::Enum(_) => DefFlags::default(), // Enums can't be forward declared
            DefKind::Bitmask(_) => DefFlags::default(), // Bitmasks can't be forward declared
            DefKind::Bitset(_) => DefFlags::default(), // Bitsets can't be forward declared
            DefKind::Module(_) => DefFlags::default(), // Modules can't be forward declared
            DefKind::Annotation(_) => DefFlags::default(), // Annotations can't be forward declared
            DefKind::Alias(_) => DefFlags::default(), // Type aliases can't be forward declared

            // These need resolution and can have forward declarations
            DefKind::Decl(_) => DefFlags::IS_INCOMPLETE, // Forward declarations are always incomplete
            DefKind::Struct(_) => DefFlags::IS_INCOMPLETE, // Can be forward declared
            DefKind::Union(_) => DefFlags::IS_INCOMPLETE, // Can be forward declared
            DefKind::Interface(_) => DefFlags::IS_INCOMPLETE, // Can be forward declared
            DefKind::Valuetype(_) => DefFlags::IS_INCOMPLETE, // Can be forward declared
            DefKind::Except(_) => DefFlags::IS_INCOMPLETE, // Exceptions need member resolution
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent,
            annotations: Vec::new(), // Annotations will be set by caller
            span,
            kind,
            flags,
        });

        // Check for duplicate names - but allow forward declarations and module reopening
        if let Some(&existing_id) = self.name_map.get(&qualified_name) {
            let existing = self.ctx.definitions.get(existing_id);

            // Check if either the existing or new definition is a forward declaration
            let existing_is_decl = matches!(&existing.kind, DefKind::Decl(_));

            // Check if existing is a module (modules can be reopened)
            let existing_is_module = matches!(&existing.kind, DefKind::Module(_));

            // Allow multiple forward declarations or a forward declaration + definition
            if !existing_is_decl || !new_is_decl {
                // We have at least one actual definition
                // If both are definitions (not forward declarations), it's an error
                // UNLESS they are both modules (which can be reopened)
                if !existing_is_decl && !new_is_decl && !(existing_is_module && new_is_module) {
                    self.errors.push(
                        error_span(
                            format!("duplicate definition of `{}`", ident.name),
                            Label::new(ident.span).message("redefined here"),
                        )
                        .label(Label::new(existing.ident.span).message("first defined here")),
                    );
                }
                // If the existing is a definition and new is a forward declaration,
                // or existing is a forward declaration and new is a definition,
                // we'll allow it and let the validation phase check type compatibility
            }
            // For multiple forward declarations, always allowed
        }

        // Always insert into name_map - for forward declarations, we want the last one
        // For definitions, we want the actual definition, not the forward declaration
        if !new_is_decl || !self.name_map.contains_key(&qualified_name) {
            self.name_map.insert(qualified_name, id);
        }

        // Add to scope tree
        self.ctx
            .scopes
            .add_definition(self.current_scope, ident.name.clone(), id);

        id
    }

    /// Creates a scoped definition with annotations
    fn alloc_scoped_definition_with_annotations(
        &mut self,
        ident: Ident,
        kind: DefKind,
        span: Span,
        annotations: &[ic_syntax::AnnotationAppl],
    ) -> DefId {
        let id = self.alloc_definition_with_annotations(ident.clone(), kind, span, annotations);
        self.scope_stack.push(ident.name.clone(), id);
        // Create a child scope in the scope tree
        let new_scope =
            self.ctx
                .scopes
                .create_child_scope(self.current_scope, ident.name, Some(id));
        self.current_scope = new_scope;

        id
    }

    /// Creates a definition that introduces a new scope.
    fn alloc_scoped_definition(&mut self, ident: Ident, kind: DefKind, span: Span) -> DefId {
        let id = self.alloc_definition(ident.clone(), kind, span);
        self.scope_stack.push(ident.name.clone(), id);

        // Create a child scope in the scope tree
        let new_scope =
            self.ctx
                .scopes
                .create_child_scope(self.current_scope, ident.name, Some(id));
        self.current_scope = new_scope;

        id
    }

    fn collect_module(&mut self, def: &ic_syntax::ModuleDef) -> DefId {
        // Check if this module already exists (for module reopening)
        let qualified_name = self.scope_stack.qualified_name(&def.ident.name);
        let existing_modules: Vec<DefId> = self
            .name_map
            .iter()
            .filter(|(k, v)| {
                k.eq_ignore_ascii_case(&qualified_name)
                    && matches!(&self.ctx.definitions.get(**v).kind, DefKind::Module(_))
            })
            .map(|(_, v)| *v)
            .collect();

        // Always create a new DefId for module declarations, even when reopening
        // This ensures each module declaration gets its own DefId in the HIR
        let id = self.alloc_scoped_definition_with_annotations(
            def.ident.clone(),
            DefKind::Module(ModuleTy {
                definitions: Vec::new(),
            }),
            def.span,
            &def.annotations,
        );

        // If this is a module reopening, copy all definitions from previous instances
        // into the new module's scope so they're visible for type resolution
        if !existing_modules.is_empty() {
            let new_scope = self.current_scope;
            for &existing_id in &existing_modules {
                if let Some(existing_scope) = self.ctx.scopes.find_scope_for_def(existing_id) {
                    // Copy all definitions from the existing module scope to the new one
                    let existing_scope_data = self.ctx.scopes.get_scope(existing_scope);
                    let definitions_to_copy: Vec<(String, DefId)> = existing_scope_data
                        .definitions
                        .iter()
                        .map(|(k, v)| (k.to_string(), *v))
                        .collect();

                    for (name, def_id) in definitions_to_copy {
                        self.ctx.scopes.add_definition(new_scope, name, def_id);
                    }
                }
            }
        }

        // Collect nested definitions
        let mut child_ids = Vec::new();
        for item in &def.definitions {
            child_ids.extend(self.collect_item(item));
        }

        // Update module with children - append to existing definitions if reopening
        if let Def {
            kind: DefKind::Module(module),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            module.definitions.extend(child_ids);
        }

        self.scope_stack.pop();

        // Restore parent scope
        let parent_scope = self.ctx.scopes.get_scope(self.current_scope).parent;
        self.current_scope = parent_scope.unwrap_or(self.ctx.scopes.root());

        id
    }

    fn collect_interface(&mut self, def: &ic_syntax::InterfaceDef) -> DefId {
        let id = self.alloc_scoped_definition_with_annotations(
            def.ident.clone(),
            DefKind::Interface(InterfaceTy::default()),
            def.span,
            &def.annotations,
        );

        // Collect nested type definitions
        let mut child_ids = Vec::new();
        for member in &def.members {
            if let ic_syntax::InterfaceMember::Item(item) = member {
                child_ids.extend(self.collect_item(item));
            }
        }

        // Update interface with children
        if let Def {
            kind: DefKind::Interface(interface),
            flags,
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            interface.definitions = child_ids;
            interface.is_local = def.local.is_some();
            // Interface with members is complete
            if !def.members.is_empty() {
                *flags &= !DefFlags::IS_INCOMPLETE;
            }
        }

        self.scope_stack.pop();

        // Restore parent scope
        let parent_scope = self.ctx.scopes.get_scope(self.current_scope).parent;
        self.current_scope = parent_scope.unwrap_or(self.ctx.scopes.root());

        id
    }

    fn collect_annotation(&mut self, def: &ic_syntax::AnnotationDef) -> DefId {
        let id = self.alloc_scoped_definition_with_annotations(
            def.ident.clone(),
            DefKind::Annotation(AnnotationTy {
                members: Vec::new(),
                types: Vec::new(),
            }),
            def.span,
            &def.annotations,
        );

        // Collect nested type definitions
        let mut child_ids = Vec::new();
        for field in &def.params {
            if let ic_syntax::AnnotationField::Item(item) = field {
                child_ids.extend(self.collect_item(item));
            }
        }

        // Update annotation with children
        if let Def {
            kind: DefKind::Annotation(ann),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            ann.types = child_ids;
        }

        self.scope_stack.pop();

        // Restore parent scope
        let parent_scope = self.ctx.scopes.get_scope(self.current_scope).parent;
        self.current_scope = parent_scope.unwrap_or(self.ctx.scopes.root());

        id
    }

    fn collect_valuetype(&mut self, def: &ic_syntax::ValuetypeDef) -> DefId {
        let id = self.alloc_scoped_definition_with_annotations(
            def.ident.clone(),
            DefKind::Valuetype(ValueTy {
                parent: None,
                extends: None,
                prototypes: Vec::new(),
                members: Vec::new(),
                definitions: Vec::new(),
            }),
            def.span,
            &def.annotations,
        );

        // Collect nested type definitions
        let mut child_ids = Vec::new();
        for item in &def.definitions {
            child_ids.extend(self.collect_item(item));
        }

        // Update valuetype with children
        if let Def {
            kind: DefKind::Valuetype(valuetype),
            flags,
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            valuetype.definitions = child_ids;
            // Valuetype with members or definitions is complete
            if !def.members.is_empty() || !def.definitions.is_empty() {
                *flags &= !DefFlags::IS_INCOMPLETE;
            }
        }

        self.scope_stack.pop();

        // Restore parent scope
        let parent_scope = self.ctx.scopes.get_scope(self.current_scope).parent;
        self.current_scope = parent_scope.unwrap_or(self.ctx.scopes.root());

        id
    }

    fn collect_bitset(&mut self, def: &ic_syntax::BitsetDef) -> DefId {
        let id = self.alloc_definition_with_annotations(
            def.ident.clone(),
            DefKind::Bitset(BitsetTy {
                parent: None,       // Will be resolved in resolve phase
                fields: Vec::new(), // Will be populated in resolve phase
            }),
            def.span,
            &def.annotations,
        );

        // Bitsets are complete immediately
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);

        id
    }

    fn collect_simple_definition(&mut self, ident: Ident, kind: DefKind, span: Span) -> DefId {
        let id = self.alloc_definition(ident, kind, span);
        id
    }

    fn collect_simple_definition_with_annotations(
        &mut self,
        ident: Ident,
        kind: DefKind,
        span: Span,
        annotations: &[ic_syntax::AnnotationAppl],
    ) -> DefId {
        let id = self.alloc_definition_with_annotations(ident, kind, span, annotations);
        id
    }

    fn collect_item(&mut self, item: &Item) -> Vec<DefId> {
        match item {
            Item::ModuleValue(v) => vec![self.collect_module(v)],
            Item::InterfaceValue(v) => vec![self.collect_interface(v)],
            Item::AnnotationValue(v) => vec![self.collect_annotation(v)],

            // Simple types without nested scopes
            Item::StructValue(v) => {
                let id = self.alloc_definition_with_annotations(
                    v.ident.clone(),
                    DefKind::Struct(StructTy {
                        parent: None,
                        members: Vec::new(),
                    }),
                    v.span,
                    &v.annotations,
                );

                // If the struct has members, it's a complete definition, not a forward declaration
                if !v.members.is_empty() {
                    if let Def { flags, .. } = self.ctx.definitions.get_mut(id) {
                        *flags &= !DefFlags::IS_INCOMPLETE;
                    }
                }

                // Already registered in scope by alloc_definition
                vec![id]
            }
            Item::UnionValue(v) => {
                let id = self.alloc_definition_with_annotations(
                    v.ident.clone(),
                    DefKind::Union(UnionTy {
                        disc: placeholder_type(v.span),
                        variants: Vec::new(),
                    }),
                    v.span,
                    &v.annotations,
                );

                // If the union has fields, it's a complete definition
                if !v.fields.is_empty() {
                    if let Def { flags, .. } = self.ctx.definitions.get_mut(id) {
                        *flags &= !DefFlags::IS_INCOMPLETE;
                    }
                }

                // Already registered in scope by alloc_definition
                vec![id]
            }
            Item::EnumValue(v) => vec![self.collect_simple_definition_with_annotations(
                v.ident.clone(),
                DefKind::Enum(EnumTy {
                    fields: Vec::new(),
                    ty: placeholder_type(v.span),
                }),
                v.span,
                &v.annotations,
            )],
            Item::ExceptionValue(v) => {
                let id = self.alloc_definition_with_annotations(
                    v.ident.clone(),
                    DefKind::Except(ExceptTy {
                        members: Vec::new(),
                    }),
                    v.span,
                    &v.annotations,
                );

                // If the exception has members, it's a complete definition
                if !v.members.is_empty() {
                    if let Def { flags, .. } = self.ctx.definitions.get_mut(id) {
                        *flags &= !DefFlags::IS_INCOMPLETE;
                    }
                }

                // Already registered in scope by alloc_definition
                vec![id]
            }
            Item::BitmaskValue(v) => vec![self.collect_simple_definition_with_annotations(
                v.ident.clone(),
                DefKind::Bitmask(BitmaskTy {
                    flags: Vec::new(),
                    ty: placeholder_type(v.span),
                }),
                v.span,
                &v.annotations,
            )],
            Item::ConstValue(v) => {
                // Constants might have array declarators
                vec![self.collect_simple_definition_with_annotations(
                    extract_declarator_name(&v.decl),
                    DefKind::Const(ConstTy {
                        value: Numeric::Null, // Placeholder
                        ty: placeholder_type(v.span),
                    }),
                    v.span,
                    &v.annotations,
                )]
            }
            Item::AliasValue(v) => {
                // Expand multiple declarators into separate definitions
                v.decl
                    .iter()
                    .map(|decl| {
                        self.collect_simple_definition_with_annotations(
                            extract_declarator_name(decl),
                            DefKind::Alias(AliasTy {
                                ty: placeholder_type(v.span),
                            }),
                            v.span,
                            &v.annotations,
                        )
                    })
                    .collect()
            }
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

            Item::ValuetypeValue(v) => vec![self.collect_valuetype(v)],
            Item::BitsetValue(v) => vec![self.collect_bitset(v)],
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
        ic_syntax::Declarator::Simple(ident) => ident.clone(),
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

    let NameCollector {
        name_map,
        order,
        errors,
        ..
    } = collector;

    (context, name_map, order, errors)
}
