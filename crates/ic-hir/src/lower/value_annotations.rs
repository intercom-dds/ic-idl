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

use ic_syntax::util::path_name;
use ic_syntax::{AnnotationAppl, AnnotationArg};

use crate::hir::{Ann, AnnArg, DefKind, Ident};
use crate::lower::eval::ConstEvaluator;
use crate::lower::value_items::ValueItemProcessor;
use crate::scope::ScopeId;

impl ValueItemProcessor<'_> {
    /// Convert AST annotations to HIR annotations.
    pub fn convert_annotations(
        &mut self,
        ast_annotations: &[AnnotationAppl],
        scope: ScopeId,
    ) -> Vec<Ann> {
        ast_annotations
            .iter()
            .map(|ann_appl| self.convert_annotation(ann_appl, scope))
            .collect()
    }

    /// Convert a single AST annotation to HIR annotation.
    fn convert_annotation(&mut self, ann_appl: &AnnotationAppl, scope: ScopeId) -> Ann {
        let start = if ann_appl.ident.leading_colons.is_some() {
            self.ctx.context.root_scope()
        } else {
            scope
        };

        // Try to resolve the annotation path
        let def_id = self.ctx.context.resolve_syntax_path(start, &ann_appl.ident);
        let (def_id, name) = if let Some(id) = def_id {
            let def = self.ctx.context.definitions.get(id);
            if matches!(def.kind, DefKind::Annotation(_)) {
                (Some(id), path_name(&ann_appl.ident))
            } else {
                self.ctx.diagnostics.error(
                    format!("`{}` is not an annotation", path_name(&ann_appl.ident)),
                    ic_diagnostic::Label::new(ann_appl.span)
                        .message("expected an annotation definition"),
                );
                // Still include it in HIR but without a valid def_id
                (None, path_name(&ann_appl.ident))
            }
        } else {
            (None, path_name(&ann_appl.ident))
        };

        // Convert annotation arguments
        let args = self.convert_annotation_args(&ann_appl.args, scope);
        Ann {
            ident: Ident {
                name,
                span: ann_appl.span,
            },
            def_id,
            args,
        }
    }

    /// Convert annotation arguments.
    fn convert_annotation_args(
        &mut self,
        ast_args: &[AnnotationArg],
        scope: ScopeId,
    ) -> Vec<AnnArg> {
        let mut args = Vec::new();

        for arg in ast_args {
            // Evaluate the argument value expression
            let mut evaluator = ConstEvaluator::new(self.ctx, scope);
            let value = evaluator.eval_annotation_arg(&arg.value);

            if let Some(val) = value {
                let ident = if let Some(ref name) = arg.ident {
                    // Named argument
                    name.clone()
                } else {
                    // Positional argument - use empty name
                    Ident {
                        name: String::new(),
                        span: arg.span,
                    }
                };

                args.push(AnnArg { ident, value: val });
            }
        }

        args
    }
}
