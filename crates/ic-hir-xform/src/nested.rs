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

//! Moves nested type definitions to their parent scope. This currently only
//! applies to types defined inside interfaces and valuetypes.

use ic_hir::fold::Fold;
use ic_hir::{hir, ResolvedGraph};

struct TyShift;

impl Fold for TyShift {
    fn fold_def(&mut self, def: hir::Def) -> hir::Def {
        match &def.kind {
            hir::DefKind::Interface(v) => {
                def
            }
            _ => def,
        }
    }

    fn fold_decl(&mut self, decl: hir::Decl) -> hir::Decl {
        decl
    }

    fn fold_ty(&mut self, ty: hir::Ty) -> hir::Ty {
        ty
    }

    fn fold_numeric(&mut self, num: hir::Numeric) -> hir::Numeric {
        num
    }
}

pub fn transform(hir: ResolvedGraph) -> ResolvedGraph {
    hir
}
