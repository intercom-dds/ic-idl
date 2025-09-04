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
        let root_scope = ctx.scopes.root();
        Self {
            ctx,
            current_scope: root_scope,
        }
    }

    /// Build HIR from AST items.
    pub fn build(&mut self, items: &[Item]) {
        for item in items {
            self.process_item(item);
        }
    }

    /// Process a single AST item.
    pub(super) fn process_item(&mut self, item: &Item) {
        match item {
            Item::ModuleValue(m) => self.process_module(m),

            // Delegate type items to type_items.rs
            Item::StructValue(s) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_struct(s);
            }
            Item::InterfaceValue(i) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_interface(i);
            }
            Item::UnionValue(u) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_union(u);
            }
            Item::ValuetypeValue(v) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_valuetype(v);
            }
            Item::DeclValue(decl) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_forward_decl(decl);
            }

            // Delegate value items to value_items.rs
            Item::ConstValue(c) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                processor.process_const(c);
            }
            Item::EnumValue(e) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                processor.process_enum(e);
            }
            Item::BitmaskValue(b) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                processor.process_bitmask(b);
            }

            // Other items
            Item::AnnotationValue(a) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                processor.process_annotation(a);
            }
            Item::AliasValue(a) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_alias(a);
            }
            Item::ExceptionValue(e) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                processor.process_exception(e);
            }
            Item::BitsetValue(b) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                processor.process_bitset(b);
            }
        }
    }

    /// Process a module definition.
    fn process_module(&mut self, m: &ic_syntax::ModuleDef) {
        // Find or create the module scope (handles reopening)
        let module_scope = self.ctx.scopes.find_or_create_module(
            self.current_scope,
            &m.ident.name,
            &mut self.ctx.context,
            &mut self.ctx.diagnostics,
        );

        // Save current scope and switch to module scope
        let prev_scope = self.current_scope;
        self.current_scope = module_scope;

        // Record definitions before processing contents
        let definitions_before = self
            .ctx
            .context
            .scopes
            .get_scope(module_scope)
            .definitions
            .values()
            .cloned()
            .collect::<Vec<_>>();

        // Process module contents
        self.build(&m.definitions);

        // Collect new definitions added by this module block
        let all_definitions = self
            .ctx
            .context
            .scopes
            .get_scope(module_scope)
            .definitions
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let new_definitions: Vec<DefId> = all_definitions
            .into_iter()
            .filter(|id| !definitions_before.contains(id))
            .collect();

        // Create a module definition in the HIR for this module block
        let module_ty = crate::hir::ModuleTy {
            definitions: new_definitions,
        };

        let def_id = self
            .ctx
            .context
            .definitions
            .alloc_with_id(|id| crate::hir::Def {
                id,
                ident: m.ident.clone(),
                parent: self.ctx.context.scopes.get_scope(self.current_scope).def_id,
                annotations: Vec::new(), // TODO: Convert annotations
                span: m.ident.span,
                kind: crate::hir::DefKind::Module(module_ty),
                flags: crate::hir::DefFlags::nil(),
            });

        // Don't register in scope - module names are already handled by the scope mechanism
        // Just record as a top-level item
        self.ctx.order.push(def_id);

        // Restore previous scope
        self.current_scope = prev_scope;
    }
}
