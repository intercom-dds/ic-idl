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

//! Main builder orchestration for Phase 1: Build & Resolve.

use ic_syntax::Item;

use super::LoweringContext;
use super::type_items::TypeItemProcessor;
use super::value_items::ValueItemProcessor;
use crate::hir::DefId;
use crate::scope::ScopeId;

/// Main HIR builder that orchestrates the lowering process.
pub struct HirBuilder<'ctx> {
    ctx: &'ctx mut LoweringContext,
    pub(super) current_scope: ScopeId,
}

impl<'ctx> HirBuilder<'ctx> {
    pub(super) fn new(ctx: &'ctx mut LoweringContext) -> Self {
        let root_scope = ctx.context.root_scope();
        Self {
            ctx,
            current_scope: root_scope,
        }
    }

    /// Build HIR from AST items.
    pub fn build(&mut self, items: &[Item]) -> Vec<DefId> {
        let mut definitions = Vec::new();
        for item in items {
            definitions.extend(self.process_item(item));
        }
        definitions
    }

    /// Process a single AST item.
    #[allow(clippy::too_many_lines)]
    pub(super) fn process_item(&mut self, item: &Item) -> Vec<DefId> {
        match item {
            Item::ModuleValue(m) => {
                let def_id = self.process_module(m);
                vec![def_id]
            }

            // Delegate type items to type_items.rs
            Item::StructValue(s) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_struct(s);
                // Only add to order if at root scope
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::InterfaceValue(i) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_interface(i);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::UnionValue(u) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_union(u);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::ValuetypeValue(v) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_valuetype(v);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::DeclValue(decl) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_forward_decl(decl);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }

            // Delegate value items to value_items.rs
            Item::ConstValue(c) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_const(c);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::EnumValue(e) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_enum(e);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::BitmaskValue(b) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_bitmask(b);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }

            // Other items
            Item::AnnotationValue(a) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_annotation(a);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::AliasValue(a) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_ids = processor.process_alias(a);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.extend(&def_ids);
                }
                def_ids
            }
            Item::ExceptionValue(e) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_exception(e);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
            Item::BitsetValue(b) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_bitset(b);
                if self.current_scope == self.ctx.context.root_scope() {
                    self.ctx.order.push(def_id);
                }
                vec![def_id]
            }
        }
    }

    /// Process a module definition.
    fn process_module(&mut self, m: &ic_syntax::ModuleDef) -> DefId {
        // Find or create the module scope (handles reopening)
        let module_scope = self.ctx.context.scopes.find_or_create_module(
            self.current_scope,
            &m.ident.name,
            m.ident.span,
            &mut self.ctx.diagnostics,
        );

        // Save current scope and switch to module scope
        let prev_scope = self.current_scope;

        // Convert annotations before the closure
        let annotations = {
            let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
            processor.convert_annotations(&m.annotations, self.current_scope)
        };

        // Create a placeholder module definition first
        let def_id = self
            .ctx
            .context
            .definitions
            .alloc_with_id(|id| crate::hir::Def {
                id,
                ident: m.ident.clone(),
                parent: self.ctx.context.scopes.get_scope(prev_scope).def_id,
                annotations,
                span: m.ident.span,
                kind: crate::hir::DefKind::Module(crate::hir::ModuleTy {
                    definitions: Vec::new(), // Will be updated later
                }),
                flags: crate::hir::DefFlags::nil(),
            });

        // Update the module scope's def_id BEFORE processing contents
        self.ctx.context.scopes.get_scope_mut(module_scope).def_id = Some(def_id);

        // NOW switch to module scope and process contents
        self.current_scope = module_scope;
        let module_block_definitions = self.build(&m.definitions);

        // Update the module definition with the collected definitions
        if let crate::hir::DefKind::Module(ref mut module_ty) =
            self.ctx.context.definitions.get_mut(def_id).kind
        {
            module_ty.definitions = module_block_definitions;
        }

        // Check if this is the first module block with this name in the parent scope
        let parent_scope = self.ctx.context.scopes.get_scope(prev_scope);
        let is_first_module = !parent_scope.definitions.contains_key(&m.ident.name);

        if is_first_module {
            // First module block with this name: register it in the parent scope's name map
            self.ctx
                .context
                .scopes
                .add_definition(prev_scope, m.ident.name.clone(), def_id);
        }

        // Only record as a top-level item if we're at the root scope
        if prev_scope == self.ctx.context.root_scope() {
            self.ctx.order.push(def_id);
        }

        // Restore previous scope
        self.current_scope = prev_scope;

        def_id
    }
}
