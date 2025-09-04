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

//! Builder for struct and union members.

use ic_syntax::{Declarator, Field, Ident};

use crate::hir::{Ann, Member, Ty};

/// Interface for resolving types and annotations.
pub trait MemberResolver {
    /// Resolves an AST type to a HIR type.
    fn resolve_type(&mut self, ty: &ic_syntax::Type) -> Ty;

    /// Resolves AST annotations to HIR annotations.
    fn resolve_annotations(&mut self, annotations: &[ic_syntax::AnnotationAppl]) -> Vec<Ann>;
}

/// Builds members from AST fields.
pub struct MemberBuilder<'a, R: MemberResolver> {
    resolver: &'a mut R,
}

impl<'a, R: MemberResolver> MemberBuilder<'a, R> {
    /// Creates a new member builder.
    pub fn new(resolver: &'a mut R) -> Self {
        Self { resolver }
    }

    /// Builds members from a list of fields.
    pub fn build_members(&mut self, fields: &[Field]) -> Vec<Member> {
        let mut members = Vec::new();

        for field in fields {
            let field_annotations = self.resolver.resolve_annotations(&field.annotations);
            let base_ty = self.resolver.resolve_type(&field.ty);

            // Process each declarator
            for decl in &field.names {
                let (ident, ty) = self.resolve_declarator(decl, base_ty.clone());
                members.push(Member {
                    ident,
                    ty,
                    annotations: field_annotations.clone(),
                });
            }
        }

        members
    }

    /// Resolves a declarator to produce an identifier and type.
    fn resolve_declarator(&self, decl: &Declarator, base_ty: Ty) -> (Ident, Ty) {
        match decl {
            Declarator::Simple(ident) => (ident.clone(), base_ty),
            Declarator::Array(arr) => {
                // Build array type from innermost to outermost
                let mut ty = base_ty;
                for bound_expr in &arr.bounds {
                    ty = Ty {
                        span: ty.span,
                        kind: crate::hir::TyKind::Array {
                            ty: Box::new(ty.clone()),
                            len: 0, // Will be filled in evaluation phase
                            len_span: ic_syntax::util::expr_span(bound_expr),
                        },
                    };
                }
                (arr.ident.clone(), ty)
            }
        }
    }
}
