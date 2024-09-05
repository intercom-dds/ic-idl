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

use ic_emit::case::Case;
use ic_hir::fold::Fold;
use ic_hir::hir::{ConstTy, Item};

/// Defines the naming convention to use for types of a specific kind.
///
/// If there are specific language items that should not be renamed, setting
/// the corresponding field to `None` will prevent the transformation from
/// renaming them.
#[derive(Copy, Clone, Debug)]
pub struct Target {
    /// Algebraic data types like structs, unions and enums.
    pub adt: Option<Case>,

    /// Value of a C-like enumerator.
    pub enumerator: Option<Case>,
    pub constant: Option<Case>,
    pub module: Option<Case>,
    pub prototype: Option<Case>,
}

/// Logic for renaming items in the HIR to conform to a specific naming
/// convention. This can be used to e.g. make all types in a data model follow
/// the PEP-8 style guide for Python.
pub fn rename_all<I>(items: I, target: Target)
where
    I: IntoIterator<Item = Item>,
{
    let mut renamer = Renamer { target };
    for item in items.into_iter() {
        renamer.fold_item(item);
    }
}

struct Renamer {
    target: Target,
}

impl Fold for Renamer {
    fn fold_item(&mut self, item: Item) -> Item {
        if let Item::Const(def) = item {
            Item::Const(self.fold_const(def))
        } else {
            item
        }
    }

    fn fold_const(&mut self, mut ty: ConstTy) -> ConstTy {
        if let Some(case) = self.target.constant {
            ty.ident.name = ic_emit::case::convert(&ty.ident.name, case);
        }
        ty
    }
}
