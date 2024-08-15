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
use std::ops::{Neg, Not};
use std::rc::Rc;

use ic_alloc::arena::{Arena, Id};
use ic_macros::EnumIter;
use ic_syntax::util::{path_name, type_name};
use ic_syntax::visit::{visit_item, Visitor};
use ic_syntax::{Ident, Span};

use crate::hir::{
    self, AliasTy, BitmaskTy, ConstTy, DeclTy, EnumTy, Enumerator, Item, Member, ModuleTy, Numeric,
    PrimitiveTy, StructTy, TyFlags, Type, UnionTy,
};
use crate::{Context, TypeId};

pub struct Scope {
    symbols: HashMap<String, TypeId>,
}

pub struct Resolver {}

#[derive(Debug)]
pub struct Interp<'a> {
    ctx: &'a Context,
}

impl Interp<'_> {
    fn sub_num(&mut self, num: Numeric) -> Numeric {
        match num {
            // Use the NOT operator for unsigned numbers to simulate an
            // unsigned overflow
            Numeric::Bool(v) => Numeric::Int8(i8::from(v).not()),
            Numeric::Octet(v) => Numeric::Octet(v.not()),
            Numeric::UInt16(v) => Numeric::UInt16(v.not()),
            Numeric::UInt32(v) => Numeric::UInt32(v.not()),
            Numeric::UInt64(v) => Numeric::UInt64(v.not()),

            // Signed numbers are negated
            Numeric::Int8(v) => Numeric::Int8(v.neg()),
            Numeric::Int16(v) => Numeric::Int16(v.neg()),
            Numeric::Int32(v) => Numeric::Int32(v.neg()),
            Numeric::Int64(v) => Numeric::Int64(v.neg()),
            Numeric::Float(v) => Numeric::Float(v.neg()),
            Numeric::Double(v) => Numeric::Double(v.neg()),

            Numeric::Const(_) => todo!(),
            _ => panic!("tried to negate non-primitive type"),
        }
    }

    fn not_num(&mut self, mut num: Numeric) -> Numeric {
        match &mut num {
            Numeric::Bool(v) => *v = v.not(),
            Numeric::Octet(v) => *v = v.not(),
            Numeric::UInt16(v) => *v = v.not(),
            Numeric::UInt32(v) => *v = v.not(),
            Numeric::UInt64(v) => *v = v.not(),
            Numeric::Int8(v) => *v = v.not(),
            Numeric::Int16(v) => *v = v.not(),
            Numeric::Int32(v) => *v = v.not(),
            Numeric::Int64(v) => *v = v.not(),
            Numeric::Const(_) => todo!(),
            _ => panic!("tried to negate non-primitive or floating-point type"),
        }
        num
    }

    fn eval_unary(&mut self, unary: &ic_syntax::Unary) -> i64 {
        use ic_syntax::OpKind;

        let val = self.to_value(&unary.expr);
        match unary.op.kind {
            OpKind::OpSub => -val,
            OpKind::OpNot => !val,
            OpKind::OpAdd => val,
            _ => panic!("invalid operator in unary expression"),
        }
    }

    fn eval_binary(&mut self, binary: &ic_syntax::Binary) -> i64 {
        use ic_syntax::OpKind;

        let lhs = self.to_value(&binary.lhs);
        let rhs = self.to_value(&binary.rhs);
        match binary.op.kind {
            OpKind::OpAdd => lhs + rhs,
            OpKind::OpSub => lhs - rhs,
            OpKind::OpMultiply => lhs * rhs,
            OpKind::OpDivide => lhs / rhs,
            OpKind::OpModulo => lhs % rhs,
            OpKind::OpLshift => lhs << rhs,
            OpKind::OpRshift => lhs >> rhs,
            OpKind::OpOr => lhs | rhs,
            OpKind::OpXor => lhs ^ rhs,
            OpKind::OpAnd => lhs & rhs,
            OpKind::OpNot => panic!("expected binary op, found bitwise NOT"),
        }
    }

    pub(crate) fn to_value(&mut self, expr: &ic_syntax::Expr) -> i64 {
        use ic_syntax::{Expr, LitKind};

        match expr {
            Expr::Literal(v) => match &v.kind {
                LitKind::LitBool(v) => i64::from(*v),
                LitKind::LitInt(v) => *v as i64,
                LitKind::LitFloat(_) => todo!(),
                LitKind::LitChar(_) => todo!(),
                LitKind::LitString(_) => todo!(),
            },
            // ic_syntax::Expr::Path(v) => Numeric::Const(self.ctx.resolve_path(v)),
            Expr::Unary(v) => self.eval_unary(v),
            Expr::Binary(v) => self.eval_binary(v),
            _ => panic!("called to_value on a non-primitive numeric"),
        }
    }

    pub(crate) fn eval_expr(&mut self, expr: &ic_syntax::Expr) -> Numeric {
        use ic_syntax::Expr;

        match expr {
            Expr::Literal(v) => Numeric::Octet(0),
            Expr::Path(v) => Numeric::Const(self.ctx.resolve_path(v)),
            Expr::Unary(v) => Numeric::Int64(self.eval_unary(v) as i64),
            Expr::Binary(v) => Numeric::Int64(self.eval_binary(v) as i64),
            Expr::InitList(_) => todo!(),
        }
    }

    pub(crate) fn eval_expr_ty<T>(&mut self, expr: &ic_syntax::Expr) -> Numeric
    where
        T: TryFrom<i64>,
        T::Error: Debug,
        Numeric: From<T>,
    {
        use ic_syntax::{Expr, LitKind};

        match expr {
            Expr::Literal(v) => match v.kind {
                LitKind::LitBool(v) => Numeric::Bool(v),
                LitKind::LitInt(v) => Numeric::from(T::try_from(v as i64).unwrap()),
                LitKind::LitString(ref v) => Numeric::String(v.clone()),
                _ => todo!(),
            },
            Expr::Path(v) => Numeric::Const(self.ctx.resolve_path(v)),
            Expr::Unary(v) => Numeric::Int64(self.eval_unary(v) as i64),
            Expr::Binary(v) => Numeric::Int64(self.eval_binary(v) as i64),
            Expr::InitList(v) => todo!(),
        }
    }

    pub(crate) fn truncate<T>(&mut self, expr: &ic_syntax::Expr) -> Result<Numeric, T::Error>
    where
        T: TryFrom<i64>,
        Numeric: From<T>,
    {
        use ic_syntax::Expr;

        let num = match expr {
            Expr::Literal(_) => Numeric::from(T::try_from(0)?),
            Expr::Unary(v) => Numeric::from(T::try_from(self.eval_unary(v))?),
            Expr::Binary(v) => Numeric::from(T::try_from(self.eval_binary(v))?),
            Expr::Path(v) => Numeric::Const(self.ctx.resolve_path(v)),
            Expr::InitList(_) => todo!(),
        };
        Ok(num)
    }
}

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
    decls: Vec<TypeId>,
}

impl<'a> Lower<'a> {
    fn with_ctx(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            order: vec![],
            decls: vec![],
        }
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
        self.eval_expr(lhs) == self.eval_expr(rhs)
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
        // TODO: can we make this accept TypeId instead?
        Interp { ctx: &self.ctx }.eval_expr_ty::<i64>(expr)
    }

    pub(crate) fn bound_expr(&mut self, expr: &ic_syntax::Expr) -> usize {
        match self.eval_expr(expr) {
            Numeric::Int32(v) => v as usize,
            Numeric::UInt32(v) => v as usize,
            Numeric::Int64(v) => v as usize,
            Numeric::UInt64(v) => v as usize,
            _ => todo!(),
        }
    }

    fn register_type<I>(&mut self, name: I, ty: TypeId)
    where
        I: Into<String>,
    {
        self.ctx.symbols.insert(name.into(), ty);
    }

    /// Destructures a declarator into a (ident, type) tuple. For arrays, this
    /// will create a suitable type with the defined bounds.
    fn lower_declarator(&mut self, decl: &ic_syntax::Declarator, ty: TypeId) -> (Ident, TypeId) {
        match decl {
            ic_syntax::Declarator::Simple(v) => (v.clone(), ty),
            ic_syntax::Declarator::Array(v) => {
                let bounds: Vec<_> = v.bounds.iter().map(|v| self.bound_expr(v)).collect();
                let ty = self.lower_array(ty, &bounds);
                (v.ident.clone(), ty)
            }
        }
    }

    /// Constructs an array type.
    fn lower_array(&mut self, ty: TypeId, bounds: &[usize]) -> TypeId {
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
    fn lower_decl(&mut self, symbol: ic_syntax::Decl) -> TypeId {
        self.ctx.arena.alloc_with_id(|id| {
            Type::Decl(DeclTy {
                ident: symbol.name.clone(),
                ty: id,
            })
        })
    }

    fn lower_const(&mut self, symbol: ic_syntax::ConstDef) -> ConstTy {
        let ty = self.ctx.resolve_type(&symbol.ty);
        let value = match symbol.value {
            ic_syntax::Expr::InitList(v) => {
                Numeric::InitList(v.iter().map(|v| self.eval_expr(v)).collect())
            }
            v => self.eval_expr(&v),
        };

        ConstTy {
            ident: symbol.name,
            span: symbol.span,
            ty,
            value,
        }
    }

    // A typedef with multiple declarators will be expanded to multiple,
    // individual typedefs, each with one declarator.
    fn lower_alias(&mut self, symbol: ic_syntax::Typedef) -> TypeId {
        let ty = self.ctx.resolve_type(&symbol.ty);
        let id = self.ctx.arena.alloc_with_id(|id| {
            Type::Alias(AliasTy {
                id,
                ident: symbol.name.clone(),
                span: symbol.span,
                ty,
            })
        });

        self.register_type(symbol.name.name.clone(), id);
        id
    }

    fn lower_mod(&mut self, symbol: ic_syntax::ModuleDef) -> ModuleTy {
        let definitions = symbol
            .definitions
            .into_iter()
            .filter_map(|v| self.lower_item(v))
            .collect();

        // self.ctx.items.alloc_with_id(|id| {
        ModuleTy {
            // id,
            ident: symbol.name,
            span: symbol.span,
            definitions,
        }
        // })
    }

    // Members with multiple declarators are expanded into multiple members,
    // each with a single declarator.
    fn lower_struct(&mut self, symbol: ic_syntax::StructDef) -> TypeId {
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
        self.register_type(symbol.name.name, id);
        id
    }

    fn lower_except(&mut self, _: ic_syntax::ExceptDef) -> TypeId {
        todo!()
    }

    fn lower_union(&mut self, symbol: ic_syntax::UnionDef) -> TypeId {
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
                ident: symbol.name,
                span: symbol.span,
                disc,
                variants,
            })
        })
    }

    fn lower_bitmask(&mut self, symbol: ic_syntax::BitmaskDef) -> TypeId {
        let bits = symbol.bits.iter().cloned().map(|v| (v.name, 0)).collect();
        self.ctx.arena.alloc_with_id(|id| {
            Type::Bitmask(BitmaskTy {
                id,
                ident: symbol.name,
                span: symbol.span,
                ty: PrimitiveTy::UInt32,
                bits,
            })
        })
    }

    fn lower_enum(&mut self, symbol: ic_syntax::EnumDef) -> TypeId {
        let mut last_value = 0;
        let mut enumerators = vec![];

        for lit in &symbol.fields {
            let value = lit
                .value
                .as_ref()
                .map_or_else(|| last_value + 1, |e| self.bound_expr(e) as i64);

            last_value = value;

            enumerators.push(Enumerator {
                ident: lit.name.clone(),
                value,
            });
        }

        self.ctx.arena.alloc_with_id(|id| {
            Type::Enum(EnumTy {
                id,
                ident: symbol.name,
                span: symbol.span,
                ty: PrimitiveTy::UInt32,
                enumerators,
            })
        })
    }

    fn lower_item2(&mut self, item: ic_syntax::Item) -> Option<Item> {
        use ic_syntax as syn;

        Some(match item {
            syn::Item::AnnotationValue(_) => todo!(),
            syn::Item::ModuleValue(v) => Item::Module(self.lower_mod(v)),
            syn::Item::StructValue(v) => Item::Adt(self.lower_struct(v)),
            syn::Item::UnionValue(v) => Item::Adt(self.lower_union(v)),
            syn::Item::EnumValue(v) => Item::Adt(self.lower_enum(v)),
            syn::Item::ConstValue(v) => Item::Const(self.lower_const(v)),
            syn::Item::TypedefValue(v) => Item::Adt(self.lower_alias(v)),
            syn::Item::ExceptionValue(v) => Item::Adt(self.lower_except(v)),
            // syn::Item::InterfaceValue(v) => Item::Interface(self.lower_interface(v)),
            syn::Item::ValuetypeValue(_) => todo!(),
            // syn::Item::DeclValue(v) => Item::Decl(self.lower_decl(v)),
            syn::Item::BitmaskValue(v) => Item::Adt(self.lower_bitmask(v)),
            syn::Item::BitsetValue(_) => return None,
            _ => return None,
        })
    }

    // TODO: implement as visitor instead?
    fn lower_item(&mut self, item: ic_syntax::Item) -> Option<TypeId> {
        use ic_syntax::Item;

        let ty = match item {
            Item::AnnotationValue(_) => todo!(),
            Item::ModuleValue(v) => {
                self.lower_mod(v);
                return None;
            }
            Item::StructValue(v) => self.lower_struct(v),
            Item::UnionValue(v) => self.lower_union(v),
            Item::EnumValue(v) => self.lower_enum(v),
            Item::ExceptionValue(v) => self.lower_except(v),
            Item::BitmaskValue(v) => self.lower_bitmask(v),
            // Item::ConstValue(v) => self.lower_const(v),
            Item::TypedefValue(v) => self.lower_alias(v),
            Item::InterfaceValue(_) => todo!(),
            Item::ValuetypeValue(_) => todo!(),
            Item::DeclValue(v) => self.lower_decl(v),
            Item::BitsetValue(_) => return None,
            _ => todo!(),
        };
        self.order.push(ty);
        Some(ty)
    }
}

struct HirBuilder<'a, 'cx> {
    lower: &'a mut Lower<'cx>,

    /// IDs of top-level declarations in the order they were defined.
    defined: Vec<TypeId>,

    /// IDs of top-level declarations in the order they were defined.
    items: Vec<TypeId>,

    /// Types that have been declared but not yet defined.
    declared: HashSet<TypeId>,
}

impl<'a, 'cx> ic_syntax::visit::Visitor<'a> for HirBuilder<'a, 'cx> {
    // fn visit_module(&mut self, module: &'a ic_syntax::ModuleDef) {
    //     let definitions = module
    //         .definitions
    //         .iter()
    //         .flat_map(|v| lower_item(self.lower, v))
    //         .collect();
    //
    //     let id = self.lower.ctx.arena.alloc_with_id(|id| {
    //         Type::Module(ModuleTy {
    //             id,
    //             ident: module.name.clone(),
    //             span: module.span,
    //             definitions,
    //         })
    //     });
    //     self.defined.push(id);
    // }

    // Forward declarations are somewhat tricky. Types that depend on the
    // forward-declared type should point to the type definition and not the
    // declaration, but at this point we're not guaranteed to have the seen the
    // definition.
    //
    // Instead, we allocate two type entries: one for the declaration, and one
    // for the yet-to-be-seen definition. The declaration will point to the
    // definition, and future calls to `Context::resolve_type` will yield the
    // ID of the definition. When we encounter the definition, we will mutate
    // the existing entry. Once construction of the HIR is done, we'll iterate
    // over all declared types and ensure they've been defined.
    fn visit_decl(&mut self, symbol: &'a ic_syntax::Decl) {
        let id = self.lower.ctx.arena.alloc_with_id(|id| {
            Type::Decl(DeclTy {
                ident: symbol.name.clone(),
                ty: id,
            })
        });
        self.declared.insert(id);
    }
}

fn lower_item<'cx>(lower: &mut Lower<'cx>, item: &ic_syntax::Item) -> Vec<TypeId> {
    let mut builder = HirBuilder {
        lower,
        defined: vec![],
        items: vec![],
        declared: HashSet::new(),
    };
    visit_item(&mut builder, item);
    builder.defined
}

fn all_unique(types: &[TypeId]) -> bool {
    let unique: HashSet<_> = types.iter().collect();
    unique.len() == types.len()
}

pub fn from_ast<I>(ctx: &mut Context, ast: I) -> Vec<TypeId>
where
    I: IntoIterator<Item = ic_syntax::Item>,
{
    let mut state = Lower::with_ctx(ctx);
    for item in ast {
        state.lower_item2(item);
    }
    debug_assert!(
        all_unique(&state.order),
        "order of types contains duplicate entries",
    );
    state.order
}
