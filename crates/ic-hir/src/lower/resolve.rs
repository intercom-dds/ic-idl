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

//! Resolution phase of lowering from AST to HIR.
//!
//! This module processes items in declaration order,
//! ensuring that types can only be used after they have been declared.

use ic_alloc::insensitive::CaseMap;
use ic_cli::color::Colorize;
use ic_diagnostic::{Diag, Label, error_span, warn_span};
use ic_syntax::{Ident, Item, Path};

use super::convert_annotation_value;
use crate::Context;
use crate::hir::{
    AliasTy, Ann, AnnParam, AnnotationTy, Attribute, ConstTy, Decl, Def, DefFlags, DefId, DefKind,
    EnumTy, ExceptTy, InterfaceTy, Member, Numeric, ParamKind, Parameter, PrimitiveTy, ProtoTy,
    StructTy, Ty, TyKind, UnionTy,
};
use crate::scope::ScopeId;

/// Maps fully-qualified names to their `DefIds`.
type NameMap = CaseMap<DefId>;

/// HIR resolver that processes items in order.
pub struct Resolver<'a> {
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

impl<'a> Resolver<'a> {
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
            let Some(def_id) = self.resolve_path(&ann.ident) else {
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
            let params = {
                let def = self.ctx.definitions.get(def_id);
                if let DefKind::Annotation(ann_ty) = &def.kind {
                    ann_ty.params.clone()
                } else {
                    // Not an annotation type
                    continue;
                }
            };

            // Check for multi-parameter annotations with positional arguments
            if params.len() > 1 && ann.args.iter().any(|arg| arg.ident.is_none()) {
                let ann_name = path_to_string(&ann.ident);
                self.warnings.push(warn_span(
                    format!(
                        "@{ann_name} has {} parameters and requires named arguments",
                        params.len()
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
            if params.len() == 1 && ann.args.iter().all(|arg| arg.ident.is_none()) {
                if let Some(arg) = ann.args.first() {
                    // Evaluate the expression (simple literals only for now)
                    let value = convert_annotation_value(&arg.value);
                    args.push(crate::hir::AnnArg {
                        ident: params[0].ident.clone(),
                        value,
                    });
                }
            } else {
                // For named arguments or multiple members, match by name
                for arg in &ann.args {
                    if let Some(name) = &arg.ident {
                        // Find matching member
                        if let Some(param) = params.iter().find(|p| p.ident.name == name.name) {
                            // Evaluate the expression (simple literals only for now)
                            let value = convert_annotation_value(&arg.value);
                            args.push(crate::hir::AnnArg {
                                ident: param.ident.clone(),
                                value,
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
                // Check case consistency for single-segment paths
                self.check_case_consistency(path, def_id);
                return Some(def_id);
            }
        }

        // For multi-segment paths or if local resolution failed, use regular path resolution
        if let Some(def_id) = self.ctx.scopes.resolve_path(start_scope, &segments) {
            // Check case consistency for multi-segment paths
            self.check_case_consistency(path, def_id);
            Some(def_id)
        } else {
            None
        }
    }

    /// Check if a path reference has consistent capitalization with the definition
    fn check_case_consistency(&mut self, path: &Path, def_id: DefId) {
        // For multi-segment paths like foo::Bar, we need to check each segment
        if path.segments.len() > 1 {
            // Since everything must be defined in order, we know that all the module
            // segments in the path have been defined. We can use the scope tree
            // to walk through and verify case consistency.

            let start_scope = if path.leading_colons.is_some() {
                self.ctx.scopes.root()
            } else {
                self.current_scope
            };

            // Start from the beginning and resolve each prefix to check module names
            let mut current_scope = start_scope;

            for segment in &path.segments[..path.segments.len() - 1] {
                // Get the scope for this segment name
                if let Some(&ScopeId(scope_idx)) = self.ctx.scopes.scopes[current_scope.0]
                    .children
                    .get(&segment.name)
                {
                    current_scope = ScopeId(scope_idx);

                    // Get the actual definition for this scope to check its canonical name
                    if let Some(module_def_id) = self.ctx.scopes.scopes[scope_idx].def_id {
                        let module_def = self.ctx.definitions.get(module_def_id);
                        let reference_name = &segment.name;
                        let canonical_name = &module_def.ident.name;

                        if reference_name != canonical_name
                            && reference_name.eq_ignore_ascii_case(canonical_name)
                        {
                            self.warnings.push(
                                warn_span(
                                    format!(
                                        "inconsistent capitalization: `{}` should be `{}`",
                                        reference_name.yellow(),
                                        canonical_name.yellow()
                                    ),
                                    Label::new(segment.span).message("module name used here"),
                                )
                                .note(format!("the canonical module name is `{canonical_name}`")),
                            );
                        }
                    }
                } else {
                    // This shouldn't happen if resolve_path succeeded
                    break;
                }
            }
        }

        // Check the final type name
        let def = self.ctx.definitions.get(def_id);
        if let Some(last_segment) = path.segments.last() {
            let reference_name = &last_segment.name;
            let canonical_name = &def.ident.name;

            // Check if they differ in case
            if reference_name != canonical_name
                && reference_name.eq_ignore_ascii_case(canonical_name)
            {
                self.warnings.push(
                    warn_span(
                        format!(
                            "inconsistent capitalization: `{}` should be `{}`",
                            reference_name.yellow(),
                            canonical_name.yellow()
                        ),
                        Label::new(last_segment.span).message("used here"),
                    )
                    .note(format!("the canonical name is `{canonical_name}`")),
                );
            }
        }
    }

    /// Converts AST type to HIR type.
    fn resolve_type(&mut self, ty: &ic_syntax::Type) -> Ty {
        use ic_syntax::Type;

        match ty {
            Type::Fixed(v) => Self::make_fixed_type(v.span),
            Type::Sequence(v) => self.resolve_sequence_type(v),
            Type::String(v) => Self::resolve_string_type(v),
            Type::Map(v) => self.resolve_map_type(v),
            Type::Path(v) => self.resolve_path_type(v),
        }
    }

    /// Creates a Fixed type.
    fn make_fixed_type(span: ic_syntax::Span) -> Ty {
        Ty {
            kind: TyKind::Fixed,
            span,
        }
    }

    fn make_null_type(span: ic_syntax::Span) -> Ty {
        Ty {
            kind: TyKind::Null,
            span,
        }
    }

    /// Resolves a sequence type.
    fn resolve_sequence_type(&mut self, v: &ic_syntax::SequenceType) -> Ty {
        Ty {
            kind: TyKind::Sequence {
                ty: Box::new(self.resolve_type(&v.ty)),
                bound: None, // Will be filled in evaluation phase
                bound_span: v.bound.as_ref().map(ic_syntax::util::expr_span),
            },
            span: v.span,
        }
    }

    /// Resolves a string type.
    fn resolve_string_type(v: &ic_syntax::StringType) -> Ty {
        Ty {
            kind: TyKind::String {
                wide: v.wide,
                bound: None, // Will be filled in evaluation phase
                bound_span: v.bound.as_ref().map(ic_syntax::util::expr_span),
            },
            span: v.span,
        }
    }

    /// Resolves a map type.
    fn resolve_map_type(&mut self, v: &ic_syntax::MapType) -> Ty {
        Ty {
            kind: TyKind::Map {
                key: Box::new(self.resolve_type(&v.key)),
                elem: Box::new(self.resolve_type(&v.value)),
                bound: None, // Will be filled in evaluation phase
                bound_span: v.bound.as_ref().map(ic_syntax::util::expr_span),
            },
            span: v.span,
        }
    }

    /// Resolves a path type (either primitive or user-defined).
    fn resolve_path_type(&mut self, v: &ic_syntax::Path) -> Ty {
        let span = ic_syntax::util::path_span(v);

        // Check if it's a single identifier
        if v.segments.len() == 1 && v.leading_colons.is_none() {
            let name = &v.segments[0].name;

            // Special case for "any" type
            if name == "any" {
                return Ty {
                    kind: TyKind::Any,
                    span,
                };
            }

            // Check if it's a primitive type
            if let Some(prim) = resolve_primitive(name) {
                return Ty {
                    kind: TyKind::Primitive(prim),
                    span,
                };
            }
        }

        // Try to resolve as user-defined type
        if let Some(id) = self.resolve_path(v) {
            return Ty {
                kind: TyKind::Adt(id),
                span,
            };
        }

        // Type not found - report error
        let qualified = path_to_string(v);
        self.errors.push(error_span(
            format!("unresolved type `{qualified}`"),
            Label::new(span).message("unknown type"),
        ));

        // Return placeholder type - use Null to distinguish from legitimate Any
        Self::make_null_type(span)
    }

    /// Gets the current parent `DefId` based on scope path.
    fn get_current_parent(&self) -> Option<DefId> {
        if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        }
    }

    /// Checks for duplicate struct definition.
    fn check_duplicate_struct(&mut self, qualified_name: &str, def: &ic_syntax::StructDef) {
        if let Some(existing_id) = self.name_map.get(qualified_name).copied() {
            let existing = self.ctx.definitions.get(existing_id);
            if matches!(existing.kind, DefKind::Struct(_)) {
                self.errors.push(
                    error_span(
                        format!("duplicate definition of `{}`", def.ident.name),
                        Label::new(def.ident.span).message("redefined here"),
                    )
                    .label(Label::new(existing.ident.span).message("first defined here")),
                );
            }
        }
    }

    /// Resolves struct parent and validates inheritance.
    fn resolve_struct_parent(&mut self, def: &ic_syntax::StructDef) -> Option<DefId> {
        let parent_path = def.parent.as_ref()?;

        if let Some(parent_id) = self.resolve_path(parent_path) {
            self.validate_struct_parent(parent_id, def)
        } else {
            self.errors.push(error_span(
                format!(
                    "struct `{}` inherits from type that is not defined",
                    def.ident.name
                ),
                Label::new(ic_syntax::util::path_span(parent_path)).message("undefined type"),
            ));
            None
        }
    }

    /// Validates that a parent is a valid struct for inheritance.
    fn validate_struct_parent(
        &mut self,
        parent_id: DefId,
        def: &ic_syntax::StructDef,
    ) -> Option<DefId> {
        let parent_def = self.ctx.definitions.get(parent_id);

        if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
            self.errors.push(
                error_span(
                    format!(
                        "struct `{}` cannot inherit from incomplete type `{}`",
                        def.ident.name, parent_def.ident.name
                    ),
                    Label::new(def.span).message("invalid inheritance"),
                )
                .label(
                    Label::new(parent_def.ident.span)
                        .message("forward declaration here, but no definition found"),
                ),
            );
            None
        } else if matches!(&parent_def.kind, DefKind::Struct(_)) {
            Some(parent_id)
        } else {
            self.errors.push(error_span(
                format!(
                    "struct `{}` cannot inherit from non-struct type `{}`",
                    def.ident.name, parent_def.ident.name
                ),
                Label::new(def.span).message("invalid inheritance"),
            ));
            None
        }
    }

    /// Registers a definition in the name map and current scope.
    fn register_definition(&mut self, qualified_name: String, name: String, id: DefId) {
        self.name_map.insert(qualified_name, id);
        self.ctx.scopes.add_definition(self.current_scope, name, id);
    }

    /// Resolves struct members.
    fn resolve_struct_members(&mut self, fields: &[ic_syntax::Field]) -> Vec<Member> {
        let mut members = Vec::new();

        for field in fields {
            let base_ty = self.resolve_type(&field.ty);
            let field_annotations = self.resolve_ast_annotations(&field.annotations);

            for decl in &field.names {
                let (ident, ty) = resolve_declarator(decl, base_ty.clone());
                members.push(Member {
                    ident,
                    ty,
                    annotations: field_annotations.clone(),
                });
            }
        }

        members
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
            flags: DefFlags::IS_INCOMPLETE, // Forward declarations are incomplete until defined
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
    #[allow(clippy::too_many_lines)]
    fn process_struct(&mut self, def: &ic_syntax::StructDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Check for duplicate definition
        self.check_duplicate_struct(&qualified_name, def);

        // Resolve parent if any
        let parent_id = self.resolve_struct_parent(def);

        // Get parent for scope
        let parent = self.get_current_parent();

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Create placeholder struct so it can be referenced by its own members
        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Struct(StructTy {
                parent: parent_id,
                members: Vec::new(), // Placeholder - will be updated
            }),
            flags: DefFlags::default(),
        });

        // Register struct before resolving members
        self.register_definition(qualified_name, def.ident.name.clone(), id);

        // Resolve members
        let members = self.resolve_struct_members(&def.members);

        // Update struct with resolved members
        let def = self.ctx.definitions.get_mut(id);
        if let DefKind::Struct(struct_ty) = &mut def.kind {
            struct_ty.members = members;
        }

        id
    }

    /// Resolves interface parents and validates inheritance.
    fn resolve_interface_parents(&mut self, def: &ic_syntax::InterfaceDef) -> Vec<DefId> {
        let mut parents = Vec::new();

        for parent_path in &def.inherits {
            if let Some(parent_id) = self.resolve_path(parent_path) {
                if let Some(valid_parent) = self.validate_interface_parent(parent_id, def) {
                    parents.push(valid_parent);
                }
            } else {
                self.errors.push(error_span(
                    format!(
                        "interface `{}` inherits from type that is not defined",
                        def.ident.name
                    ),
                    Label::new(ic_syntax::util::path_span(parent_path)).message("undefined type"),
                ));
            }
        }

        parents
    }

    /// Validates that a parent is a valid interface for inheritance.
    fn validate_interface_parent(
        &mut self,
        parent_id: DefId,
        def: &ic_syntax::InterfaceDef,
    ) -> Option<DefId> {
        let parent_def = self.ctx.definitions.get(parent_id);

        if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
            self.errors.push(
                error_span(
                    format!(
                        "interface `{}` cannot inherit from incomplete type `{}`",
                        def.ident.name, parent_def.ident.name
                    ),
                    Label::new(def.span).message("invalid inheritance"),
                )
                .label(
                    Label::new(parent_def.ident.span)
                        .message("forward declaration here, but no definition found"),
                ),
            );
            None
        } else if matches!(&parent_def.kind, DefKind::Interface(_)) {
            Some(parent_id)
        } else {
            self.errors.push(error_span(
                format!(
                    "interface `{}` cannot inherit from non-interface type `{}`",
                    def.ident.name, parent_def.ident.name
                ),
                Label::new(def.span).message("invalid inheritance"),
            ));
            None
        }
    }

    /// Processes interface members and returns child IDs, prototypes, and attributes.
    fn process_interface_members(
        &mut self,
        interface: &ic_syntax::InterfaceDef,
        members: &[ic_syntax::InterfaceMember],
    ) -> (Vec<DefId>, Vec<ProtoTy>, Vec<Attribute>) {
        let mut child_ids = Vec::new();
        let mut prototypes = Vec::new();
        let mut attributes = Vec::new();

        // Track method names for duplicate detection (case-insensitive)
        let mut seen_methods = CaseMap::<ic_syntax::Span>::new();

        for member in members {
            match member {
                ic_syntax::InterfaceMember::Item(item) => {
                    let ids = self.process_item(item);
                    child_ids.extend(ids);
                }
                ic_syntax::InterfaceMember::Proto(proto) => {
                    // Check for duplicate method names (case-insensitive)
                    if let Some(&first_span) = seen_methods.get(&proto.ident.name) {
                        self.errors.push(
                            error_span(
                                format!(
                                    "duplicate method `{}` in interface '{}'",
                                    proto.ident.name, interface.ident.name,
                                ),
                                Label::new(proto.ident.span).message("duplicate method"),
                            )
                            .label(Label::new(first_span).message("first defined here"))
                            .note("method names are case-insensitive"),
                        );
                    } else {
                        seen_methods.insert(proto.ident.name.clone(), proto.ident.span);
                    }

                    let proto_ty = self.process_prototype(proto);
                    prototypes.push(proto_ty);
                }
                ic_syntax::InterfaceMember::Attr(attr) => {
                    let attribute_ty = self.process_attribute(attr);
                    attributes.push(attribute_ty);
                }
            }
        }

        (child_ids, prototypes, attributes)
    }

    /// Checks if a module already exists and returns its scope if so.
    fn check_existing_module(&mut self, def: &ic_syntax::ModuleDef) -> Option<ScopeId> {
        let existing_scope_id = self.ctx.scopes.scopes[self.current_scope.0]
            .children
            .get(&def.ident.name)
            .copied()?;

        // Check case consistency
        if let Some(existing_def_id) = self.ctx.scopes.scopes[existing_scope_id.0].def_id {
            let existing_def = self.ctx.definitions.get(existing_def_id);
            if def.ident.name != existing_def.ident.name
                && def
                    .ident
                    .name
                    .eq_ignore_ascii_case(&existing_def.ident.name)
            {
                self.warnings.push(
                    warn_span(
                        format!(
                            "inconsistent capitalization: module `{}` was previously defined as \
                             `{}`",
                            def.ident.name.yellow(),
                            existing_def.ident.name.yellow()
                        ),
                        Label::new(def.ident.span).message("module reopened here"),
                    )
                    .label(Label::new(existing_def.ident.span).message("first defined here")),
                );
            }
        }

        Some(existing_scope_id)
    }

    /// Processes module items and returns child IDs.
    fn process_module_items(&mut self, items: &[Item]) -> Vec<DefId> {
        let mut child_ids = Vec::new();
        for item in items {
            let ids = self.process_item(item);
            child_ids.extend(ids);
        }
        child_ids
    }

    /// Processes a module definition.
    fn process_module(&mut self, def: &ic_syntax::ModuleDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);
        let annotations = self.resolve_ast_annotations(&def.annotations);
        let parent = self.get_current_parent();

        // Create module definition
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

        // Register in name map
        self.name_map.insert(qualified_name, id);

        // Check for existing module or create new scope
        let module_scope = self.check_existing_module(def).unwrap_or_else(|| {
            self.ctx
                .scopes
                .create_child_scope(self.current_scope, def.ident.name.clone(), Some(id))
        });

        // Save current state
        let old_scope = self.current_scope;
        self.scope_path.push(def.ident.name.clone());
        self.current_scope = module_scope;

        // Process nested items
        let child_ids = self.process_module_items(&def.definitions);

        // Update module with children
        if let DefKind::Module(module) = &mut self.ctx.definitions.get_mut(id).kind {
            module.definitions = child_ids;
        }

        // Restore state
        self.scope_path.pop();
        self.current_scope = old_scope;

        id
    }

    /// Processes an enum definition.
    fn process_enum(&mut self, def: &ic_syntax::EnumDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        // Determine the underlying type (defaults to Int32)
        let underlying_ty = Ty {
            kind: TyKind::Primitive(PrimitiveTy::Int32),
            span: def.span,
        };

        // Create constants for each enumerator
        let mut field_ids = Vec::new();
        for field in &def.fields {
            let field_qualified_name = self.qualified_name(&field.ident.name);
            let field_annotations = self.resolve_ast_annotations(&field.annotations);

            // Create a constant definition for this enumerator
            let field_id = self.ctx.definitions.alloc_with_id(|id| Def {
                id,
                ident: field.ident.clone(),
                parent: Some(id), // Will be fixed below
                annotations: field_annotations,
                span: field.ident.span,
                kind: DefKind::Const(ConstTy {
                    value: Numeric::Int32(0),  // Will be filled in evaluation phase
                    ty: underlying_ty.clone(), // Enum constants have the enum's underlying type
                }),
                flags: DefFlags::default(),
            });

            field_ids.push(field_id);
            self.name_map.insert(field_qualified_name, field_id);
            self.ctx
                .scopes
                .add_definition(self.current_scope, field.ident.name.clone(), field_id);
        }

        // Create the enum definition
        let enum_id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Enum(EnumTy {
                fields: field_ids.clone(),
                ty: Ty {
                    kind: TyKind::Primitive(PrimitiveTy::Int32), // Default to int32
                    span: def.span,
                },
            }),
            flags: DefFlags::default(),
        });

        // Fix the parent references for the enumerator constants
        for field_id in &field_ids {
            let field_def = self.ctx.definitions.get_mut(*field_id);
            field_def.parent = Some(enum_id);
        }

        self.name_map.insert(qualified_name, enum_id);
        self.ctx
            .scopes
            .add_definition(self.current_scope, def.ident.name.clone(), enum_id);

        enum_id
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

                    // Check if this is a default case
                    let is_default = field
                        .labels
                        .iter()
                        .any(|label| matches!(label, ic_syntax::Label::Default(_)));

                    // Process case labels (we'll store them as DefIds later in evaluation phase)
                    let labels = Vec::new(); // Labels will be evaluated in the evaluation phase

                    variants.push(crate::hir::Variant {
                        ident,
                        ty,
                        annotations: field_annotations,
                        labels,
                        is_default,
                    });
                }
                ic_syntax::UnionElement::Null(null_elem) => {
                    // Generate a synthetic identifier for the null case based on its position
                    let ident = Ident {
                        name: format!("_null_case_{}", variants.len()),
                        span: null_elem.span,
                    };

                    // Check if this is a default case
                    let is_default = field
                        .labels
                        .iter()
                        .any(|label| matches!(label, ic_syntax::Label::Default(_)));

                    // Process case labels (will be evaluated in the evaluation phase)
                    let labels = Vec::new();

                    // Use a null type for null cases
                    let null_ty = Self::make_null_type(null_elem.span);
                    variants.push(crate::hir::Variant {
                        ident,
                        ty: null_ty,
                        annotations: self.resolve_ast_annotations(&field.annotations),
                        labels,
                        is_default,
                    });
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
                params: Vec::new(),
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
        let mut params = Vec::new();
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
                    params.push(AnnParam {
                        ident,
                        ty,
                        default: None, // Will be resolved in evaluation phase
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
            ann.params = params;
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
    #[allow(clippy::too_many_lines)]
    fn process_valuetype(&mut self, def: &ic_syntax::ValuetypeDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);

        // Resolve annotations
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Handle inheritance
        let parent_ty = if let Some(parent_path) = &def.inherits {
            if let Some(parent_id) = self.resolve_path(parent_path) {
                let parent_def = self.ctx.definitions.get(parent_id);

                // If we found a forward declaration, check if there's an actual definition
                if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                    // Look for the actual definition by checking all definitions with the same name
                    let parent_name = &parent_def.ident.name;
                    let mut found_definition = None;

                    // Search through all definitions to find a non-forward declaration
                    for (def_id, def) in self.ctx.definitions.iter() {
                        if def.ident.name == *parent_name
                            && !def.flags.contains(DefFlags::IS_INCOMPLETE)
                        {
                            if matches!(&def.kind, DefKind::Valuetype(_)) {
                                found_definition = Some(def_id);
                                break;
                            }
                        }
                    }

                    if let Some(actual_parent_id) = found_definition {
                        // Found the actual definition, use it
                        Some(actual_parent_id)
                    } else {
                        // No actual definition found, only forward declaration
                        self.errors.push(
                            error_span(
                                format!(
                                    "valuetype `{}` cannot inherit from incomplete type `{}`",
                                    def.ident.name, parent_def.ident.name
                                ),
                                Label::new(def.span).message("invalid inheritance"),
                            )
                            .label(
                                Label::new(parent_def.ident.span)
                                    .message("forward declaration here, but no definition found"),
                            ),
                        );
                        None
                    }
                } else {
                    // Not a forward declaration, check that parent is actually a valuetype
                    if matches!(&parent_def.kind, DefKind::Valuetype(_)) {
                        Some(parent_id)
                    } else {
                        self.errors.push(error_span(
                            format!(
                                "valuetype `{}` cannot inherit from non-valuetype type `{}`",
                                def.ident.name, parent_def.ident.name
                            ),
                            Label::new(def.span).message("invalid inheritance"),
                        ));
                        None
                    }
                }
            } else {
                self.errors.push(error_span(
                    format!(
                        "valuetype `{}` inherits from type that is not defined",
                        def.ident.name
                    ),
                    Label::new(ic_syntax::util::path_span(parent_path)).message("undefined type"),
                ));
                None
            }
        } else {
            None
        };

        // Resolve supports interface
        let supports_ty = if let Some(supports_path) = &def.supports {
            if let Some(supports_id) = self.resolve_path(supports_path) {
                let supports_def = self.ctx.definitions.get(supports_id);

                // Check if supports is an interface
                if matches!(&supports_def.kind, DefKind::Interface(_)) {
                    Some(supports_id)
                } else {
                    self.errors.push(error_span(
                        format!(
                            "valuetype `{}` cannot support non-interface type `{}`",
                            def.ident.name, supports_def.ident.name
                        ),
                        Label::new(ic_syntax::util::path_span(supports_path))
                            .message("not an interface"),
                    ));
                    None
                }
            } else {
                self.errors.push(error_span(
                    format!(
                        "valuetype `{}` supports interface that is not defined",
                        def.ident.name
                    ),
                    Label::new(ic_syntax::util::path_span(supports_path))
                        .message("undefined interface"),
                ));
                None
            }
        } else {
            None
        };

        let parent = if self.scope_path.is_empty() {
            None
        } else {
            let parent_name = self.scope_path.join("::");
            self.name_map.get(&parent_name).copied()
        };

        // Create the valuetype definition first with empty members
        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Valuetype(crate::hir::ValueTy {
                parent: parent_ty,
                supports: supports_ty,
                prototypes: Vec::new(),
                attributes: Vec::new(),
                members: Vec::new(),
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

        // Process elements in declaration order
        let mut child_ids = Vec::new();
        let mut members = Vec::new();
        let mut prototypes = Vec::new();
        let mut attributes = Vec::new();

        for element in &def.elements {
            match element {
                ic_syntax::ValueElement::Item(item) => {
                    // Process nested type definitions
                    let ids = self.process_item(item);
                    child_ids.extend(ids);
                }
                ic_syntax::ValueElement::State(member) => {
                    if member.is_public {
                        // Public members - types defined before this point should be resolvable
                        let ty = self.resolve_type(&member.ty);
                        for decl in &member.decl {
                            if let ic_syntax::Declarator::Simple(ident) = decl {
                                members.push(Member {
                                    ident: ident.clone(),
                                    ty: ty.clone(),
                                    annotations: Vec::new(),
                                });
                            } else {
                                // TODO: Handle array declarators
                            }
                        }
                    }
                }
                ic_syntax::ValueElement::Proto(proto) => {
                    let proto_ty = self.process_prototype(proto);
                    prototypes.push(proto_ty);
                }
                ic_syntax::ValueElement::Attr(attr) => {
                    let attr_ty = self.process_attribute(attr);
                    attributes.push(attr_ty);
                }
            }
        }

        // Update valuetype with children
        if let Def {
            kind: DefKind::Valuetype(vt),
            ..
        } = self.ctx.definitions.get_mut(id)
        {
            vt.definitions = child_ids;
            vt.members = members;
            vt.prototypes = prototypes;
            vt.attributes = attributes;
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
                    // Use Null as placeholder - evaluation phase will assign correct type based on size
                    Self::make_null_type(field.span)
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
    #[allow(clippy::too_many_lines)]
    fn process_interface(&mut self, def: &ic_syntax::InterfaceDef) -> DefId {
        let qualified_name = self.qualified_name(&def.ident.name);
        let annotations = self.resolve_ast_annotations(&def.annotations);
        let parent = self.get_current_parent();

        // Create interface with empty collections
        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            parent,
            annotations,
            span: def.span,
            kind: DefKind::Interface(InterfaceTy {
                parents: Vec::new(),
                prototypes: Vec::new(),
                attributes: Vec::new(),
                definitions: Vec::new(),
                is_local: false, // TODO: Determine from annotations
            }),
            flags: DefFlags::default(),
        });

        // Register before resolving inheritance
        self.register_definition(qualified_name, def.ident.name.clone(), id);

        // Resolve parents after interface is registered
        let parents = self.resolve_interface_parents(def);

        // Update interface with resolved parents
        if let DefKind::Interface(iface) = &mut self.ctx.definitions.get_mut(id).kind {
            iface.parents = parents;
        }

        // Create scope for interface members
        let new_scope = self.ctx.scopes.create_child_scope(
            self.current_scope,
            def.ident.name.clone(),
            Some(id),
        );

        // Save current state
        let old_scope = self.current_scope;
        self.scope_path.push(def.ident.name.clone());
        self.current_scope = new_scope;

        // Process members
        let (child_ids, prototypes, attributes) = self.process_interface_members(def, &def.members);

        // Update interface with processed members
        if let DefKind::Interface(iface) = &mut self.ctx.definitions.get_mut(id).kind {
            iface.definitions = child_ids;
            iface.prototypes = prototypes;
            iface.attributes = attributes;
        }

        // Restore state
        self.scope_path.pop();
        self.current_scope = old_scope;

        id
    }

    /// Processes a prototype (method) definition.
    fn process_prototype(&mut self, proto: &ic_syntax::Prototype) -> ProtoTy {
        // Resolve return type
        let ret_ty = self.resolve_type(&proto.ret);

        // Process parameters
        let mut params = Vec::new();
        for param in &proto.params {
            let param_ty = self.resolve_type(&param.ty);

            // Extract parameter name from declarator
            let param_ident = match &param.decl {
                ic_syntax::Declarator::Simple(ident) => ident.clone(),
                ic_syntax::Declarator::Array(arr) => arr.ident.clone(),
            };

            params.push(Parameter {
                ident: param_ident,
                ty: param_ty,
                kind: match &param.kind {
                    Some(ic_syntax::ParamKind::In) | None => ParamKind::In, // Default to In
                    Some(ic_syntax::ParamKind::Out) => ParamKind::Out,
                    Some(ic_syntax::ParamKind::Inout) => ParamKind::Inout,
                },
            });
        }

        ProtoTy {
            ident: proto.ident.clone(),
            ty: ret_ty,
            params,
        }
    }

    /// Processes an attribute definition.
    fn process_attribute(&mut self, attr: &ic_syntax::Attribute) -> Attribute {
        let ty = self.resolve_type(&attr.ty);

        // Process declarators
        let mut attributes = Vec::new();
        for decl in &attr.decl {
            if let ic_syntax::Declarator::Simple(ident) = decl {
                attributes.push(Attribute {
                    ident: ident.clone(),
                    ty: ty.clone(),
                    is_readonly: attr.readonly.is_some(),
                    getraises: Vec::new(), // TODO: Process getraises exceptions
                    setraises: Vec::new(), // TODO: Process setraises exceptions
                });
            } else {
                // TODO: Handle array declarators
            }
        }

        // Return the first attribute (for now)
        // TODO: Handle multiple declarators properly
        attributes.into_iter().next().unwrap_or(Attribute {
            ident: ic_syntax::Ident {
                name: String::new(),
                span: ic_syntax::util::ty_span(&attr.ty),
            },
            ty,
            is_readonly: attr.readonly.is_some(),
            getraises: Vec::new(),
            setraises: Vec::new(),
        })
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
    pub fn process(mut self, items: &[Item]) -> (Vec<DefId>, Vec<Diag>, Vec<Diag>) {
        for item in items {
            let ids = self.process_item(item);
            self.order.extend(ids);
        }

        (self.order, self.errors, self.warnings)
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
        "void" => PrimitiveTy::Void,
        "boolean" => PrimitiveTy::Bool,
        "octet" | "uint8" => PrimitiveTy::UInt8,
        "int8" => PrimitiveTy::Int8,
        "int16" => PrimitiveTy::Int16,
        "uint16" => PrimitiveTy::UInt16,
        "int32" => PrimitiveTy::Int32,
        "uint32" => PrimitiveTy::UInt32,
        "int64" => PrimitiveTy::Int64,
        "uint64" => PrimitiveTy::UInt64,
        "char" => PrimitiveTy::Char,
        "wchar" => PrimitiveTy::WChar,
        "float" => PrimitiveTy::Float32,
        "double" => PrimitiveTy::Float64,
        "long double" => PrimitiveTy::Float128,
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
