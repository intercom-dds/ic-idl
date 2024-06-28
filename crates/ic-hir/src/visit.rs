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

use crate::hir::{EnumTy, ModuleTy, Numeric, StructTy, Type, UnionTy};

pub trait Visitor<'a> {
    fn visit_ty(&mut self, ty: &'a Type) {
        visit_ty(self, ty)
    }

    fn visit_module(&mut self, ty: &'a ModuleTy) {}

    fn visit_struct(&mut self, ty: &'a StructTy) {}

    fn visit_enum(&mut self, ty: &'a EnumTy) {}

    fn visit_union(&mut self, ty: &'a UnionTy) {}

    fn visit_numeric(&mut self, ty: &'a Numeric) {}
}

pub fn visit_ty<'a, V>(visitor: &mut V, ty: &'a Type)
where
    V: Visitor<'a> + ?Sized,
{
    match ty {
        Type::Primitive(_) => todo!(),
        Type::Annotation(_) => todo!(),
        Type::Module(v) => visitor.visit_module(v),
        Type::Alias(_) => todo!(),
        Type::Const(_) => todo!(),
        Type::Struct(v) => visitor.visit_struct(v),
        Type::Except(_) => todo!(),
        Type::Union(v) => visitor.visit_union(v),
        Type::Enum(v) => visitor.visit_enum(v),
        Type::Bitmask(_) => todo!(),
        Type::Interface(_) => todo!(),
        Type::Decl(_) => todo!(),
        Type::Array { ty, len } => todo!(),
    }
}
