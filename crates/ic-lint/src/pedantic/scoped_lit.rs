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

use std::collections::HashMap;
use std::fmt::{Display, Write as _};
use std::iter::{Enumerate, Map};

use ic_cli::color::Colorize;
use ic_diagnostic::{Label, warn_span};
use ic_syntax::visit::{Visitor, walk_expr, walk_tree};
use ic_syntax::{BitmaskDef, Declarator, EnumDef, Expr, Item, LiteralValue, Path};

use crate::iter::IterExt;
use crate::{Category, Lint, LintCtx};

#[derive(Copy, Clone)]
enum Kind {
    Bitmask,
    Enum,
}

pub struct ScopedLit<'a> {
    ctx: &'a LintCtx<'a>,
    seen: HashMap<&'a str, Kind>,
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

impl<'a> Visitor<'a> for ScopedLit<'a> {
    // TODO: in the future we should use the HIR ctx to do lookups instead of
    // registering the type name here.
    fn visit_enum(&mut self, def: &'a EnumDef) {
        self.seen.insert(&def.ident.name, Kind::Enum);
    }

    fn visit_bitmask(&mut self, def: &'a BitmaskDef) {
        self.seen.insert(&def.ident.name, Kind::Bitmask);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Path(path) = expr {
            if let Some(v) = path.segments.iter().rev().nth(1) {
                if let Some(kind) = self.seen.get(v.name.as_str()) {
                    let (ty, member) = match kind {
                        Kind::Bitmask => ("bitmask", "bitmask flags"),
                        Kind::Enum => ("enum", "enumerators"),
                    };

                    let fixed = fixed_path(path).green();
                    let label = warn_span(
                        format!("scoped {ty}s are an InterCOM extension"),
                        Label::new(v.span).message("used here"),
                    )
                    .note(format!("{member} are registered in the parent scope"))
                    .help(format!("remove the type name: `{fixed}`"));

                    self.ctx.report_warn(label);
                }
            }
        } else {
            // Continue traversal -- this may be a binary expression of bitmask
            // flags, so we'll want to check those as well.
            walk_expr(self, expr);
        }
    }
}

impl<'a> Lint<'a> for ScopedLit<'_> {
    fn name() -> &'static str {
        "scoped_lit"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[Item]) {
        let mut lint = ScopedLit {
            ctx,
            seen: HashMap::default(),
        };
        walk_tree(&mut lint, tree);
    }
}
