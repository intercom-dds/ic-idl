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

use crate::scope::ScopeId;

use super::type_items::TypeItemProcessor;
use super::value_items::ValueItemProcessor;
use super::LoweringContext;

/// Main HIR builder that orchestrates the lowering process.
pub struct HirBuilder<'ctx> {
    ctx: &'ctx mut LoweringContext,
    current_scope: ScopeId,
}

impl<'ctx> HirBuilder<'ctx> {
    pub fn new(ctx: &'ctx mut LoweringContext) -> Self {
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
    fn process_item(&mut self, item: &Item) {
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
            Item::AnnotationValue(_a) => {
                // TODO: Process annotation
            }
            Item::AliasValue(_t) => {
                // TODO: Process type alias
            }
            Item::ExceptionValue(_e) => {
                // TODO: Process exception
            }
            Item::BitsetValue(_b) => {
                // TODO: Process bitsets
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
        
        // Process module contents
        self.build(&m.definitions);
        
        // Restore previous scope
        self.current_scope = prev_scope;
    }
}