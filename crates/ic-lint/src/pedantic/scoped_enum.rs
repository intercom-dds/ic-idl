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

use std::collections::HashSet;
use std::fmt::{Display, Write as _};
use std::iter::{Enumerate, Map};

use ic_cli::color::Colorize;
use ic_diagnostic::{warn_span, Label};
use ic_syntax::visit::{visit_tree, Visitor};
use ic_syntax::{Declarator, EnumDef, Expr, Item, LiteralValue, Path};

use crate::iter::IterExt;
use crate::{Category, Lint, LintCtx};

pub struct ScopedEnum<'a> {
    ctx: &'a LintCtx<'a>,
    seen: HashSet<&'a str>,
}

fn fixed_path(path: &Path) -> String {
    let name = path
        .segments
        .iter()
        .skip_nth(path.segments.len() - 2)
        .map(|p| &p.name)
        .join("::");

    if path.leading_colons.is_some() {
        format!("::{name}")
    } else {
        name
    }
}

impl<'a> Visitor<'a> for ScopedEnum<'a> {
    // TODO: in the future we should use the HIR ctx to do lookups instead of
    // registering the type name here.
    fn visit_enum(&mut self, def: &'a EnumDef) {
        self.seen.insert(&def.ident.name);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Path(path) = expr {
            if let Some(v) = path.segments.iter().rev().nth(1) {
                if self.seen.contains(v.name.as_str()) {
                    let fixed = fixed_path(path).green();
                    let label = warn_span(
                        "scoped enums are an InterCOM extension",
                        Label::new(v.span).message("used here"),
                    )
                    .note("enumerators are registered in the parent scope")
                    .help(format!("remove the type name: `{fixed}`"));

                    self.ctx.report(label);
                }
            }
        }
    }
}

impl<'a> Lint<'a> for ScopedEnum<'_> {
    fn category() -> Category {
        Category::Pedantic
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[Item]) {
        let mut lint = ScopedEnum {
            ctx,
            seen: HashSet::default(),
        };
        visit_tree(&mut lint, tree);
    }
}
