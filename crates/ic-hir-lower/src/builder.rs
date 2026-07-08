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

use ic_hir::hir::{DefId, DefKind};
use ic_hir::scope::ScopeId;
use ic_syntax::Item;
use ic_syntax::util::{item_ident_name, item_variant_name};
use tracing::{trace, trace_span};

use crate::type_items::TypeItemProcessor;
use crate::value_items::ValueItemProcessor;
use crate::{LoweringContext, resolve};

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

    fn add_to_order_if_root(&mut self, def_id: DefId) {
        if self.current_scope == self.ctx.context.root_scope() {
            self.ctx.order.push(def_id);
        }
    }

    fn add_all_to_order_if_root(&mut self, def_ids: &[DefId]) {
        if self.current_scope == self.ctx.context.root_scope() {
            self.ctx.order.extend(def_ids);
        }
    }

    /// Process a single AST item.
    pub(super) fn process_item(&mut self, item: &Item) -> Vec<DefId> {
        let kind = item_variant_name(item);
        let name = item_ident_name(item).unwrap_or("<anonymous>");
        let _span = trace_span!("lower", %kind, %name).entered();

        let def_ids = match item {
            Item::ModuleValue(m) => {
                let def_id = self.process_module(m);
                vec![def_id]
            }

            Item::StructValue(s) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_struct(s);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::InterfaceValue(i) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_interface(i);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::UnionValue(u) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_union(u);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::ValuetypeValue(v) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_valuetype(v);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::DeclValue(decl) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_forward_decl(decl);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }

            Item::ConstValue(c) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_const(c);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::EnumValue(e) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_enum(e);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::BitmaskValue(b) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_bitmask(b);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }

            Item::AnnotationValue(a) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_annotation(a);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::AliasValue(a) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_ids = processor.process_alias(a);
                self.add_all_to_order_if_root(&def_ids);
                def_ids
            }
            Item::ExceptionValue(e) => {
                let mut processor = TypeItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_exception(e);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
            Item::BitsetValue(b) => {
                let mut processor = ValueItemProcessor::new(self.ctx, self.current_scope);
                let def_id = processor.process_bitset(b);
                self.add_to_order_if_root(def_id);
                vec![def_id]
            }
        };

        trace!(def_ids = ?def_ids, "lowered");
        def_ids
    }

    fn process_module(&mut self, m: &ic_syntax::ModuleDef) -> DefId {
        let module_scope = resolve::find_or_create_module(
            &mut self.ctx.context.scopes,
            self.current_scope,
            &m.ident.name,
            m.ident.span,
            &mut self.ctx.module_scopes,
            &mut self.ctx.diagnostics,
        );

        let prev_scope = self.current_scope;
        let def_id = crate::define::define(
            self.ctx,
            prev_scope,
            &m.ident,
            m.span,
            &m.annotations,
            crate::registry::DefKindTag::Module,
            |_| {
                DefKind::Module(ic_hir::hir::ModuleTy {
                    definitions: Vec::new(), // Will be updated later
                })
            },
        );
        self.ctx
            .context
            .scopes
            .set_scope_def_id(module_scope, def_id);

        self.current_scope = module_scope;
        let module_block_definitions = self.build(&m.definitions);

        if let DefKind::Module(ref mut module_ty) =
            self.ctx.context.definitions.get_mut(def_id).kind
        {
            module_ty.definitions = module_block_definitions;
        }

        if prev_scope == self.ctx.context.root_scope() {
            self.ctx.order.push(def_id);
        }

        self.current_scope = prev_scope;
        def_id
    }
}

/// Processes a single AST item nested inside another definition's body
/// (an interface member, valuetype element, or annotation parameter that's
/// itself an item), using `scope` as the nested item's enclosing scope.
pub(super) fn process_nested_item(
    ctx: &mut LoweringContext,
    scope: ScopeId,
    item: &Item,
) -> Vec<DefId> {
    let mut builder = HirBuilder::new(ctx);
    builder.current_scope = scope;
    builder.process_item(item)
}
