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
use ic_alloc::insensitive::{CaseMap, CaseSet};
use ic_cli::color::Colorize;
use ic_diagnostic::{error_span, Diag, Label};
use ic_macros::EnumIter;
use ic_syntax::util::{path_name, type_name};
use ic_syntax::visit::{visit_item, Visitor};
use ic_syntax::{Expr, Ident, LiteralValue, Span};

use crate::hir::{Variant, *};
use crate::resolve::Resolver;
use crate::{Context, TypeId};

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
            OpKind::Sub => -val,
            OpKind::Not => !val,
            OpKind::Add => val,
            _ => panic!("invalid operator in unary expression"),
        }
    }

    fn eval_binary(&mut self, binary: &ic_syntax::Binary) -> i64 {
        use ic_syntax::OpKind;

        let lhs = self.to_value(&binary.lhs);
        let rhs = self.to_value(&binary.rhs);
        match binary.op.kind {
            OpKind::Add => lhs + rhs,
            OpKind::Sub => lhs - rhs,
            OpKind::Multiply => lhs * rhs,
            OpKind::Divide => lhs / rhs,
            OpKind::Modulo => lhs % rhs,
            OpKind::Lshift => lhs << rhs,
            OpKind::Rshift => lhs >> rhs,
            OpKind::Or => lhs | rhs,
            OpKind::Xor => lhs ^ rhs,
            OpKind::And => lhs & rhs,
            OpKind::Not => panic!("expected binary op, found bitwise NOT"),
        }
    }

    pub(crate) fn to_value(&mut self, expr: &ic_syntax::Expr) -> i64 {
        use ic_syntax::{Expr, LitKind};

        match expr {
            Expr::Literal(v) => match &v.value {
                LiteralValue::Bool(v) => i64::from(*v),
                LiteralValue::Int(v) => *v as i64,
                LiteralValue::Float(_) => todo!(),
                LiteralValue::Char(_) => todo!(),
                LiteralValue::String(_) => todo!(),
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
            Expr::Literal(v) => match &v.value {
                LiteralValue::Bool(v) => Numeric::Bool(*v),
                LiteralValue::Int(v) => Numeric::from(T::try_from(*v as i64).unwrap()),
                LiteralValue::Char(v) => Numeric::Char(*v),
                LiteralValue::String(ref v) => Numeric::String(v.clone()),
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
/// the source code happens as subsequent passes on the HIR.
///
/// Note: non-critical warnings and errors are better implemented as lints.
struct Lower<'a> {
    ctx: &'a mut Context,
    order: Vec<TypeId>,
    decls: Vec<TypeId>,
    resolver: Resolver,
    registered: CaseMap<'static, Span>,

    // Do we need to know the ID? don't think so? I think maybe the name is
    // enough?
    // global: Scope,
    scope: Vec<Scope>,
    errors: Vec<Diag>,
}

// enum Scope {
//     Module,
//     Interface,
// }
//
// struct SymbolTable {
//     table: HashMap<String, Symbol>,
//     parent: Option<SymbolTable>,
// }

#[derive(Debug)]
struct Scope {
    name: String,
    // symbols: CaseMap<DefId>,
    // scopes: CaseMap<Scope>,
}

impl<'a> Lower<'a> {
    fn with_ctx(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            order: vec![],
            decls: vec![],
            resolver: Resolver::default(),
            scope: vec![],
            registered: CaseMap::default(),
            errors: vec![],
            // global: CaseMap::default(),
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

        if !lhs.ident.name.eq_ignore_ascii_case(&rhs.ident.name)
            || lhs.params.len() != rhs.params.len()
        {
            return false;
        }

        lhs.params.iter().zip(rhs.params.iter()).all(|v| match v {
            (AnnotationField::Member(lhs), AnnotationField::Member(rhs)) => {
                true
                // self.check_decl_consistency(&lhs.names, &rhs.names)
                //     && self.check_type_consistency(&lhs.ty, &rhs.ty)
            }
            (AnnotationField::Item(lhs), AnnotationField::Item(rhs)) => {
                true
                // lhs.ident.name.eq_ignore_ascii_case(&rhs.name.name)
                //     && self.check_type_consistency(&lhs.ty, &rhs.ty)
                //     && self.check_expr_consistency(&lhs.value, &rhs.value)
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
            (Type::String(lhs), Type::String(rhs)) => {
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

    // // Forward declarations are somewhat tricky. Types that depend on the
    // // forward-declared type should point to the type definition and not the
    // // declaration, but at this point we're not guaranteed to have the seen the
    // // definition.``
    // //
    // // Instead, we allocate two type entries: one for the declaration, and one
    // // for the yet-to-be-seen definition. The declaration will point to the
    // // definition, and future calls to `Context::resolve_type` will yield the
    // // ID of the definition. When we encounter the definition, we will mutate
    // // the existing entry. Once construction of the HIR is done, we'll check
    // // that all types have been defined.
    // fn lower_decl(&mut self, symbol: ic_syntax::Decl) -> TypeId {
    //     self.ctx.types.alloc_with_id(|id| {
    //         Type::Decl(DeclTy {
    //             ident: symbol.ident.clone(),
    //             ty: id,
    //         })
    //     })
    // }

    fn lookup_path(&mut self, path: ic_syntax::Path) -> DefId {
        self.ctx
            .definitions
            .iter()
            .find(|(id, def)| def.ident.name == path.segments.last().as_ref().unwrap().name)
            .unwrap()
            .0
    }

    fn array_type(&mut self, mut ty: Ty, bounds: &[ic_syntax::Expr]) -> Ty {
        // Start with the outermost bound
        let mut bounds = bounds.iter().rev();
        for b in bounds {
            let len = self.bound_expr(b);
            ty = Ty::Array {
                ty: Box::new(ty),
                len,
            };
        }
        ty
    }

    fn qualified_name(&self, ident: &Ident) -> String {
        let mut segments: Vec<&str> = vec![];
        for scope in &self.scope {
            segments.push(&scope.name);
        }
        segments.push(&ident.name);
        segments.join("::")
    }

    /// Destructures a declarator into an (ident, type) tuple. For arrays, this
    /// will create a suitable type with the defined bounds.
    fn lower_declarator(&mut self, decl: ic_syntax::Declarator, ty: Ty) -> (Ident, Ty) {
        match decl {
            ic_syntax::Declarator::Simple(v) => (v, ty),
            ic_syntax::Declarator::Array(v) => {
                let ty = self.array_type(ty, &v.bounds);
                (v.ident, ty)
            }
        }
    }

    fn lower_type(&mut self, ty: ic_syntax::Type) -> Ty {
        use ic_syntax::Type;

        match ty {
            Type::Any(_) => Ty::Any,
            Type::Fixed(_) => Ty::Fixed,
            Type::Sequence(v) => Ty::Sequence {
                ty: Box::new(self.lower_type(*v.ty)),
                bound: v.bound.map(|e| self.bound_expr(&e)),
            },
            Type::String(v) => Ty::String {
                wide: v.wide,
                bound: v.bound.map(|e| self.bound_expr(&e)),
            },
            Type::Map(v) => Ty::Map {
                key: Box::new(self.lower_type(*v.key)),
                elem: Box::new(self.lower_type(*v.value)),
                bound: v.bound.map(|e| self.bound_expr(&e)),
            },
            Type::Path(v) => Ty::Adt(self.lookup_path(v)),
        }
    }

    fn with_scope<R>(&mut self, ident: &Ident, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scope.push(Scope {
            name: ident.name.clone(),
        });
        let ret = f(self);
        self.scope.pop();
        ret
    }

    // TODO: or constants? expr? numeric?
    //
    // or should we just store an enum with the respective ID?
    fn register_symbol(&mut self, ident: &Ident) {
        let name = self.qualified_name(ident);
        tracing::info!("registering symbol: {name}");

        match self.registered.entry(name) {
            Entry::Occupied(prev) => {
                let diag = error_span(
                    format!("duplicate registration of `{}`", prev.key().yellow()),
                    Label::new(ident.span).message("redefined here"),
                );
                self.errors.push(diag);

                tracing::error!(
                    "duplicate registration of {}, first registered near {:?}",
                    prev.key(),
                    prev.get(),
                );
            }
            Entry::Vacant(v) => {
                v.insert(ident.span);
            }
        }
    }

    // TODO: we need to register symbols too, for things like constants...
    fn register_type(&mut self, ident: &Ident, ty: TypeId) {
        // FIXME: this is already checked in the sanity lint. this can either
        // be removed or asserted
        if ident.name.is_empty() {
            panic!("attempted to register unnamed type");
        }

        self.register_symbol(ident);

        // TODO: Must handle forward dcls and check they are of the same type,
        // i.e. not a struct fwd dcl to a union def.
        let name = self.qualified_name(ident);
        match self.ctx.symbols.entry(name) {
            Entry::Occupied(_) => {}
            Entry::Vacant(v) => {
                v.insert(ty);
            }
        }
    }

    fn lower_mod(&mut self, def: ic_syntax::ModuleDef) -> DefId {
        let definitions: Vec<_> = self.with_scope(&def.ident, |this| {
            def.definitions
                .into_iter()
                .flat_map(|v| this.lower_item(v))
                .collect()
        });

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            annotations: vec![],
            span: def.span,
            kind: DefKind::Module(ModuleTy { definitions }),
        });
        self.register_type(&def.ident, id);
        id
    }

    fn lower_struct(&mut self, def: ic_syntax::StructDef) -> DefId {
        let parent = def.parent.map(|v| self.lookup_path(v));
        let members = self.with_scope(&def.ident, |this| {
            def.members
                .into_iter()
                .flat_map(|v| this.lower_field(v))
                .collect()
        });

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            annotations: vec![],
            span: def.span,
            kind: DefKind::Struct(StructTy { parent, members }),
        });

        self.register_type(&def.ident, id);
        id
    }

    fn define(&mut self, id: DefId) {
        let ident = self.ctx.definitions.get(id).ident.clone();
        self.register_type(&ident, id);
    }

    fn lower_field(&mut self, field: ic_syntax::Field) -> Vec<Member> {
        let annotations: Vec<_> = field
            .annotations
            .into_iter()
            .map(|v| self.lower_annotation(v))
            .collect();

        field
            .names
            .into_iter()
            .map(|decl| {
                let ty = self.lower_type(field.ty.clone());
                let (ident, ty) = self.lower_declarator(decl, ty);
                Member {
                    ident,
                    ty,
                    annotations: annotations.clone(),
                }
            })
            .collect()
    }

    fn lower_except(&mut self, def: ic_syntax::ExceptDef) -> DefId {
        let members = self.with_scope(&def.ident, |this| {
            def.members
                .into_iter()
                .flat_map(|v| this.lower_field(v))
                .collect()
        });

        self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident,
            annotations: vec![],
            span: def.span,
            kind: DefKind::Except(ExceptTy { members }),
        })
    }

    fn lower_variant(&mut self, var: ic_syntax::UnionField) -> Variant {
        use ic_syntax::{Label, UnionElement};

        let labels: Vec<_> = var
            .labels
            .into_iter()
            .map(|label| match label {
                Label::Case(v) => self.eval_expr(&v),
                Label::Default(_) => todo!(),
            })
            .collect();

        match var.field {
            UnionElement::Member(v) => {
                let ty = self.lower_type(*v.ty);
                let (ident, ty) = self.lower_declarator(v.decl, ty);

                Variant {
                    annotations: vec![],
                    ident,
                    ty,
                    labels,
                    is_default: false,
                }
            }
            UnionElement::Null(_) => todo!(),
        }
    }

    fn lower_union(&mut self, def: ic_syntax::UnionDef) -> DefId {
        let mut variants = self.with_scope(&def.ident, |this| {
            def.fields
                .into_iter()
                .map(|v| this.lower_variant(v))
                .collect()
        });

        let disc = self.lower_type(def.disc.ty);
        self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident,
            annotations: vec![],
            span: def.span,
            kind: DefKind::Union(UnionTy { disc, variants }),
        })
    }

    fn lower_annotation(&mut self, _ann: ic_syntax::AnnotationAppl) {}

    fn lower_enum_lit(&mut self, lit: ic_syntax::Enumerator, last: &mut isize) -> EnumLit {
        *last = lit
            .value
            .map(|v| self.bound_expr(&v) as isize)
            .unwrap_or_else(|| *last + 1);

        self.register_symbol(&lit.ident);
        EnumLit {
            ident: lit.ident,
            value: *last,
            annotations: vec![],
        }
    }

    fn lower_enum(&mut self, def: ic_syntax::EnumDef) -> DefId {
        let mut last = -1;
        let fields = def
            .fields
            .into_iter()
            .map(|lit| self.lower_enum_lit(lit, &mut last))
            .collect();

        // TODO: report conflicts in the resolver. this can go on as before
        // (though the member will not be registered(?)).
        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            annotations: vec![],
            span: def.span,
            kind: DefKind::Enum(EnumTy { fields }),
        });
        self.register_type(&def.ident, id);
        id
    }

    fn lower_bitmask_flag(&mut self, lit: ic_syntax::Bit, last: &mut isize) -> BitFlag {
        *last = lit
            .value
            .map(|v| self.bound_expr(&v) as isize)
            .unwrap_or_else(|| *last + 1);

        self.register_symbol(&lit.ident);
        BitFlag {
            ident: lit.ident,
            value: *last as usize,
            annotations: vec![],
        }
    }

    fn lower_bitmask(&mut self, def: ic_syntax::BitmaskDef) -> DefId {
        let mut last = 0;
        let flags: Vec<_> = def
            .bits
            .into_iter()
            .map(|lit| self.lower_bitmask_flag(lit, &mut last))
            .collect();

        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident.clone(),
            annotations: vec![],
            span: def.span,
            kind: DefKind::Bitmask(BitmaskTy {
                flags,
                ty: Ty::Primitive(PrimitiveTy::UInt32),
            }),
        });
        self.register_type(&def.ident, id);
        id
    }

    fn lower_const(&mut self, def: ic_syntax::ConstDef) -> DefId {
        let value = self.eval_expr(&def.value);
        let ty = self.lower_type(def.ty);
        let (ident, ty) = self.lower_declarator(def.decl, ty);

        self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident,
            annotations: vec![],
            span: def.span,
            kind: DefKind::Const(ConstTy { value, ty }),
        })
    }

    // A typedef with multiple declarators will be expanded to multiple,
    fn lower_alias(&mut self, def: ic_syntax::AliasDef) -> Vec<DefId> {
        let ty = self.lower_type(def.ty);

        def.decl
            .into_iter()
            .map(|decl| {
                let (ident, ty) = self.lower_declarator(decl, ty.clone());
                self.ctx.definitions.alloc_with_id(|id| Def {
                    id,
                    ident,
                    annotations: vec![],
                    span: def.span,
                    kind: DefKind::Alias(AliasTy { ty }),
                })
            })
            .collect()
    }

    fn lower_interface(&mut self, def: ic_syntax::InterfaceDef) -> DefId {
        // TODO:
        let prototypes = vec![];
        let attributes = vec![];

        self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident,
            annotations: vec![],
            span: def.span,
            kind: DefKind::Interface(InterfaceTy {
                prototypes,
                attributes,
            }),
        })
    }

    fn lower_decl(&mut self, def: ic_syntax::Decl) -> DefId {
        use ic_syntax::DeclKind;

        let kind = match def.kind {
            DeclKind::Struct => Decl::Struct,
            DeclKind::Union => Decl::Union,
            DeclKind::Native => Decl::Native,
            DeclKind::Interface => Decl::Interface,
            DeclKind::Valuetype => Decl::Valuetype,
        };

        self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: def.ident,
            annotations: vec![],
            span: def.span,
            kind: DefKind::Decl(kind),
        })
    }

    fn lower_item(&mut self, item: ic_syntax::Item) -> Vec<DefId> {
        use ic_syntax::Item;
        tracing::debug!("lowering item: {item:?} in scope {:?}", self.scope);

        let id = match item {
            // Item::AnnotationValue(_) => todo!(),
            Item::ModuleValue(v) => self.lower_mod(v),
            Item::StructValue(v) => self.lower_struct(v),
            Item::UnionValue(v) => self.lower_union(v),
            Item::EnumValue(v) => self.lower_enum(v),
            Item::ExceptionValue(v) => self.lower_except(v),
            Item::BitmaskValue(v) => self.lower_bitmask(v),
            Item::ConstValue(v) => self.lower_const(v),
            Item::AliasValue(v) => {
                let ids = self.lower_alias(v);
                self.order.extend(ids.iter());
                return ids;
            }
            Item::InterfaceValue(v) => self.lower_interface(v),
            // Item::ValuetypeValue(_) => todo!(),
            Item::DeclValue(v) => self.lower_decl(v),
            // Item::BitsetValue(_) => return None,
            _ => todo!(),
        };

        // TODO: should we perhaps only push top-level `DefId`s?
        // since everything else can be driven by examining the tree.
        self.order.push(id);
        vec![id]
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
    // fn visit_forward_decl(&mut self, symbol: &'a ic_syntax::Decl) {
    //     let id = self.lower.ctx.types.alloc_with_id(|id| {
    //         Type::Decl(DeclTy {
    //             ident: symbol.ident.clone(),
    //             ty: id,
    //         })
    //     });
    //     self.declared.insert(id);
    // }
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

pub fn from_ast<I>(ctx: &mut Context, ast: I) -> (Vec<TypeId>, Vec<Diag>)
where
    I: IntoIterator<Item = ic_syntax::Item>,
{
    let mut state = Lower::with_ctx(ctx);
    for item in ast {
        state.lower_item(item);
    }
    assert!(
        state.scope.is_empty(),
        "{} lexical scope(s) was opened but never closed",
        state.scope.len(),
    );
    debug_assert!(
        all_unique(&state.order),
        "order of types contains duplicate entries",
    );
    tracing::info!("registered: {:#?}", state.registered);
    (state.order, state.errors)
}
