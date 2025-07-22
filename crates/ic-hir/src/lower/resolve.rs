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

//! Phase 2: Type resolution.
//!
//! This phase walks the AST again and:
//! - Resolves all type references (Path -> `DefId`)
//! - Fills in struct members, union variants, etc.
//! - Resolves inheritance relationships
//! - Does NOT evaluate constant expressions yet

use std::collections::HashMap;

use ic_cli::color::Colorize;
use ic_diagnostic::{Diag, Label, error_span, warn_span};
use ic_syntax::{Item, Path, Span};

use super::collect::NameMap;
use crate::Context;
use crate::hir::{
    Ann, BitFlag, BitsetField, DefFlags, DefId, DefKind, Ident, Member, ParamKind, Parameter,
    PrimitiveTy, ProtoTy, Ty, TyKind, Variant,
};
use crate::scope::ScopeId;

/// Resolves type references in the HIR.
pub struct TypeResolver<'a> {
    ctx: &'a mut Context,
    name_map: &'a NameMap,
    errors: Vec<Diag>,
    warnings: Vec<Diag>,
    /// Maps AST items to their `DefIds` for easy lookup.
    item_map: HashMap<ItemKey, DefId>,
    /// Current scope for resolving unqualified names.
    current_scope: Vec<String>,
    /// Current scope ID in the scope tree.
    current_scope_id: crate::scope::ScopeId,
}

/// Key for looking up items by their AST identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ItemKey {
    name: String,
    kind: &'static str,
}

impl<'a> TypeResolver<'a> {
    fn new(ctx: &'a mut Context, name_map: &'a NameMap, _ast_items: &'a [Item]) -> Self {
        let root_scope = ctx.scopes.root();
        Self {
            ctx,
            name_map,
            errors: Vec::new(),
            warnings: Vec::new(),
            item_map: HashMap::new(),
            current_scope: Vec::new(),
            current_scope_id: root_scope,
        }
    }

    /// Resolves annotations from AST and returns only those that could be resolved.
    /// Unresolved annotations are filtered out with warnings.
    #[allow(clippy::too_many_lines)]
    fn resolve_ast_annotations(
        &mut self,
        ast_annotations: &[ic_syntax::AnnotationAppl],
    ) -> Vec<Ann> {
        let mut resolved_annotations = Vec::new();

        for ast_ann in ast_annotations {
            // Convert the path to a single identifier
            let name = ast_ann
                .ident
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            let ident = crate::hir::Ident {
                name: name.clone(),
                span: ic_syntax::util::path_span(&ast_ann.ident),
            };

            // Try to resolve the annotation name
            // Get the path segments
            let segments: Vec<&str> = ast_ann
                .ident
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect();

            let mut def_id = None;

            // If it's a qualified path (e.g., M::custom), resolve from root
            if segments.len() > 1 {
                def_id = self
                    .ctx
                    .scopes
                    .resolve_path(self.ctx.scopes.root(), &segments);

                // Special handling for annotations in ext namespace (e.g., @ext::no_serializer)
                // These should resolve to intercom::annotations::ext::no_serializer
                if def_id.is_none() && segments[0] == "ext" {
                    let mut full_path = vec!["intercom", "annotations"];
                    full_path.extend(&segments);
                    def_id = self
                        .ctx
                        .scopes
                        .resolve_path(self.ctx.scopes.root(), &full_path);
                }
            } else {
                // Single segment - first try intercom::annotations for built-in annotations
                if segments.len() == 1 {
                    let intercom_path = vec!["intercom", "annotations", &name];
                    def_id = self
                        .ctx
                        .scopes
                        .resolve_path(self.ctx.scopes.root(), &intercom_path);
                }

                // If not found in intercom::annotations, try current scope and parent scopes
                if def_id.is_none() {
                    let mut scope_id = self.current_scope_id;

                    // Walk up the scope chain looking for the annotation
                    loop {
                        def_id = self.ctx.scopes.resolve_path(scope_id, &segments);
                        if def_id.is_some() {
                            break;
                        }

                        // Move to parent scope
                        let scope_data = self.ctx.scopes.get_scope(scope_id);
                        if let Some(parent) = scope_data.parent {
                            scope_id = parent;
                        } else {
                            def_id = None;
                            break; // Reached root scope
                        }
                    }
                }
            }

            if let Some(id) = def_id {
                // Verify it's an annotation definition and get members
                let members = {
                    let def = self.ctx.definitions.get(id);
                    if let DefKind::Annotation(ann_ty) = &def.kind {
                        Some(ann_ty.members.clone())
                    } else {
                        None
                    }
                };

                if let Some(members) = members {
                    // Process and validate arguments
                    let args = self.process_annotation_args(
                        &ast_ann.args,
                        &members,
                        &name,
                        ic_syntax::util::path_span(&ast_ann.ident),
                    );

                    let ann = Ann {
                        ident,
                        def_id: id,
                        args,
                    };
                    resolved_annotations.push(ann);
                } else {
                    self.warnings.push(warn_span(
                        format!("'{name}' is not an annotation"),
                        Label::new(ident.span).message("expected an @annotation definition"),
                    ));
                    // Don't include non-annotation types
                }
            } else {
                // Annotation not found - emit warning and exclude from HIR
                self.warnings.push(warn_span(
                    format!("unknown annotation '{name}'"),
                    Label::new(ident.span).message("annotation not found"),
                ));
                // Don't include unresolved annotations
            }
        }

        resolved_annotations
    }

    /// Finds the span of the failing segment in a path by resolving incrementally.
    /// Check if a path reference has consistent capitalization with the definition
    fn check_case_consistency(&mut self, path: &Path, def_id: DefId) {
        // For multi-segment paths like foo::Bar, we need to check each segment
        if path.segments.len() > 1 {
            // Check module segments
            let start_scope = if path.leading_colons.is_some() {
                self.ctx.scopes.root()
            } else {
                self.current_scope_id
            };

            // Check each segment except the last one (which is the type name)
            for (i, segment) in path.segments[..path.segments.len() - 1].iter().enumerate() {
                let segments_so_far: Vec<&str> = path.segments[..=i]
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect();

                // Try to resolve this prefix
                if let Some(module_id) = self.ctx.scopes.resolve_path(start_scope, &segments_so_far)
                {
                    let module_def = self.ctx.definitions.get(module_id);
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

    fn find_failing_segment(&self, start_scope: ScopeId, path: &Path) -> Span {
        // Try resolving prefixes of the path to find where it fails
        for i in 1..=path.segments.len() {
            let prefix_segments: Vec<&str> =
                path.segments[..i].iter().map(|s| s.name.as_str()).collect();

            if self
                .ctx
                .scopes
                .resolve_path(start_scope, &prefix_segments)
                .is_none()
            {
                // This segment failed - return its span
                return path.segments[i - 1].span;
            }
        }

        // Fallback to the whole path span
        ic_syntax::util::path_span(path)
    }

    /// Resolves a path to a `DefId`.
    fn resolve_path(&mut self, path: &Path) -> Option<DefId> {
        // Convert path segments to string slice
        let segments: Vec<&str> = path.segments.iter().map(|s| s.name.as_str()).collect();

        // If path has leading colons (::), resolve from global scope
        let start_scope = if path.leading_colons.is_some() {
            self.ctx.scopes.root()
        } else {
            self.current_scope_id
        };

        // For single-segment paths, use visibility-aware resolution
        if segments.len() == 1 && path.leading_colons.is_none() {
            if let Some(def_id) = self.ctx.scopes.resolve_name_with_visibility(
                start_scope,
                segments[0],
                &self.ctx.definitions,
            ) {
                return Some(def_id);
            }

            // Check if it's a primitive type before giving up
            if resolve_primitive(segments[0]).is_some() {
                return None; // Let resolve_type handle primitives
            }

            // Not a primitive and not found - report error
            self.errors.push(error_span(
                format!("unresolved type `{}`", segments[0]),
                Label::new(path.segments[0].span).message("unknown type"),
            ));
            return None;
        }

        // For multi-segment paths, use regular resolution
        if let Some(def_id) = self.ctx.scopes.resolve_path(start_scope, &segments) {
            return Some(def_id);
        }

        // Try to find which segment failed by resolving incrementally
        let failing_segment_span = self.find_failing_segment(start_scope, path);

        // Report error with specific segment span
        let qualified = path_to_string(path);
        self.errors.push(error_span(
            format!("unresolved type `{qualified}`"),
            Label::new(failing_segment_span).message("unknown type"),
        ));
        None
    }

    /// Resolves an AST type to a HIR type.
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
                // Check for primitive types first
                if v.segments.len() == 1 && v.leading_colons.is_none() {
                    if let Some(prim) = resolve_primitive(&v.segments[0].name) {
                        return Ty {
                            kind: TyKind::Primitive(prim),
                            span: ic_syntax::util::path_span(v),
                        };
                    }
                }

                // Otherwise resolve as user-defined type
                if let Some(id) = self.resolve_path(v) {
                    // Check for case consistency
                    self.check_case_consistency(v, id);

                    Ty {
                        kind: TyKind::Adt(id),
                        span: ic_syntax::util::path_span(v),
                    }
                } else {
                    // Error already reported by resolve_path
                    // Return a placeholder type to continue processing
                    Ty {
                        kind: TyKind::Any,
                        span: ic_syntax::util::path_span(v),
                    }
                }
            }
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

    /// Resolves struct members.
    fn resolve_struct_members(&mut self, def: &ic_syntax::StructDef) -> Vec<Member> {
        let mut members = Vec::new();

        for field in &def.members {
            let base_ty = self.resolve_type(&field.ty);

            for decl in &field.names {
                let (ident, ty) = Self::resolve_declarator(decl, base_ty.clone());
                let annotations = self.resolve_ast_annotations(&field.annotations);
                members.push(Member {
                    ident,
                    ty,
                    annotations,
                    default_value: None,
                });
            }
        }

        members
    }

    /// Marks any forward declarations of the given type as resolved.
    fn mark_forward_declarations_resolved(&mut self, name: &str) {
        // Find all definitions with this name
        for (_, def) in &mut self.ctx.definitions {
            if def.ident.name == name && matches!(def.kind, DefKind::Decl(_)) {
                // This is a forward declaration - mark it as complete since we found the definition
                def.flags.unset(DefFlags::IS_INCOMPLETE);
            }
        }
    }

    /// Finds the span of a forward declaration for the given type name.
    fn find_forward_declaration_span(&self, name: &str) -> Option<Span> {
        // First try with the simple name
        if let Some((_, span)) = self.ctx.forward_declarations.get(name) {
            return Some(*span);
        }

        // If not found, try to find by searching all entries
        // This handles cases where we have the simple name but the map has qualified names
        for (key, (_, span)) in self.ctx.forward_declarations.iter() {
            if key.ends_with(name) && (key == name || key.ends_with(&format!("::{name}"))) {
                return Some(*span);
            }
        }

        None
    }

    /// Resolves a struct definition.
    fn resolve_struct(&mut self, id: DefId, def: &ic_syntax::StructDef) {
        // Mark any forward declarations as resolved
        self.mark_forward_declarations_resolved(&def.ident.name);

        let parent = if let Some(parent_path) = &def.parent {
            if let Some(parent_id) = self.resolve_path(parent_path) {
                // Check if parent type is complete
                let parent_def = self.ctx.definitions.get(parent_id);
                if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                    // Find the forward declaration to point to in the error
                    let forward_decl_span = self
                        .find_forward_declaration_span(&parent_def.ident.name)
                        .unwrap_or(parent_def.ident.span);

                    self.errors.push(
                        error_span(
                            format!(
                                "struct `{}` cannot inherit from incomplete type `{}`",
                                def.ident.name, parent_def.ident.name
                            ),
                            Label::new(def.ident.span).message("invalid inheritance"),
                        )
                        .label(
                            Label::new(forward_decl_span)
                                .message("type is not yet defined at this point"),
                        ),
                    );
                    None
                } else {
                    Some(parent_id)
                }
            } else {
                None
            }
        } else {
            None
        };

        let members = self.resolve_struct_members(def);

        // Resolve annotations for the struct itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Update the definition
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);
        hir_def.annotations = annotations;

        if let DefKind::Struct(struct_ty) = &mut hir_def.kind {
            struct_ty.parent = parent;
            struct_ty.members = members;
        }
    }

    /// Resolves union variants.
    fn resolve_union(&mut self, id: DefId, def: &ic_syntax::UnionDef) {
        // Mark any forward declarations as resolved
        self.mark_forward_declarations_resolved(&def.ident.name);
        let disc = self.resolve_type(&def.disc.ty);
        let mut variants = Vec::new();

        for field in &def.fields {
            use ic_syntax::UnionElement;

            let variant = match &field.field {
                UnionElement::Member(m) => {
                    let base_ty = self.resolve_type(&m.ty);
                    let (ident, ty) = Self::resolve_declarator(&m.decl, base_ty);

                    let annotations = self.resolve_ast_annotations(&field.annotations);

                    Variant {
                        annotations,
                        ident,
                        ty,
                        labels: Vec::new(), // Will be filled in evaluation phase
                        is_default: field
                            .labels
                            .iter()
                            .any(|l| matches!(l, ic_syntax::Label::Default(_))),
                    }
                }
                UnionElement::Null(n) => {
                    Variant {
                        annotations: Vec::new(),
                        ident: Ident {
                            name: "null".to_string(),
                            span: n.span,
                        },
                        ty: Ty {
                            kind: TyKind::Any,
                            span: n.span,
                        },
                        labels: Vec::new(), // Will be filled in evaluation phase
                        is_default: field
                            .labels
                            .iter()
                            .any(|l| matches!(l, ic_syntax::Label::Default(_))),
                    }
                }
            };

            variants.push(variant);
        }

        // Resolve annotations for the union itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Update the definition
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);
        hir_def.annotations = annotations;

        if let DefKind::Union(union_ty) = &mut hir_def.kind {
            union_ty.disc = disc;
            union_ty.variants = variants;
        }
    }

    /// Resolves an exception definition.
    fn resolve_exception(&mut self, id: DefId, def: &ic_syntax::ExceptDef) {
        let members = self.resolve_struct_members(&ic_syntax::StructDef {
            ident: def.ident.clone(),
            parent: None,
            members: def.members.clone(),
            annotations: def.annotations.clone(),
            span: def.span,
        });

        // Resolve annotations for the exception itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);
        hir_def.annotations = annotations;

        if let DefKind::Except(except_ty) = &mut hir_def.kind {
            except_ty.members = members;
        }
    }

    /// Resolves an alias definition.
    fn resolve_alias(&mut self, id: DefId, def: &ic_syntax::AliasDef, decl_idx: usize) {
        let base_ty = self.resolve_type(&def.ty);
        let (_, ty) = Self::resolve_declarator(&def.decl[decl_idx], base_ty);

        // Resolve annotations for the alias itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);
        hir_def.annotations = annotations;

        if let DefKind::Alias(alias_ty) = &mut hir_def.kind {
            alias_ty.ty = ty;
        }
    }

    /// Resolves an interface definition.
    fn resolve_interface(&mut self, id: DefId, def: &ic_syntax::InterfaceDef) {
        // Mark any forward declarations as resolved
        self.mark_forward_declarations_resolved(&def.ident.name);

        // Save current scope
        let saved_scope_id = self.current_scope_id;

        // Enter interface scope
        if let Some(interface_scope) = self.ctx.scopes.find_scope_for_def(id) {
            self.current_scope_id = interface_scope;
        }

        // Resolve annotations for the interface itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        // Resolve nested type definitions first
        let mut nested_items = Vec::new();
        for member in &def.members {
            if let ic_syntax::InterfaceMember::Item(item) = member {
                nested_items.push(item.clone());
            }
        }
        if !nested_items.is_empty() {
            self.resolve_all(&nested_items);
        }

        let parents = def
            .inherits
            .iter()
            .filter_map(|p| {
                if let Some(parent_id) = self.resolve_path(p) {
                    // Check if parent type is complete
                    let parent_def = self.ctx.definitions.get(parent_id);
                    if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                        // Find the forward declaration to point to in the error
                        let forward_decl_span = self
                            .find_forward_declaration_span(&parent_def.ident.name)
                            .unwrap_or(parent_def.ident.span);

                        self.errors.push(
                            error_span(
                                format!(
                                    "interface `{}` cannot inherit from incomplete type `{}`",
                                    def.ident.name, parent_def.ident.name
                                ),
                                Label::new(def.ident.span).message("invalid inheritance"),
                            )
                            .label(
                                Label::new(forward_decl_span)
                                    .message("type is not yet defined at this point"),
                            ),
                        );
                        None
                    } else {
                        Some(parent_id)
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut prototypes = Vec::new();

        for member in &def.members {
            if let ic_syntax::InterfaceMember::Proto(proto) = member {
                let ret_ty = self.resolve_type(&proto.ret);
                let mut params = Vec::new();

                for param in &proto.params {
                    let param_ty = self.resolve_type(&param.ty);
                    let (ident, ty) = Self::resolve_declarator(&param.decl, param_ty);

                    params.push(Parameter {
                        ident,
                        ty,
                        kind: param.kind.unwrap_or(ParamKind::In),
                    });
                }

                prototypes.push(ProtoTy {
                    ident: proto.ident.clone(),
                    ty: ret_ty,
                    params,
                });
            }
        }

        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);
        hir_def.annotations = annotations;

        if let DefKind::Interface(interface) = &mut hir_def.kind {
            interface.parents = parents;
            interface.prototypes = prototypes;
        }

        // Restore scope
        self.current_scope_id = saved_scope_id;
    }

    /// Builds a mapping from AST items to their `DefIds`.
    fn build_item_map(&mut self, items: &[Item]) {
        self.build_item_map_with_scope(items, &Vec::new());
    }

    /// Builds a mapping from AST items to their `DefIds` with a given scope.
    fn build_item_map_with_scope(&mut self, items: &[Item], scope: &[String]) {
        for item in items {
            let (name, kind, nested_items): (String, &str, Option<Vec<Item>>) = match item {
                Item::StructValue(v) => (v.ident.name.clone(), "struct", None),
                Item::UnionValue(v) => (v.ident.name.clone(), "union", None),
                Item::EnumValue(v) => (v.ident.name.clone(), "enum", None),
                Item::ExceptionValue(v) => (v.ident.name.clone(), "exception", None),
                Item::BitmaskValue(v) => (v.ident.name.clone(), "bitmask", None),
                Item::BitsetValue(v) => (v.ident.name.clone(), "bitset", None),
                Item::InterfaceValue(v) => {
                    // Extract nested items from interface members
                    let nested_items: Vec<Item> = v
                        .members
                        .iter()
                        .filter_map(|m| match m {
                            ic_syntax::InterfaceMember::Item(item) => Some(item.clone()),
                            _ => None,
                        })
                        .collect();
                    (
                        v.ident.name.clone(),
                        "interface",
                        if nested_items.is_empty() {
                            None
                        } else {
                            Some(nested_items)
                        },
                    )
                }
                Item::ModuleValue(v) => {
                    (v.ident.name.clone(), "module", Some(v.definitions.clone()))
                }
                Item::AnnotationValue(v) => (v.ident.name.clone(), "annotation", None),
                Item::ValuetypeValue(v) => {
                    // Valuetypes can have nested definitions
                    (
                        v.ident.name.clone(),
                        "valuetype",
                        if v.definitions.is_empty() {
                            None
                        } else {
                            Some(v.definitions.clone())
                        },
                    )
                }
                _ => continue,
            };

            // Build the qualified name
            let qualified_name = if scope.is_empty() {
                name.clone()
            } else {
                format!("{}::{name}", scope.join("::"))
            };

            let key = ItemKey {
                name: name.clone(),
                kind,
            };

            // Look up with qualified name
            if let Some(&id) = self.name_map.get(&qualified_name) {
                self.item_map.insert(key, id);
            }

            // Process nested items if this is a module or interface
            if let Some(nested) = nested_items {
                let mut new_scope = scope.to_vec();
                new_scope.push(name.clone());
                self.build_item_map_with_scope(&nested, &new_scope);
            }
        }
    }

    /// Recursively collects and resolves all annotation definitions.
    fn resolve_annotations_recursively(&mut self, items: &[Item], scope: &[String]) {
        for item in items {
            match item {
                Item::AnnotationValue(v) => {
                    // Build qualified name for lookup
                    let qualified_name = if scope.is_empty() {
                        v.ident.name.clone()
                    } else {
                        format!("{}::{}", scope.join("::"), v.ident.name)
                    };

                    // Look up by qualified name instead of using item_map
                    if let Some(&id) = self.name_map.get(&qualified_name) {
                        self.resolve_annotation(id, v);
                    }
                }
                Item::ModuleValue(v) => {
                    // Recursively process annotations in modules with updated scope
                    let mut new_scope = scope.to_vec();
                    new_scope.push(v.ident.name.clone());
                    self.resolve_annotations_recursively(&v.definitions, &new_scope);
                }
                Item::InterfaceValue(v) => {
                    // Process annotations in interfaces
                    let nested_items: Vec<Item> = v
                        .members
                        .iter()
                        .filter_map(|m| {
                            if let ic_syntax::InterfaceMember::Item(item) = m {
                                Some(item.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    self.resolve_annotations_recursively(&nested_items, scope);
                }
                _ => {}
            }
        }
    }

    /// Resolves all type references in the HIR.
    #[allow(clippy::too_many_lines)]
    fn resolve_all(&mut self, items: &[Item]) {
        // First pass: build item map
        self.build_item_map(items);

        // Second pass: resolve ALL annotation definitions recursively
        // This ensures that annotation definitions are fully resolved before
        // they are used in other items
        self.resolve_annotations_recursively(items, &[]);

        // Third pass: resolve all other items
        for item in items {
            match item {
                Item::AnnotationValue(_) => {
                    // Already processed in second pass
                }
                Item::StructValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "struct",
                    }) {
                        self.resolve_struct(id, v);
                    }
                }
                Item::UnionValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "union",
                    }) {
                        self.resolve_union(id, v);
                    }
                }
                Item::ExceptionValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "exception",
                    }) {
                        self.resolve_exception(id, v);
                    }
                }
                Item::InterfaceValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "interface",
                    }) {
                        self.resolve_interface(id, v);
                    }
                }
                Item::AliasValue(v) => {
                    // Handle multiple declarators
                    for (idx, decl) in v.decl.iter().enumerate() {
                        let name = match decl {
                            ic_syntax::Declarator::Simple(n) => n.name.clone(),
                            ic_syntax::Declarator::Array(a) => a.ident.name.clone(),
                        };

                        if let Some(&id) = self.name_map.get(&name) {
                            self.resolve_alias(id, v, idx);
                        }
                    }
                }
                Item::ConstValue(v) => {
                    let name = match &v.decl {
                        ic_syntax::Declarator::Simple(n) => n.name.clone(),
                        ic_syntax::Declarator::Array(a) => a.ident.name.clone(),
                    };
                    if let Some(&id) = self.name_map.get(&name) {
                        self.resolve_const(id, v);
                    }
                }
                Item::ValuetypeValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "valuetype",
                    }) {
                        self.resolve_valuetype(id, v);
                    }
                }
                Item::EnumValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "enum",
                    }) {
                        self.resolve_enum(id, v);
                    }
                }
                Item::BitmaskValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "bitmask",
                    }) {
                        self.resolve_bitmask(id, v);
                    }
                }
                Item::BitsetValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "bitset",
                    }) {
                        self.resolve_bitset(id, v);
                    }
                }
                Item::ModuleValue(v) => {
                    if let Some(&id) = self.item_map.get(&ItemKey {
                        name: v.ident.name.clone(),
                        kind: "module",
                    }) {
                        self.resolve_module(id, v);
                    }
                }
                // TODO: Handle other item types
                Item::DeclValue(_) => {}
            }
        }
    }

    /// Resolves a constant definition.
    fn resolve_const(&mut self, id: DefId, ast: &ic_syntax::ConstDef) {
        // Resolve the base type
        let base_ty = self.resolve_type(&ast.ty);

        // Apply declarator to get the actual type
        let (_, ty) = Self::resolve_declarator(&ast.decl, base_ty);

        // Resolve annotations for the constant itself
        let annotations = self.resolve_ast_annotations(&ast.annotations);

        // Update the constant's type
        let def = self.ctx.definitions.get_mut(id);
        def.annotations = annotations;
        if let DefKind::Const(const_ty) = &mut def.kind {
            const_ty.ty = ty;
        }
    }

    /// Resolves a valuetype definition.
    fn resolve_valuetype(&mut self, id: DefId, def: &ic_syntax::ValuetypeDef) {
        // Mark any forward declarations as resolved
        self.mark_forward_declarations_resolved(&def.ident.name);

        // Resolve parent/extends types
        let parent_id = if let Some(parent_path) = &def.inherits {
            if let Some(parent_id) = self.resolve_path(parent_path) {
                // Check if parent type is complete
                let parent_def = self.ctx.definitions.get(parent_id);
                if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                    self.errors.push(
                        error_span(
                            format!(
                                "valuetype `{}` cannot inherit from incomplete type `{}`",
                                def.ident.name, parent_def.ident.name
                            ),
                            Label::new(def.ident.span).message("invalid inheritance"),
                        )
                        .label(
                            Label::new(parent_def.ident.span)
                                .message("type is not yet defined at this point"),
                        ),
                    );
                    None
                } else {
                    Some(parent_id)
                }
            } else {
                None
            }
        } else {
            None
        };

        let extends_id = if let Some(extends_path) = &def.supports {
            if let Some(extends_id) = self.resolve_path(extends_path) {
                // Check if extends type is complete
                let extends_def = self.ctx.definitions.get(extends_id);
                if extends_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                    self.errors.push(
                        error_span(
                            format!(
                                "valuetype `{}` cannot extend incomplete type `{}`",
                                def.ident.name, extends_def.ident.name
                            ),
                            Label::new(def.ident.span).message("invalid extends"),
                        )
                        .label(
                            Label::new(extends_def.ident.span)
                                .message("type is not yet defined at this point"),
                        ),
                    );
                    None
                } else {
                    Some(extends_id)
                }
            } else {
                None
            }
        } else {
            None
        };

        // Resolve prototypes
        let mut prototypes = Vec::new();
        for proto in &def.prototypes {
            let ty = self.resolve_type(&proto.ret);

            let mut params = Vec::new();
            for param in &proto.params {
                // Get the identifier from the declarator
                let ident = match &param.decl {
                    ic_syntax::Declarator::Simple(name) => name.clone(),
                    ic_syntax::Declarator::Array(arr) => arr.ident.clone(),
                };

                params.push(Parameter {
                    ident,
                    ty: self.resolve_type(&param.ty),
                    kind: param.kind.unwrap_or(ic_syntax::ParamKind::In),
                });
            }

            prototypes.push(ProtoTy {
                ident: proto.ident.clone(),
                ty,
                params,
            });
        }

        // Update the definition
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);

        if let DefKind::Valuetype(vt) = &mut hir_def.kind {
            vt.parent = parent_id;
            vt.extends = extends_id;
            vt.prototypes = prototypes;
            // Members are still TODO - they're more complex
            vt.members = Vec::new();
        }
    }

    /// Resolves an enum definition.
    fn resolve_enum(&mut self, id: DefId, def: &ic_syntax::EnumDef) {
        // TODO: Handle underlying type from annotations
        // For now, default to int32
        let underlying_ty = Ty {
            kind: TyKind::Primitive(PrimitiveTy::Int32),
            span: def.span,
        };

        // Resolve annotations for each enum field first
        let field_annotations: Vec<Vec<Ann>> = def
            .fields
            .iter()
            .map(|field| self.resolve_ast_annotations(&field.annotations))
            .collect();

        // Resolve annotations for the enum itself
        let annotations = self.resolve_ast_annotations(&def.annotations);

        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.annotations = annotations;

        if let DefKind::Enum(enum_ty) = &mut hir_def.kind {
            enum_ty.ty = underlying_ty;

            // Apply resolved annotations to each enum field
            for (i, annotations) in field_annotations.into_iter().enumerate() {
                if i < enum_ty.fields.len() {
                    enum_ty.fields[i].annotations = annotations;
                }
            }
        }
    }

    /// Resolves a bitmask definition.
    fn resolve_bitmask(&mut self, id: DefId, def: &ic_syntax::BitmaskDef) {
        // TODO: Handle underlying type from annotations
        // For now, default to uint32
        let underlying_ty = Ty {
            kind: TyKind::Primitive(PrimitiveTy::UInt32),
            span: def.span,
        };

        // Resolve annotations for each bit flag first
        let flag_annotations: Vec<Vec<Ann>> = def
            .bits
            .iter()
            .map(|bit| self.resolve_ast_annotations(&bit.annotations))
            .collect();

        let hir_def = self.ctx.definitions.get_mut(id);
        if let DefKind::Bitmask(bitmask_ty) = &mut hir_def.kind {
            bitmask_ty.ty = underlying_ty;

            // Create flags if they don't exist yet
            if bitmask_ty.flags.is_empty() {
                for bit in &def.bits {
                    bitmask_ty.flags.push(BitFlag {
                        ident: bit.ident.clone(),
                        value: 0, // Will be filled in evaluate phase
                        annotations: Vec::new(),
                    });
                }
            }

            // Apply resolved annotations to each bit flag
            for (i, annotations) in flag_annotations.into_iter().enumerate() {
                if i < bitmask_ty.flags.len() {
                    bitmask_ty.flags[i].annotations = annotations;
                }
            }
        }
    }

    /// Resolves a bitset definition.
    fn resolve_bitset(&mut self, id: DefId, def: &ic_syntax::BitsetDef) {
        // Resolve parent if present
        let parent_id = if let Some(parent_path) = &def.parent {
            if let Some(parent_id) = self.resolve_path(parent_path) {
                // Check if parent type is complete
                let parent_def = self.ctx.definitions.get(parent_id);
                if parent_def.flags.contains(DefFlags::IS_INCOMPLETE) {
                    self.errors.push(
                        error_span(
                            format!(
                                "bitset `{}` cannot inherit from incomplete type `{}`",
                                def.ident.name, parent_def.ident.name
                            ),
                            Label::new(def.ident.span).message("invalid inheritance"),
                        )
                        .label(
                            Label::new(parent_def.ident.span)
                                .message("type is not yet defined at this point"),
                        ),
                    );
                    None
                } else {
                    Some(parent_id)
                }
            } else {
                None
            }
        } else {
            None
        };

        // Resolve field types (sizes will be evaluated in the evaluation phase)
        // We need a placeholder type for now; the actual type will be determined
        // in the evaluation phase based on the size
        let mut fields = Vec::new();
        for field in &def.fields {
            let ty = if let Some(explicit_ty) = &field.ty {
                self.resolve_type(explicit_ty)
            } else {
                // Use a placeholder type - will be replaced in evaluation phase
                Ty {
                    kind: TyKind::Any,
                    span: field.span,
                }
            };
            fields.push(BitsetField {
                ident: field.ident.clone(),
                size: 0, // Will be filled in evaluation phase
                ty,
                annotations: self.resolve_ast_annotations(&field.annotations),
            });
        }

        let hir_def = self.ctx.definitions.get_mut(id);
        if let DefKind::Bitset(bitset_ty) = &mut hir_def.kind {
            bitset_ty.parent = parent_id;
            bitset_ty.fields = fields;
        }
    }

    /// Resolves a module definition and its nested items.
    fn resolve_module(&mut self, id: DefId, def: &ic_syntax::ModuleDef) {
        // Mark module as resolved
        let hir_def = self.ctx.definitions.get_mut(id);
        hir_def.flags.unset(DefFlags::IS_INCOMPLETE);

        // Push module name to current scope (for compatibility)
        self.current_scope.push(def.ident.name.clone());

        // Find the child scope for this module
        let current_scope_data = self.ctx.scopes.get_scope(self.current_scope_id);
        if let Some(&module_scope) = current_scope_data.children.get(&def.ident.name) {
            // Save current scope
            let saved_scope = self.current_scope_id;
            self.current_scope_id = module_scope;

            // Recursively resolve all nested items
            self.resolve_all(&def.definitions);

            // Restore scope
            self.current_scope_id = saved_scope;
        } else {
            // Fallback: just resolve without changing scope
            self.resolve_all(&def.definitions);
        }

        // Pop module name from current scope
        self.current_scope.pop();
    }

    /// Resolves an annotation definition.
    fn resolve_annotation(&mut self, id: DefId, ast: &ic_syntax::AnnotationDef) {
        // Save current scope
        let saved_scope_id = self.current_scope_id;

        // Enter annotation scope so nested types can be resolved
        if let Some(annotation_scope) = self.ctx.scopes.find_scope_for_def(id) {
            self.current_scope_id = annotation_scope;
        }

        // Resolve annotations on the annotation itself
        let annotations = self.resolve_ast_annotations(&ast.annotations);

        // Resolve nested type definitions first
        let nested_items: Vec<Item> = ast
            .params
            .iter()
            .filter_map(|field| {
                if let ic_syntax::AnnotationField::Item(item) = field {
                    Some((**item).clone())
                } else {
                    None
                }
            })
            .collect();
        if !nested_items.is_empty() {
            self.resolve_all(&nested_items);
        }

        let mut members = Vec::new();
        for field in &ast.params {
            if let ic_syntax::AnnotationField::Member(member) = field {
                // Resolve the member type
                let base_ty = self.resolve_type(&member.ty);
                let (_name, ty) = Self::resolve_declarator(&member.decl, base_ty);

                // Resolve member annotations
                let member_annotations = self.resolve_ast_annotations(&member.annotations);

                // Default values will be evaluated in the evaluate phase
                let default_value = None;

                let ident = match &member.decl {
                    ic_syntax::Declarator::Simple(id) => id.clone(),
                    ic_syntax::Declarator::Array(arr) => arr.ident.clone(),
                };

                members.push(Member {
                    ident: ident.clone(),
                    ty,
                    annotations: member_annotations,
                    default_value,
                });
            }
        }

        // Update the annotation definition
        let def = self.ctx.definitions.get_mut(id);
        def.annotations = annotations;
        def.flags.unset(DefFlags::IS_INCOMPLETE);

        if let DefKind::Annotation(ann) = &mut def.kind {
            ann.members = members;
        }

        // Restore scope
        self.current_scope_id = saved_scope_id;
    }

    /// Process annotation arguments, ensuring all are named and validated against the definition
    fn process_annotation_args(
        &mut self,
        args: &[ic_syntax::AnnotationArg],
        members: &[crate::hir::Member],
        ann_name: &str,
        ann_span: Span,
    ) -> Vec<crate::hir::AnnArg> {
        let mut result = Vec::new();
        let mut used_params = std::collections::HashSet::new();

        // Process named arguments first
        for arg in args.iter().filter(|a| a.ident.is_some()) {
            let param_name = arg.ident.as_ref().unwrap().name.clone();

            // Check if this parameter exists in the annotation definition
            if let Some(member) = members.iter().find(|m| m.ident.name == param_name) {
                if used_params.contains(&param_name) {
                    self.warnings.push(warn_span(
                        format!("duplicate parameter '{param_name}' in @{ann_name}"),
                        Label::new(arg.ident.as_ref().unwrap().span)
                            .message("parameter already specified"),
                    ));
                } else {
                    used_params.insert(param_name.clone());
                    result.push(crate::hir::AnnArg {
                        ident: member.ident.clone(),
                        value: super::convert_annotation_value(&arg.value),
                    });
                }
            } else {
                self.warnings.push(warn_span(
                    format!("unknown parameter '{param_name}' in @{ann_name}"),
                    Label::new(arg.ident.as_ref().unwrap().span)
                        .message("parameter not found in annotation definition"),
                ));
            }
        }

        // Process positional arguments
        let positional_args: Vec<_> = args.iter().filter(|a| a.ident.is_none()).collect();

        // Check if annotation has multiple parameters and positional args are used
        if members.len() > 1 && !positional_args.is_empty() {
            self.warnings.push(warn_span(
                format!(
                    "@{ann_name} has {} parameters and requires named arguments",
                    members.len()
                ),
                Label::new(ann_span)
                    .message("annotations with multiple parameters must use named arguments"),
            ));
            // Don't process the positional arguments
        } else if positional_args.len() == 1 {
            // Single positional argument - assign to first parameter without default
            if let Some(member) = members
                .iter()
                .find(|m| !used_params.contains(&m.ident.name) && m.default_value.is_none())
            {
                used_params.insert(member.ident.name.clone());
                result.push(crate::hir::AnnArg {
                    ident: member.ident.clone(),
                    value: super::convert_annotation_value(&positional_args[0].value),
                });
            } else {
                self.warnings.push(warn_span(
                    format!("no available parameter for positional argument in @{ann_name}"),
                    Label::new(ann_span)
                        .message("all parameters have defaults or are already specified"),
                ));
            }
        } else if positional_args.len() > 1 {
            self.warnings.push(warn_span(
                format!("multiple positional arguments in @{ann_name}"),
                Label::new(ann_span).message("only one positional argument is allowed"),
            ));
        }

        result
    }
}

/// Converts a path to its string representation.
fn path_to_string(path: &Path) -> String {
    let segments = path
        .segments
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("::");

    if path.leading_colons.is_some() {
        format!("::{segments}")
    } else {
        segments
    }
}

/// Resolves a primitive type name.
fn resolve_primitive(name: &str) -> Option<PrimitiveTy> {
    // IDL is case-insensitive
    match name.to_lowercase().as_str() {
        "void" => Some(PrimitiveTy::Void),
        "boolean" => Some(PrimitiveTy::Bool),
        "char" => Some(PrimitiveTy::Char),
        "wchar" => Some(PrimitiveTy::WChar),
        "int8" => Some(PrimitiveTy::Int8),
        "octet" | "uint8" => Some(PrimitiveTy::UInt8),
        "short" | "int16" => Some(PrimitiveTy::Int16),
        "unsigned short" | "uint16" => Some(PrimitiveTy::UInt16),
        "long" | "int32" => Some(PrimitiveTy::Int32),
        "unsigned long" | "uint32" => Some(PrimitiveTy::UInt32),
        "long long" | "int64" => Some(PrimitiveTy::Int64),
        "unsigned long long" | "uint64" => Some(PrimitiveTy::UInt64),
        "float" => Some(PrimitiveTy::Float32),
        "double" => Some(PrimitiveTy::Float64),
        "long double" => Some(PrimitiveTy::Float128),
        _ => None,
    }
}

/// Resolves all type references in the HIR.
pub fn resolve_types(
    ctx: &mut Context,
    name_map: &NameMap,
    items: &[Item],
    errors: &mut Vec<Diag>,
    warnings: &mut Vec<Diag>,
) {
    let mut resolver = TypeResolver::new(ctx, name_map, items);
    resolver.resolve_all(items);

    // Note: Annotations are now resolved directly in each resolve_* method
    // so we don't need a separate pass for annotations

    errors.append(&mut resolver.errors);
    warnings.append(&mut resolver.warnings);
}
