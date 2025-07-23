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

//! Single-pass lowering from AST to HIR.
//!
//! This module combines collection and resolution into a single pass,
//! ensuring that types can only be used after they have been declared.

use ic_alloc::insensitive::CaseMap;
use ic_diagnostic::{Diag, Label, error_span, warn_span};
use ic_syntax::{Ident, Item, Path};

use crate::Context;
use crate::hir::{
    AliasTy, Ann, AnnotationTy, ConstTy, Decl, Def, DefFlags, DefId, DefKind, EnumLit, EnumTy,
    ExceptTy, InterfaceTy, Member, Numeric, PrimitiveTy, StructTy, Ty, TyKind, UnionTy,
};
use crate::scope::ScopeId;

/// Maps fully-qualified names to their `DefIds`.
pub type NameMap = CaseMap<DefId>;

/// Single-pass lowerer that processes items in order.
pub struct SinglePassLowerer<'a> {
    ctx: &'a mut Context,
    /// Maps names to `DefIds` for already-processed definitions
    name_map: NameMap,
    /// Current scope path for qualified name generation
    scope_path: Vec<String>,
    /// Current scope ID in the scope tree
    current_scope: ScopeId,
    /// Top-level definitions in order
    order: Vec<DefId>,
    /// Accumulated errors
    errors: Vec<Diag>,
    /// Accumulated warnings
    warnings: Vec<Diag>,
}

impl<'a> SinglePassLowerer<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        let root_scope = ctx.scopes.root();
        Self {
            ctx,
            name_map: CaseMap::new(),
            scope_path: vec![],
            current_scope: root_scope,
            order: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Gets the qualified name for a definition in the current scope.
    fn qualified_name(&self, name: &str) -> String {
        if self.scope_path.is_empty() {
            name.to_string()
        } else {
            let mut path = self.scope_path.clone();
            path.push(name.to_string());
            path.join("::")
        }
    }

    /// Resolves annotations from AST and returns only those that could be resolved.
    fn resolve_ast_annotations(
        &mut self,
        ast_annotations: &[ic_syntax::AnnotationAppl],
    ) -> Vec<Ann> {
        let mut resolved_annotations = Vec::new();

        for ann in ast_annotations {
            // Try to resolve annotation name
            let def_id = if let Some(id) = self.resolve_path(&ann.ident) {
                id
            } else {
                // Annotation not found - skip with warning
                let name = path_to_string(&ann.ident);
                self.warnings.push(warn_span(
                    format!("annotation `@{name}` not found"),
                    Label::new(ic_syntax::util::path_span(&ann.ident))
                        .message("unknown annotation"),
                ));
                continue;
            };

            // Get annotation definition
            let members = {
                let def = self.ctx.definitions.get(def_id);
                if let DefKind::Annotation(ann_ty) = &def.kind {
                    ann_ty.members.clone()
                } else {
                    // Not an annotation type
                    continue;
                }
            };

            // Check for multi-parameter annotations with positional arguments
            if members.len() > 1 && ann.args.iter().any(|arg| arg.ident.is_none()) {
                let ann_name = path_to_string(&ann.ident);
                self.warnings.push(warn_span(
                    format!(
                        "@{ann_name} has {} parameters and requires named arguments",
                        members.len()
                    ),
                    Label::new(ic_syntax::util::path_span(&ann.ident))
                        .message("use named arguments for annotations with multiple parameters"),
                ));
            }

            // Get the annotation identifier - use the full path
            let ann_ident = Ident {
                name: path_to_string(&ann.ident),
                span: ic_syntax::util::path_span(&ann.ident),
            };

            // Resolve annotation arguments
            let mut args = Vec::new();

            // If all arguments are positional and there's exactly one member, assign to that member
            if members.len() == 1 && ann.args.iter().all(|arg| arg.ident.is_none()) {
                if let Some(_arg) = ann.args.first() {
                    args.push(crate::hir::AnnArg {
                        ident: members[0].ident.clone(),
                        value: Numeric::Int32(0), // TODO: Evaluate expression properly
                    });
                }
            } else {
                // For named arguments or multiple members, match by name
                for arg in &ann.args {
                    if let Some(name) = &arg.ident {
                        // Find matching member
                        if let Some(member) = members.iter().find(|m| m.ident.name == name.name) {
                            args.push(crate::hir::AnnArg {
                                ident: member.ident.clone(),
                                value: Numeric::Int32(0), // TODO: Evaluate expression properly
                            });
                        }
                    }
                }
            }

            resolved_annotations.push(Ann {
                ident: ann_ident,
                def_id,
                args,
            });
        }

        resolved_annotations
    }

    /// Resolves a path to a `DefId` if it has been defined.
    fn resolve_path(&mut self, path: &Path) -> Option<DefId> {
        let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();

        // If path has leading colons (::), resolve from global scope
        let start_scope = if path.leading_colons.is_some() {
            self.ctx.scopes.root()
        } else {
            self.current_scope
        };

        // For single-segment paths, try local resolution first
        if segments.len() == 1 && path.leading_colons.is_none() {
            if let Some(def_id) = self.ctx.scopes.resolve_name_with_visibility(
                start_scope,
                segments[0],
                &self.ctx.definitions,
            ) {
                return Some(def_id);
            }
        }

        // For multi-segment paths or if local resolution failed, use regular path resolution
        self.ctx.scopes.resolve_path(start_scope, &segments)
    }

    /// Converts AST type to HIR type.
    fn resolve_type(&mut self, ty: &ic_syntax::Type) -> Ty {
        use ic_syntax::Type;

        match ty {
            Type::Any(v) => Ty {
                kind: TyKind::Any,
                span: v.span,
            },
            Type::Fixed(v) => Ty {
                kind: TyKind::Fixed,
                span: v.span,
            },
            Type::Sequence(v) => Ty {
                kind: TyKind::Sequence {
                    ty: Box::new(self.resolve_type(&v.ty)),
                    bound: None, // Will be filled in evaluation phase
                    bound_span: v.bound.as_ref().map(ic_syntax::util::expr_span),
                },
                span: v.span,
            },
            Type::String(v) => Ty {
                kind: TyKind::String {
                    wide: v.wide,
                    bound: None, // Will be filled in evaluation phase
                    bound_span: v.bound.as_ref().map(ic_syntax::util::expr_span),
                },
                span: v.span,
            },
            Type::Map(v) => Ty {
                kind: TyKind::Map {
                    key: Box::new(self.resolve_type(&v.key)),
                    elem: Box::new(self.resolve_type(&v.value)),
                    bound: None, // Will be filled in evaluation phase
                    bound_span: v.bound.as_ref().map(ic_syntax::util::expr_span),
                },
                span: v.span,
            },
            Type::Path(v) => {
                // Check if it's a primitive type
                if v.segments.len() == 1 && v.leading_colons.is_none() {
                    if let Some(prim) = resolve_primitive(&v.segments[0].name) {
                        return Ty {
                            kind: TyKind::Primitive(prim),
                            span: ic_syntax::util::path_span(v),
                        };
                    }
                }

                // Try to resolve as user-defined type
                if let Some(id) = self.resolve_path(v) {
                    Ty {
                        kind: TyKind::Adt(id),
                        span: ic_syntax::util::path_span(v),
                    }
                } else {
                    // Type not found - report error
                    let qualified = path_to_string(v);
                    self.errors.push(error_span(
                        format!("unresolved type `{qualified}`"),
                        Label::new(ic_syntax::util::path_span(v)).message("unknown type"),
                    ));

                    // Return placeholder type
                    Ty {
                        kind: TyKind::Any,
                        span: ic_syntax::util::path_span(v),
                    }
                }
            }
        }
    }

    /// Processes a forward declaration.
    fn process_forward_declaration(&mut self, decl: &ic_syntax::Decl) -> DefId {
        let qualified_name = self.qualified_name(&decl.ident.name);

        // Always create a new forward declaration DefId
        // (even if one already exists - we keep all declarations)

        // Create the forward declaration
        let kind = DefKind::Decl(match decl.kind {
            ic_syntax::DeclKind::Struct => Decl::Struct,
            ic_syntax::DeclKind::Union => Decl::Union,
            ic_syntax::DeclKind::Native => Decl::Native,
            ic_syntax::DeclKind::Interface => Decl::Interface,
            ic_syntax::DeclKind::Valuetype => Decl::Valuetype,
        });

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: decl.ident.clone(),
            parent,
            annotations: Vec::new(),
            span: decl.span,
            kind,
            flags: DefFlags::default(), // Forward declarations are complete
        });

        // DON'T update name map for forward declarations - we want to keep all of them
        // Only update if this is the first one
        if !self.name_map.contains_key(&qualified_name) {
            self.name_map.insert(qualified_name, id);
        }
        self.ctx
            .scopes
            .add_definition(self.current_scope, decl.ident.name.clone(), id);

        id
    }

    /// Processes a struct definition.
    fn process_struct(&mut self, def: &ic_syntax::StructDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Check if there's already a definition (forward declaration or full)
        let existing_def = self.name_map.get(&qualified_name).copied();

        // Check if it's a duplicate full definition
        if let Some(existing_id) = existing_def {
            let existing = self.ctx.definitions.get(existing_id);
            if matches!(existing.kind, DefKind::Struct(_)) {
                // Already have a full struct definition - error
                self.errors.push(
                    error_span(
                        format!("duplicate definition of `{}`", def.ident.name),
                        Label::new(def.ident.span).message("redefined here"),
                    )
                    .label(Label::new(existing.ident.span).message("first defined here")),
                );
            }
        }

        // Resolve parent if any
        let parent_id = if let Some(parent_path) = &def.parent {
            self.resolve_type(&ic_syntax::Type::Path(parent_path.clone()))
                .as_adt()
        } else {
            None
        };

        // Resolve members
        let mut members = Vec::new();
        for field in &def.members {
            let base_ty = self.resolve_type(&field.ty);
            let field_annotations = self.resolve_ast_annotations(&field.annotations);
            for decl in &field.names {
                let (ident, ty) = resolve_declarator(decl, base_ty.clone());
                members.push(Member {
                    ident,
                    ty,
                    annotations: field_annotations.clone(),
                    default_value: None, // Will be resolved later
                });
            }
        }

        // Always create a new definition (don't reuse forward declaration DefId)
        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Struct(StructTy {
                parent: parent_id,
                members,
            }),
            flags: DefFlags::default(),
        });

        // Update name map to point to the full definition
        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        id
    }

    /// Processes a module definition.
    fn process_module(&mut self, def: &ic_syntax::ModuleDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Create module definition
        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Module(crate::hir::ModuleTy {
                definitions: Vec::new(),
            }),
            flags: DefFlags::default(),
        });

        // Register in name map and scope
        self.name_map.insert(qualified_name, id);

        // Create new scope for module
        let new_scope = self.ctx.scopes.create_child_scope(
            self.current_scope,
            def.ident.name.clone(),
            Some(id),
        );

        // Push to scope stack
        self.scope_path.push(def.ident.name.clone());
        let old_scope = self.current_scope;
        self.current_scope = new_scope;

        // Process nested items
        let mut child_ids = Vec::new();
        for item in &def.definitions {
            let ids = self.process_item(item);
            child_ids.extend(ids);
        }

        // Update module with children
        if let Def {
            kind: DefKind::Module(module),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            module.definitions = child_ids;
        }

        // Pop scope
        self.scope_path.pop();
        self.current_scope = old_scope;

        id
    }

    /// Processes an enum definition.
    fn process_enum(&mut self, def: &ic_syntax::EnumDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Create enum literals
        let fields = def
            .fields
            .iter()
            .map(|f| {
                let field_annotations = self.resolve_ast_annotations(&f.annotations);
                EnumLit {
                    ident: f.ident.clone(),
                    value: 0, // Will be filled in evaluation phase
                    annotations: field_annotations,
                }
            })
            .collect();

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Enum(EnumTy {
                fields,
                ty: Ty {
                    kind: TyKind::Primitive(PrimitiveTy::Int32), // Default to int32
                    span: def.span,
                },
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        id
    }

    /// Processes a type alias definition.
    fn process_alias(&mut self, def: &ic_syntax::AliasDef) -> Vec<DefId> {
        let mut ids = Vec::new();

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Resolve the base type
        let base_ty = self.resolve_type(&def.ty);

        // Process each declarator
        for decl in &def.decl {
            let (ident, ty) = resolve_declarator(decl, base_ty.clone());
            let qualified_name = self.qualified_name(&ident.name);

            let parent = if self.scope_path.is_empty() {
                None
            } else {
                let parent_name = self.scope_path.join("::");
                self.name_map.get(&parent_name).copied()
            };

            let id = self.ctx.definitions.alloc_with_id(|id| Def {
                id,
                ident: ident.clone(),
                parent,
                annotations: annotations.clone(),
                span: def.span,
                kind: DefKind::Alias(AliasTy { ty }),
                flags: DefFlags::default(),
            });

            self.name_map.insert(qualified_name, id);
            self.ctx
                .scopes
                .add_definition(self.current_scope, ident.name.clone(), id);
            ids.push(id);
        }

        ids
    }

    /// Processes a union definition.
    fn process_union(&mut self, def: &ic_syntax::UnionDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Resolve discriminator type
        let disc_ty = self.resolve_type(&def.disc.ty);

        // Process union variants
        let mut variants = Vec::new();
        for field in &def.fields {
            match &field.field {
                ic_syntax::UnionElement::Member(member) => {
                    let base_ty = self.resolve_type(&member.ty);
                    let (ident, ty) = resolve_declarator(&member.decl, base_ty);
                    let field_annotations = self.resolve_ast_annotations(&field.annotations);

                    variants.push(crate::hir::Variant {
                        ident,
                        ty,
                        annotations: field_annotations,
                        labels: Vec::new(), // TODO: Process case labels properly
                        is_default: false,
                    });
                }
                ic_syntax::UnionElement::Null(_) => {
                    // TODO: Handle null/default case
                }
            }
        }

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Union(UnionTy {
                disc: disc_ty,
                variants,
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        id
    }

    /// Processes an annotation definition.
    fn process_annotation(&mut self, def: &ic_syntax::AnnotationDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        // Resolve annotations on the annotation definition itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Annotation(AnnotationTy {
                members: Vec::new(),
                types: Vec::new(),
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        // Create new scope for annotation
        let new_scope = self.ctx.scopes.create_child_scope(
            self.current_scope,
            def.ident.name.clone(),
            Some(id),
        );

        // Push to scope stack
        self.scope_path.push(def.ident.name.clone());
        let old_scope = self.current_scope;
        self.current_scope = new_scope;

        // Process annotation members
        let mut members = Vec::new();
        let mut child_ids = Vec::new();

        for param in &def.params {
            match param {
                ic_syntax::AnnotationField::Item(item) => {
                    // Handle nested type definitions
                    let ids = self.process_item(item);
                    child_ids.extend(ids);
                }
                ic_syntax::AnnotationField::Member(member) => {
                    let base_ty = self.resolve_type(&member.ty);
                    let (ident, ty) = resolve_declarator(&member.decl, base_ty);
                    members.push(Member {
                        ident,
                        ty,
                        annotations: Vec::new(),
                        default_value: None, // Will be resolved in evaluation phase
                    });
                }
            }
        }

        // Update annotation with members and types
        if let Def {
            kind: DefKind::Annotation(ann),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            ann.members = members;
            ann.types = child_ids;
        }

        // Pop scope
        self.scope_path.pop();
        self.current_scope = old_scope;

        id
    }

    /// Processes an exception definition.
    fn process_exception(&mut self, def: &ic_syntax::ExceptDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Process members
        let mut members = Vec::new();
        for field in &def.members {
            let base_ty = self.resolve_type(&field.ty);
            let field_annotations = self.resolve_ast_annotations(&field.annotations);
            for decl in &field.names {
                let (ident, ty) = resolve_declarator(decl, base_ty.clone());
                members.push(Member {
                    ident,
                    ty,
                    annotations: field_annotations.clone(),
                    default_value: None,
                });
            }
        }

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Except(ExceptTy { members }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        id
    }

    /// Processes a constant definition.
    fn process_const(&mut self, def: &ic_syntax::ConstDef) -> DefId {
        // Resolve the type
        let base_ty = self.resolve_type(&def.ty);
        let (ident, ty) = resolve_declarator(&def.decl, base_ty);

        let qualified_name = self.qualified_name(&ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Const(ConstTy {
                ty,
                value: Numeric::Int32(0), // Placeholder, will be filled in evaluation phase
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, ident.name.clone(), id);

        id
    }

    /// Processes a valuetype definition.
    fn process_valuetype(&mut self, def: &ic_syntax::ValuetypeDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // TODO: Handle inheritance
        let parent_ty = if let Some(parent_path) = &def.inherits {
            self.resolve_type(&ic_syntax::Type::Path(parent_path.clone()))
                .as_adt()
        } else {
            None
        };

        // Process members
        let mut members = Vec::new();
        for member in &def.members {
            if member.public.is_some() {
                // Public members
                let ty = self.resolve_type(&member.ty);
                members.push(Member {
                    ident: member.ident.clone(),
                    ty,
                    annotations: Vec::new(),
                    default_value: None,
                });
            }
        }

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Valuetype(crate::hir::ValueTy {
                parent: parent_ty,
                extends: None, // TODO: Handle extends
                prototypes: Vec::new(),
                members: Vec::new(), // TODO: ValueTy has Vec<()> for members
                definitions: Vec::new(),
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);

        // Create new scope for valuetype
        let new_scope = self.ctx.scopes.create_child_scope(
            self.current_scope,
            def.ident.name.clone(),
            Some(id),
        );

        // Push to scope stack
        self.scope_path.push(def.ident.name.clone());
        let old_scope = self.current_scope;
        self.current_scope = new_scope;

        // Process nested items and operations
        let mut child_ids = Vec::new();

        for item in &def.definitions {
            let ids = self.process_item(item);
            child_ids.extend(ids);
        }

        // TODO: Process prototypes

        // Update valuetype with children
        if let Def {
            kind: DefKind::Valuetype(vt),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            vt.definitions = child_ids;
        }

        // Pop scope
        self.scope_path.pop();
        self.current_scope = old_scope;

        id
    }

    /// Processes a bitmask definition.
    fn process_bitmask(&mut self, def: &ic_syntax::BitmaskDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Process bits
        let flags = def
            .bits
            .iter()
            .map(|bit| {
                let bit_annotations = self.resolve_ast_annotations(&bit.annotations);
                crate::hir::BitFlag {
                    ident: bit.ident.clone(),
                    value: 0, // Will be filled in evaluation phase
                    annotations: bit_annotations,
                }
            })
            .collect();

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Bitmask(crate::hir::BitmaskTy {
                flags,
                ty: Ty {
                    kind: TyKind::Primitive(PrimitiveTy::UInt32), // Default to unsigned long
                    span: def.span,
                },
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        id
    }

    /// Processes a bitset definition.
    fn process_bitset(&mut self, def: &ic_syntax::BitsetDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Resolve parent if any
        let parent_ty = if let Some(parent_path) = &def.parent {
            self.resolve_type(&ic_syntax::Type::Path(parent_path.clone()))
                .as_adt()
        } else {
            None
        };

        // Process fields
        let fields = def
            .fields
            .iter()
            .map(|field| {
                let field_annotations = self.resolve_ast_annotations(&field.annotations);
                let ty = if let Some(field_ty) = &field.ty {
                    self.resolve_type(field_ty)
                } else {
                    // Use Any as placeholder - evaluation phase will assign correct type based on size
                    Ty {
                        kind: TyKind::Any,
                        span: field.span,
                    }
                };

                crate::hir::BitsetField {
                    ident: field.ident.clone(),
                    ty,
                    size: 0, // Will be filled in evaluation phase
                    annotations: field_annotations,
                }
            })
            .collect();

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Bitset(crate::hir::BitsetTy {
                parent: parent_ty,
                fields,
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), id);

        id
    }

    /// Processes an interface definition.
    fn process_interface(&mut self, def: &ic_syntax::InterfaceDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // TODO: Handle inheritance
        let parents = Vec::new();

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Interface(InterfaceTy {
                parents,
                prototypes: Vec::new(),
                attributes: Vec::new(),
                definitions: Vec::new(),
                is_local: false, // TODO: Determine from annotations
            }),
            flags: DefFlags::default(),
        });

        self.name_map.insert(qualified_name, id);

        // Create new scope for interface
        let new_scope = self.ctx.scopes.create_child_scope(
            self.current_scope,
            def.ident.name.clone(),
            Some(id),
        );

        // Push to scope stack
        self.scope_path.push(def.ident.name.clone());
        let old_scope = self.current_scope;
        self.current_scope = new_scope;

        // Process nested items and operations
        let mut child_ids = Vec::new();
        let prototypes = Vec::new();

        for member in &def.members {
            match member {
                ic_syntax::InterfaceMember::Item(item) => {
                    let ids = self.process_item(item);
                    child_ids.extend(ids);
                }
                ic_syntax::InterfaceMember::Proto(_proto) => {
                    // TODO: Process operations properly
                }
                ic_syntax::InterfaceMember::Attr(_attr) => {
                    // TODO: Process attributes properly
                }
            }
        }

        // Update interface with children
        if let Def {
            kind: DefKind::Interface(iface),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            iface.definitions = child_ids;
            iface.prototypes = prototypes;
        }

        // Pop scope
        self.scope_path.pop();
        self.current_scope = old_scope;

        id
    }

    /// Processes an item and returns the `DefIds` created.
    fn process_item(&mut self, item: &Item) -> Vec<DefId> {
        match item {
            Item::DeclValue(v) => vec![self.process_forward_declaration(v)],
            Item::StructValue(v) => vec![self.process_struct(v)],
            Item::ModuleValue(v) => vec![self.process_module(v)],
            Item::EnumValue(v) => vec![self.process_enum(v)],
            Item::AliasValue(v) => self.process_alias(v),
            Item::UnionValue(v) => vec![self.process_union(v)],
            Item::AnnotationValue(v) => vec![self.process_annotation(v)],
            Item::ExceptionValue(v) => vec![self.process_exception(v)],
            Item::ConstValue(v) => vec![self.process_const(v)],
            Item::InterfaceValue(v) => vec![self.process_interface(v)],
            Item::ValuetypeValue(v) => vec![self.process_valuetype(v)],
            Item::BitmaskValue(v) => vec![self.process_bitmask(v)],
            Item::BitsetValue(v) => vec![self.process_bitset(v)],
        }
    }

    /// Processes all items in order.
    pub fn process(mut self, items: &[Item]) -> (NameMap, Vec<DefId>, Vec<Diag>, Vec<Diag>) {
        for item in items {
            let ids = self.process_item(item);
            self.order.extend(ids);
        }

        (self.name_map, self.order, self.errors, self.warnings)
    }
}

/// Resolves a declarator into (name, type).
fn resolve_declarator(decl: &ic_syntax::Declarator, base_ty: Ty) -> (Ident, Ty) {
    match decl {
        ic_syntax::Declarator::Simple(ident) => (ident.clone(), base_ty),
        ic_syntax::Declarator::Array(arr) => {
            // Build array type from innermost to outermost
            let mut ty = base_ty;
            for bound_expr in &arr.bounds {
                ty = Ty {
                    span: ty.span,
                    kind: TyKind::Array {
                        ty: Box::new(ty.clone()),
                        len: 0, // Will be filled in evaluation phase
                        len_span: ic_syntax::util::expr_span(bound_expr),
                    },
                };
            }
            (arr.ident.clone(), ty)
        }
    }
}

/// Converts a path to a string representation.
fn path_to_string(path: &Path) -> String {
    let mut result = String::new();
    if path.leading_colons.is_some() {
        result.push_str("::");
    }
    let segments: Vec<_> = path.segments.iter().map(|s| s.name.as_str()).collect();
    result.push_str(&segments.join("::"));
    result
}

/// Resolves a primitive type name.
fn resolve_primitive(name: &str) -> Option<PrimitiveTy> {
    Some(match name {
        "boolean" => PrimitiveTy::Bool,
        "octet" => PrimitiveTy::UInt8,
        "char" => PrimitiveTy::Char,
        "wchar" => PrimitiveTy::WChar,
        "short" => PrimitiveTy::Int16,
        "long" => PrimitiveTy::Int32,
        "long long" => PrimitiveTy::Int64,
        "unsigned short" => PrimitiveTy::UInt16,
        "unsigned long" => PrimitiveTy::UInt32,
        "unsigned long long" => PrimitiveTy::UInt64,
        "float" => PrimitiveTy::Float32,
        "double" => PrimitiveTy::Float64,
        "long double" => PrimitiveTy::Float128,
        // Also support explicit integer type names
        "int8" => PrimitiveTy::Int8,
        "uint8" => PrimitiveTy::UInt8,
        "int16" => PrimitiveTy::Int16,
        "uint16" => PrimitiveTy::UInt16,
        "int32" => PrimitiveTy::Int32,
        "uint32" => PrimitiveTy::UInt32,
        "int64" => PrimitiveTy::Int64,
        "uint64" => PrimitiveTy::UInt64,
        _ => return None,
    })
}

/// Extension trait for Ty to extract ADT `DefId`.
trait TyExt {
    fn as_adt(&self) -> Option<DefId>;
}

impl TyExt for Ty {
    fn as_adt(&self) -> Option<DefId> {
        match &self.kind {
            TyKind::Adt(id) => Some(*id),
            _ => None,
        }
    }
}
