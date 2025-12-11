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

use ic_diagnostic::{Label, warn_span};
use ic_hir::hir::{Ann, DefKind};
use ic_hir::visit::{self, Visitor};

use crate::{Category, Lint, LintCtx};

/// Lint that warns about unknown annotations.
pub struct UnknownAnnotation<'a> {
    ctx: &'a LintCtx<'a>,
    hir: &'a ic_hir::ResolvedGraph,
    annotation_names: Vec<&'a str>,
}

impl<'a> Lint<'a> for UnknownAnnotation<'a> {
    fn name() -> &'static str {
        "unknown-annotation"
    }

    fn description() -> &'static str {
        "Annotations that could not be resolved"
    }

    fn category() -> Category {
        Category::Annotation
    }

    fn check_hir(ctx: &'a LintCtx<'a>, hir: &'a ic_hir::ResolvedGraph) {
        let annotation_names: Vec<&str> = hir
            .order
            .iter()
            .chain(&hir.builtin_order)
            .filter_map(|&id| {
                let def = hir.context.type_of(id);
                if matches!(def.kind, DefKind::Annotation(_)) {
                    Some(def.ident.name.as_str())
                } else {
                    None
                }
            })
            .collect();

        let mut visitor = Self {
            ctx,
            hir,
            annotation_names,
        };
        visit::walk_tree(&mut visitor, hir);
    }
}

impl<'a> Visitor<'a> for UnknownAnnotation<'a> {
    fn context(&self) -> &'a ic_hir::Context {
        &self.hir.context
    }

    fn visit_annotation(&mut self, ann: &'a Ann) {
        if ann.def_id.is_none() {
            let mut diag = warn_span(
                format!("unknown annotation `{}`", ann.ident.name),
                Label::new(ann.ident.span).message("annotation not found"),
            );

            if let Some(suggestion) = find_similar(&ann.ident.name, &self.annotation_names) {
                diag = diag.help(format!("did you mean `@{suggestion}`?"));
            }

            Self::report(self.ctx, diag);
        }
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();

    let len_a = a.len();
    let len_b = b.len();
    let mut column = vec![0; len_a + 1];

    for (i, item) in column.iter_mut().enumerate() {
        *item = i;
    }

    for x in 1..=len_b {
        column[0] = x;
        let mut last_diag = x - 1;

        for y in 1..=len_a {
            let old_diag = column[y];
            let cost = usize::from(a[y - 1] != b[x - 1]);
            column[y] = (column[y] + 1).min(column[y - 1] + 1).min(last_diag + cost);
            last_diag = old_diag;
        }
    }
    column[len_a]
}

fn find_similar<'a>(input: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for &candidate in candidates {
        let distance = levenshtein(input, candidate);
        if distance < best_distance {
            best_distance = distance;
            best_match = Some(candidate);
        }
    }

    let max_distance = (input.len() / 3).max(1);
    if best_distance <= max_distance {
        best_match
    } else {
        None
    }
}
