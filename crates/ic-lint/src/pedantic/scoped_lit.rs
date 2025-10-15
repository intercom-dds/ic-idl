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

use ic_cli::color::Colorize as _;
use ic_diagnostic::{Label, warn_span};
use ic_syntax::visit::{Visitor, walk_expr, walk_tree};
use ic_syntax::{BitmaskDef, EnumDef, Expr, Item};

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

impl<'a> Visitor<'a> for ScopedLit<'a> {
    // TODO: in the future we should use the HIR ctx to do lookups instead of
    // registering the type name here.
    fn visit_enum(&mut self, def: &'a EnumDef) {
        self.seen.insert(def.ident.name.as_str(), Kind::Enum);
    }

    fn visit_bitmask(&mut self, def: &'a BitmaskDef) {
        self.seen.insert(def.ident.name.as_str(), Kind::Bitmask);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Path(path) = expr {
            if let Some(v) = path.segments.iter().rev().nth(1) {
                if let Some(kind) = self.seen.get(v.name.as_str()) {
                    let (ty, member) = match kind {
                        Kind::Bitmask => ("bitmask", "bitmask flags"),
                        Kind::Enum => ("enum", "enumerators"),
                    };

                    // Get just the enumerator name (last segment)
                    let enumerator = path.segments.last().map_or("", |s| s.name.as_str()).green();
                    // Get enum::enumerator (last two segments)
                    let enum_and_enumerator = if path.segments.len() >= 2 {
                        let last_two: Vec<&str> = path
                            .segments
                            .iter()
                            .rev()
                            .take(2)
                            .rev()
                            .map(|s| s.name.as_str())
                            .collect();
                        last_two.join("::").green()
                    } else {
                        path.segments.iter().map(|s| &s.name).join("::").green()
                    };
                    let label = warn_span(
                        format!("scoped {ty}s are non-standard"),
                        Label::new(v.span).message("used here"),
                    )
                    .note(format!("{member} are registered in the parent scope"))
                    .help(format!(
                        "use `{enumerator}` instead of `{enum_and_enumerator}`"
                    ));

                    Self::report(self.ctx, label);
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
        "scoped-lit"
    }

    fn category() -> Category {
        Category::Pedantic
    }

    fn description() -> &'static str {
        "Enum/bitmask values using scoped notation"
    }

    fn check(ctx: &'a LintCtx<'_>, tree: &[Item]) {
        let mut lint = ScopedLit {
            ctx,
            seen: HashMap::default(),
        };
        walk_tree(&mut lint, tree);
    }
}
