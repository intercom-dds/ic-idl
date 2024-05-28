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

use crate::syntax::*;

pub trait Visitor<'a> {
    fn visit_definition(&mut self, def: &'a Definition) {}

    fn visit_module(&mut self, def: &'a ModuleDef) {}

    fn visit_struct(&mut self, def: &'a StructDef) {}

    fn visit_struct_field(&mut self, def: &'a Field) {}

    fn visit_union(&mut self, def: &'a UnionDef) {}

    fn visit_enum(&mut self, def: &'a EnumDef) {}

    fn visit_enum_variant(&mut self, def: &'a Enumerator) {}
}

pub trait Visit {
    fn visit<'a, V: Visitor<'a>>(self, visitor: &mut V);
}

impl<T: Visit> Visit for Option<T> {
    fn visit<'a, V: Visitor<'a>>(self, visitor: &mut V) {
        if let Some(v) = self {
            v.visit(visitor);
        }
    }
}
