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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_cli::color::Colorize;
use ic_diagnostic::{Color, Diag, Label, error_span};
use ic_hir::keywords::IDL_KEYWORDS;
use ic_syntax::visit::{Visitor, walk_tree};
use ic_syntax::{Item, ParamKind, Prototype, util};

use crate::{Category, Lint, LintCtx};

/// Verifies that keywords are not used as identifiers. We don't treat all
/// keywords from the spec as keywords, so we instead rely on checking it here.
pub struct KwIdent<'a> {
    ctx: &'a LintCtx<'a>,
}

impl<'a> Visitor<'a> for KwIdent<'a> {
    // skip annotations as they may have keywords as names
    fn visit_annotation_appl(&mut self, _: &'a ic_syntax::AnnotationAppl) {}

    fn visit_annotation_def(&mut self, _: &'a ic_syntax::AnnotationDef) {}

    // don't visit types
    fn visit_type(&mut self, _: &'a ic_syntax::Type) {}

    fn visit_ident(&mut self, ident: &'a ic_syntax::Ident) {
        if IDL_KEYWORDS.contains(&ident.name.as_str())
            && (ident.span.end.offset - ident.span.start.offset) as usize == ident.name.len()
        {
            let fixed = format!("_{}", ident.name);
            let diag = error_span(
                format!(
                    "`{}` is a keyword and may not be used as an identifier",
                    ident.name,
                ),
                Label::new(ident.span).message("this is an IDL keyword"),
            )
            .help(format!(
                "the keyword can be escaped by changing to `{}`",
                fixed.green(),
            ));

            self.ctx.report_error(diag);
        }
    }
}

impl<'a> Lint<'a> for KwIdent<'a> {
    fn name() -> &'static str {
        "keywords"
    }

    fn category() -> Category {
        Category::Syntax
    }

    fn check(ctx: &'a LintCtx<'_>, ast: &[Item]) {
        let mut lint = Self { ctx };
        walk_tree(&mut lint, ast);
    }
}
