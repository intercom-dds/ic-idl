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

use ic_emit::case::{self, Case};
use ic_hir::fold::Fold;
use ic_hir::{ResolvedGraph, hir};

/// Defines the naming convention to use for types of a specific kind.
///
/// If there are specific language items that should not be renamed, setting
/// the corresponding field to `None` will prevent the transformation from
/// renaming them.
#[derive(Copy, Clone, Default, Debug)]
pub struct Target {
    /// Algebraic data types like structs, unions and enums.
    pub adt: Option<Case>,

    /// Member of a struct or union.
    pub member: Option<Case>,

    /// Value of a C-like enumerator.
    pub enumerator: Option<Case>,

    pub constant: Option<Case>,
    pub module: Option<Case>,
    pub prototype: Option<Case>,
}

#[derive(Default)]
struct Renamer {
    target: Target,
}

impl Fold for Renamer {
    fn fold_def(&mut self, mut def: hir::Def) -> hir::Def {
        let case = match def.kind {
            hir::DefKind::Module(_) => self.target.module,
            hir::DefKind::Const(_) => self.target.constant,
            _ => self.target.adt,
        };

        if let Some(case) = case {
            def.ident.name = case::convert(&def.ident.name, case);
        }

        match &mut def.kind {
            hir::DefKind::Annotation(_) => todo!(),
            hir::DefKind::Module(_) => todo!(),
            hir::DefKind::Struct(data) => {
                for mem in &mut data.members {
                    // TODO: impl a trait for all data types in the HIR
                    // instead, and then use that? probably easier.
                    mem.ident.name = case::convert(&mem.ident.name, Case::Snake);
                }
            }
            hir::DefKind::Except(_) => todo!(),
            hir::DefKind::Union(_) => todo!(),
            hir::DefKind::Enum(_) => todo!(),
            hir::DefKind::Const(_) => todo!(),
            hir::DefKind::Bitmask(_) => todo!(),
            hir::DefKind::Alias(_) => todo!(),
            hir::DefKind::Interface(_) => todo!(),
            hir::DefKind::Valuetype(_) => todo!(),
            hir::DefKind::Decl(_) => todo!(),
        }
        def
    }
}

#[must_use]
pub fn transform(mut hir: ResolvedGraph) -> ResolvedGraph {
    let mut renamer = Renamer {
        target: Target {
            adt: Some(Case::Pascal),
            member: Some(Case::Snake),
            enumerator: Some(Case::Snake),
            constant: Some(Case::Snake),
            module: Some(Case::Snake),
            prototype: Some(Case::Snake),
        },
    };

    for id in &hir.order {
        hir.context
            .definitions
            .fold(id, |def| renamer.fold_def(def));
    }
    hir
}

// Logic for renaming items in the HIR to conform to a specific naming
// convention. This can be used to e.g. make all types in a data model follow
// the PEP-8 style guide for Python.
// pub fn rename_all<I>(items: I, target: Target)
// where
//     I: IntoIterator<Item = Item>,
// {
//     let mut renamer = Renamer { target };
//     for item in items {
//         renamer.fold_item(item);
//     }
// }
