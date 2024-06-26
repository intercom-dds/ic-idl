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

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::num::NonZero;
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_macros::EnumIter;
use ic_syntax::util::{path_name, type_name};
use ic_syntax::visit::{visit_item, Visitor};
use ic_syntax::{Ident, Item, Span};

use crate::hir::{
    self, AliasTy, BitmaskTy, DeclTy, EnumTy, Enumerator, Member, ModuleTy, Numeric, PrimitiveTy,
    StructTy, TyFlags, Type, UnionTy,
};
use crate::{Context, TypeId};

pub struct Scope {
    symbols: HashMap<String, TypeId>,
}

pub struct Resolver {}

/// Responsible for lowering the AST to a HIR. This process will, amongst other
/// things, perform type checking, evaluate expressions, assign values to
/// things like enumerators, and ultimately construct the type-resolved graph
/// that is the HIR.
///
/// The HIR will alter the representation of the source code in some minor
/// ways, such as expanding a typedef with multiple declarators to multiple
/// typedefs with a single declarator each. More opinionated transformations of
/// the source code happens as subsequent passes on the `HIR`.
///
/// Note: avoid triggering errors here unless absolutely necessary.
/// Non-critical warnings and errors are better suited as lints.
struct Lower<'a> {
    ctx: &'a mut Context,
    order: Vec<TypeId>,
}

impl<'a> Lower<'a> {
    fn with_ctx(ctx: &'a mut Context) -> Self {
        Self { ctx, order: vec![] }
    }

    pub(crate) fn check_name_consistency(&self, lhs: &Ident, rhs: &Ident) -> bool {
        lhs.name.eq_ignore_ascii_case(&rhs.name)
    }

    /// Determines if two annotation definitions are consistent. The standard
    /// doesn't clarify what "consistent" means, but I've interpreted it as the
    /// two definitions being identical.
    pub(crate) fn check_ann_consistency(
        &mut self,
        lhs: &ic_syntax::AnnotationDef,
        rhs: &ic_syntax::AnnotationDef,
    ) -> bool {
        use ic_syntax::AnnotationField;

        if !lhs.name.name.eq_ignore_ascii_case(&rhs.name.name)
            || lhs.params.len() != rhs.params.len()
        {
            return false;
        }

        lhs.params.iter().zip(rhs.params.iter()).all(|v| match v {
            (AnnotationField::Arg(lhs), AnnotationField::Arg(rhs)) => {
                self.check_decl_consistency(&lhs.names, &rhs.names)
                    && self.check_type_consistency(&lhs.ty, &rhs.ty)
            }
            (AnnotationField::Const(lhs), AnnotationField::Const(rhs)) => {
                lhs.name.name.eq_ignore_ascii_case(&rhs.name.name)
                    && self.check_type_consistency(&lhs.ty, &rhs.ty)
                    && self.check_expr_consistency(&lhs.value, &rhs.value)
            }
            (lhs, rhs) => lhs.disc() == rhs.disc(),
        })
    }

    /// Determines if two sets of declarators are semantically consistent. They
    /// must resolve to the same types with the same bounds for them to be
    /// considered consistent.
    pub(crate) fn check_decl_consistency(
        &mut self,
        lhs: &[ic_syntax::Declarator],
        rhs: &[ic_syntax::Declarator],
    ) -> bool {
        use ic_syntax::Declarator;

        lhs.iter().zip(rhs.iter()).all(|v| match v {
            (Declarator::Simple(lhs), Declarator::Simple(rhs)) => {
                lhs.name.eq_ignore_ascii_case(&rhs.name)
            }
            (Declarator::Array(lhs), Declarator::Array(rhs)) => {
                lhs.bounds.len() == rhs.bounds.len()
                    && lhs.ident.name.eq_ignore_ascii_case(&rhs.ident.name)
                    && lhs
                        .bounds
                        .iter()
                        .zip(rhs.bounds.iter())
                        .all(|(lhs, rhs)| self.check_expr_consistency(lhs, rhs))
            }
            _ => false,
        })
    }

    /// Determines if two bounds are consistent. Two consistent bounds may
    /// contain different expression, but they must yield the same value.
    pub(crate) fn check_bound_consistency(
        &mut self,
        lhs: Option<&ic_syntax::Expr>,
        rhs: Option<&ic_syntax::Expr>,
    ) -> bool {
        if let Some((lhs, rhs)) = rhs.zip(lhs) {
            self.check_expr_consistency(lhs, rhs)
        } else {
            lhs.is_none() && rhs.is_none()
        }
    }

    /// Determines if two expressions are consistent. They must yield the same
    /// value -- i.e. the exact same bit pattern -- to be considered
    /// consistent.
    pub(crate) fn check_expr_consistency(
        &mut self,
        lhs: &ic_syntax::Expr,
        rhs: &ic_syntax::Expr,
    ) -> bool {
        let lhs = self.eval_expr(lhs);
        let rhs = self.eval_expr(rhs);
        bitwise_eq(&lhs, &rhs)
    }

    /// Determines if two types are semantically consistent. Collection types
    /// are treated as consistent if they have the same bound and resolve to
    /// the same element type.
    pub(crate) fn check_type_consistency(
        &mut self,
        lhs: &ic_syntax::Type,
        rhs: &ic_syntax::Type,
    ) -> bool {
        use ic_syntax::{FixedType, Type};

        match (lhs, rhs) {
            (Type::Any(_), Type::Any(_)) => true,
            (Type::Sequence(lhs), Type::Sequence(rhs)) => {
                self.check_bound_consistency(lhs.bound.as_ref(), rhs.bound.as_ref())
                    && self.check_type_consistency(lhs.ty.as_ref(), rhs.ty.as_ref())
            }
            (Type::String_(lhs), Type::String_(rhs)) => {
                self.check_bound_consistency(lhs.bound.as_ref(), rhs.bound.as_ref())
                    && lhs.wide == rhs.wide
            }
            (Type::Map(lhs), Type::Map(rhs)) => {
                self.check_bound_consistency(lhs.bound.as_ref(), rhs.bound.as_ref())
                    && self.check_type_consistency(lhs.key.as_ref(), rhs.key.as_ref())
                    && self.check_type_consistency(lhs.value.as_ref(), rhs.value.as_ref())
            }
            (Type::Path(lhs), Type::Path(rhs)) => {
                self.ctx.resolve_path(lhs) == self.ctx.resolve_path(rhs)
            }
            (
                Type::Fixed(FixedType {
                    bounds: Some(lhs), ..
                }),
                Type::Fixed(FixedType {
                    bounds: Some(rhs), ..
                }),
            ) => {
                self.check_expr_consistency(&lhs.total, &rhs.total)
                    && self.check_expr_consistency(&lhs.fractional, &rhs.fractional)
            }
            _ => false,
        }
    }

    pub(crate) fn eval_expr(&mut self, expr: &ic_syntax::Expr) -> Numeric {
        todo!()
    }

    // Forward declarations are somewhat tricky. Types that depend on the
    // forward-declared type should point to the type definition and not the
    // declaration, but at this point we're not guaranteed to have the seen the
    // definition.
    //
    // Instead, we allocate two type entries: one for the declaration, and one
    // for the yet-to-be-seen definition. The declaration will point to the
    // definition, and future calls to `Context::resolve_type` will yield the
    // ID of the definition. When we encounter the definition, we will mutate
    // the existing entry. Once construction of the HIR is done, we'll check
    // that all types have been defined.
    fn lower_decl(&mut self, symbol: &ic_syntax::Decl) -> TypeId {
        self.ctx.arena.alloc_with_id(|id| {
            Type::Decl(DeclTy {
                ident: symbol.name.clone(),
                ty: id,
            })
        })
    }

    fn lower_const(&mut self, symbol: &ic_syntax::ConstDef) -> TypeId {
        let ty = self.ctx.resolve_type(&symbol.ty);

        self.ctx.arena.alloc_with_id(|id| {
            Type::Const(hir::ConstTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                ty,
                value: Numeric::Octet(0),
            })
        })
    }

    // A typedef with multiple declarators will be expanded to multiple,
    // individual typedefs, each with one declarator.
    fn lower_alias(&mut self, symbol: &ic_syntax::Typedef) -> TypeId {
        let ty = self.ctx.resolve_type(&symbol.ty);

        self.ctx.arena.alloc_with_id(|id| {
            Type::Alias(hir::AliasTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                ty,
            })
        })
    }

    fn lower_mod(&mut self, symbol: &ic_syntax::ModuleDef) -> TypeId {
        let definitions = symbol
            .definitions
            .iter()
            .filter_map(|v| self.lower_item(v))
            .collect();

        self.ctx.arena.alloc_with_id(|id| {
            Type::Module(ModuleTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                definitions,
            })
        })
    }

    // Members with multiple declarators are expanded into multiple members,
    // each with a single declarator.
    fn lower_struct(&mut self, symbol: &ic_syntax::StructDef) -> TypeId {
        let mut members = vec![];
        for mem in &symbol.members {
            assert!(
                !mem.names.is_empty(),
                "struct member without any declarators",
            );

            let ty = self.ctx.resolve_type(&mem.ty);
            for decl in &mem.names {
                let (ident, ty) = match decl {
                    ic_syntax::Declarator::Simple(v) => (v.clone(), ty),
                    ic_syntax::Declarator::Array(v) => todo!(),
                };

                // TODO: should collection types be their own type? or should it be
                // a separate ValueType enum for map/sequence/array?
                members.push(Member { ident, ty });
            }
        }

        let id = self.ctx.arena.alloc_with_id(|id| {
            Type::Struct(StructTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                members,
                flags: TyFlags::nil(),
            })
        });
        self.ctx.symbols.insert(symbol.name.name.clone(), id);
        id
    }

    fn lower_except(&mut self, _: &ic_syntax::ExceptDef) -> TypeId {
        todo!()
    }

    fn lower_union(&mut self, symbol: &ic_syntax::UnionDef) -> TypeId {
        let disc = hir::Discriminator {
            ty: self.ctx.resolve_type(&symbol.disc.ty),
            span: ic_syntax::util::ty_span(&symbol.disc.ty),
        };

        let mut variants = vec![];
        for var in &symbol.fields {
            let labels: Vec<_> = var.labels.iter().map(|_| Numeric::Octet(0)).collect();

            let variant = match &var.field {
                ic_syntax::UnionElement::Member(v) => {
                    let ty = self.ctx.resolve_type(v.ty.as_ref());
                    hir::Variant::Member(
                        Member {
                            ident: match v.decl.clone() {
                                ic_syntax::Declarator::Simple(s) => s,
                                ic_syntax::Declarator::Array(v) => v.ident,
                            },
                            ty,
                        },
                        labels,
                    )
                }
                ic_syntax::UnionElement::Null(_) => hir::Variant::Null(labels),
            };
            variants.push(variant);
        }

        self.ctx.arena.alloc_with_id(|id| {
            Type::Union(UnionTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                disc,
                variants,
            })
        })
    }

    fn lower_bitmask(&mut self, symbol: &ic_syntax::BitmaskDef) -> TypeId {
        let bits = symbol.bits.iter().cloned().map(|v| (v.name, 0)).collect();
        self.ctx.arena.alloc_with_id(|id| {
            Type::Bitmask(BitmaskTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                bits,
            })
        })
    }

    fn lower_enum(&mut self, symbol: &ic_syntax::EnumDef) -> TypeId {
        let mut last_value = 0;
        let mut enumerators = vec![];

        for lit in &symbol.fields {
            let value = lit.value.as_ref().map_or_else(|| last_value + 1, |_| 0);
            last_value = value;

            enumerators.push(Enumerator {
                ident: lit.name.clone(),
                value,
            });
        }

        self.ctx.arena.alloc_with_id(|id| {
            Type::Enum(EnumTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                enumerators,
            })
        })
    }

    // TODO: implement as visitor instead?
    fn lower_item(&mut self, item: &Item) -> Option<TypeId> {
        let ty = match item {
            Item::AnnotationValue(_) => todo!(),
            Item::ModuleValue(v) => self.lower_mod(v),
            Item::StructValue(v) => self.lower_struct(v),
            Item::UnionValue(v) => self.lower_union(v),
            Item::EnumValue(v) => self.lower_enum(v),
            Item::ExceptionValue(v) => self.lower_except(v),
            Item::BitmaskValue(v) => self.lower_bitmask(v),
            Item::ConstValue(v) => self.lower_const(v),
            Item::TypedefValue(v) => self.lower_alias(v),
            Item::InterfaceValue(_) => todo!(),
            Item::ValuetypeValue(_) => todo!(),
            Item::DeclValue(v) => self.lower_decl(v),
            Item::BitsetValue(_) => return None,
        };
        self.order.push(ty);
        Some(ty)
    }
}

struct HirBuilder<'a, 'cx> {
    lower: &'a mut Lower<'cx>,
    defined: Vec<TypeId>,
}

impl<'a, 'cx> ic_syntax::visit::Visitor<'a> for HirBuilder<'a, 'cx> {
    fn visit_module(&mut self, module: &'a ic_syntax::ModuleDef) {
        let definitions = module
            .definitions
            .iter()
            .flat_map(|v| lower_item(self.lower, v))
            .collect();

        let id = self.lower.ctx.arena.alloc_with_id(|id| {
            Type::Module(ModuleTy {
                id,
                ident: module.name.clone(),
                span: module.span,
                definitions,
            })
        });
        self.defined.push(id);
    }

    // Forward declarations are somewhat tricky. Types that depend on the
    // forward-declared type should point to the type definition and not the
    // declaration, but at this point we're not guaranteed to have the seen the
    // definition.
    //
    // Instead, we allocate two type entries: one for the declaration, and one
    // for the yet-to-be-seen definition. The declaration will point to the
    // definition, and future calls to `Context::resolve_type` will yield the
    // ID of the definition. When we encounter the definition, we will mutate
    // the existing entry. Once construction of the HIR is done, we'll check
    // that all types have been defined.
    fn visit_decl(&mut self, symbol: &'a ic_syntax::Decl) {
        self.lower.ctx.arena.alloc_with_id(|id| {
            Type::Decl(DeclTy {
                ident: symbol.name.clone(),
                ty: id,
            })
        });
    }

    fn visit_const(&mut self, symbol: &'a ic_syntax::ConstDef) {
        let ty = self.lower.ctx.resolve_type(&symbol.ty);

        self.lower.ctx.arena.alloc_with_id(|id| {
            Type::Const(hir::ConstTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                ty,
                value: Numeric::Octet(0),
            })
        });
    }
}

pub(crate) fn bitwise_eq(_lhs: &Numeric, _rhs: &Numeric) -> bool {
    false
}

fn lower_item<'cx>(lower: &mut Lower<'cx>, item: &Item) -> Vec<TypeId> {
    let mut builder = HirBuilder {
        lower,
        defined: vec![],
    };
    visit_item(&mut builder, item);
    builder.defined
}

fn all_unique(types: &[TypeId]) -> bool {
    let unique: HashSet<_> = types.iter().collect();
    unique.len() == types.len()
}

pub fn from_ast(ctx: &mut Context, ast: &[Item]) -> Vec<TypeId> {
    let mut state = Lower::with_ctx(ctx);
    for item in ast {
        state.lower_item(item);
    }
    debug_assert!(
        all_unique(&state.order),
        "order of types contains duplicate entries",
    );
    state.order
}
