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

use std::collections::HashSet;
use std::rc::Rc;

use ic_alloc::insensitive::CaseMap;
use ic_cli::color::Colorize;
use ic_diagnostic::{Diag, Label, error_span};
use ic_macros::EnumIter;
use ic_syntax::util::{self, path_name, type_name};
use ic_syntax::{Expr, Ident, Span};

use crate::Context;
use crate::hir::{
    AliasTy, Ann, AnnArg, AnnotationTy, BitFlag, BitmaskTy, ConstTy, Decl, Def, DefFlags, DefId,
    DefKind, EnumLit, EnumTy, ExceptTy, InterfaceTy, Member, ModuleTy, Numeric, ParamKind,
    Parameter, PrimitiveTy, ProtoTy, StructTy, Ty, TyKind, TypeId, UnionTy, Variant,
};
use crate::interp::Interp;
use crate::resolve::{self, ResolveError, Resolver, Symbol, SymbolKind};

/// Responsible for lowering the AST to a HIR. This process will, amongst other
/// things, perform type resolution, evaluate expressions, assign values to
/// things like enumerators, and ultimately construct the type-resolved graph
/// that is the HIR.
///
/// The HIR will alter the representation of the source code in some minor
/// ways, such as expanding a typedef with multiple declarators to multiple
/// typedefs with a single declarator each. More opinionated transformations of
/// the source code happens as subsequent passes on the HIR.
///
/// Note: non-critical warnings and errors are better implemented as lints.
pub(crate) struct Lower<'a> {
    ctx: &'a mut Context,
    order: Vec<TypeId>,
    pub resolver: Resolver,
    errors: Vec<Diag>,
}

impl<'a> Lower<'a> {
    fn with_ctx(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            order: vec![],
            resolver: Resolver::new(),
            errors: vec![],
        }
    }

    fn check_name_consistency(&self, lhs: &Ident, rhs: &Ident) -> bool {
        lhs.name.eq_ignore_ascii_case(&rhs.name)
    }

    /// Determines if two annotation definitions are consistent. The standard
    /// doesn't clarify what "consistent" means, but I've interpreted it as the
    /// two definitions being identical.
    fn check_ann_consistency(
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
    fn check_decl_consistency(
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
    fn check_bound_consistency(
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
    fn check_expr_consistency(&mut self, lhs: &ic_syntax::Expr, rhs: &ic_syntax::Expr) -> bool {
        self.eval_expr(lhs) == self.eval_expr(rhs)
    }

    /// Determines if two types are semantically consistent. Collection types
    /// are treated as consistent if they have the same bound and resolve to
    /// the same element type.
    fn check_type_consistency(&mut self, lhs: &ic_syntax::Type, rhs: &ic_syntax::Type) -> bool {
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

    fn eval_expr(&self, expr: &ic_syntax::Expr) -> Numeric {
        // TODO: can we make this accept TypeId instead?
        Interp { lower: self }.eval_expr_ty::<i64>(expr)
    }

    fn bound_expr(&self, expr: &ic_syntax::Expr) -> usize {
        match self.eval_expr(expr) {
            Numeric::Int32(v) => v as usize,
            Numeric::UInt32(v) => v as usize,
            Numeric::Int64(v) => v as usize,
            Numeric::UInt64(v) => v as usize,
            _ => 0,
        }
    }

    /// **NB**: If lookup fails, this will produce an error. It should not be
    /// used for fallible lookups.
    fn lookup_path(&mut self, path: ic_syntax::Path) -> Option<DefId> {
        let qualified = {
            let segments = path
                .segments
                .iter()
                .map(|v| v.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            if path.leading_colons.is_some() {
                format!("::{segments}")
            } else {
                segments
            }
        };

        let resolved = self.resolver.resolve_path(&path);
        match resolved {
            Ok(id) => Some(id),
            Err(err) => {
                Self::report_error(&qualified, err, &mut self.errors);
                None
            }
        }
    }

    /// Creates an array type with the given bound(s).
    fn array_type(&mut self, mut ty: Ty, bounds: &[ic_syntax::Expr]) -> Ty {
        // Start with the outermost bound
        let mut bounds = bounds.iter().rev();
        for b in bounds {
            let len = self.bound_expr(b);
            let span = ty.span;
            ty = Ty {
                kind: TyKind::Array {
                    ty: Box::new(ty),
                    len,
                },
                span,
            };
        }
        ty
    }

    fn update_type(&mut self, id: DefId, data: DefKind) {
        let def = self.ctx.definitions.get_mut(id);
        def.kind = data;
        def.flags.unset(DefFlags::IS_INCOMPLETE);
    }

    /// All definitions found in HIR contain their own ID, which requires
    /// allocating an ID before the type has been defined. In addition, since
    /// a definition can directly refer to itself, the symbol must also be
    /// registered before the type's members have been processed.
    ///
    /// This function allocates a placeholder declaration for the type, calls
    /// the given closure and then replaces the previously allocated
    /// placeholder in-place with the definition returned by the closure.
    fn alloc_in_place(
        &mut self,
        ident: Ident,
        kind: SymbolKind,
        f: impl FnOnce(&mut Self, Ident, DefId) -> Def,
    ) -> DefId {
        // TODO: It might be better to have an arena of
        // ```
        // enum {
        //     Complete(..),
        //     Incomplete(..),
        // }
        // ````
        // to better handle incomplete types instead of allocating a
        // temporary definition.
        let id = self.ctx.definitions.alloc_with_id(|id| Def {
            id,
            ident: ident.clone(),
            annotations: vec![],
            span: Span::default(),
            kind: DefKind::Decl(Decl::Struct),
            flags: DefFlags::IS_INCOMPLETE,
        });

        let res = f(self, ident, id);
        *self.ctx.definitions.get_mut(id) = res;
        id
    }

    // This is not a member function as borrowing the entirety of `Self` is
    // problematic when it's already borrowed in some closures. Ideally we
    // should have a separate error-reporting mechanism that is more flexible
    // than a vector.
    fn report_error(name: &str, error: ResolveError, errors: &mut Vec<Diag>) {
        let ident = name.yellow();
        let diag = match error {
            ResolveError::Undefined(span) => error_span(
                format!("failed to resolve `{ident}`"),
                Label::new(span).message("unknown type"),
            ),
            ResolveError::Redefined(span) => error_span(
                format!("duplicate registration of `{ident}`"),
                Label::new(span).message("redefined here"),
            ),
            ResolveError::DeclMismatch { decl, span, .. } => error_span(
                format!("`{ident}` was previously declared as a {}", decl.name()),
                Label::new(span).message("inconsistent type"),
            ),
            ResolveError::Superfluous(span) => error_span(
                format!("failed to resolve `{ident}`"),
                Label::new(span).message("unknown type"),
            ),
            ResolveError::Module(span) => error_span(
                format!("`{ident}` resolved to a module"),
                Label::new(span).message("expected type, found module"),
            ),
        };
        errors.push(diag);
    }

    /// Allocates a definition and registers the type in the resolver. This is
    /// only inteded to be used with ADTs.
    fn construct_type(
        &mut self,
        ident: Ident,
        kind: SymbolKind,
        f: impl FnOnce(&mut Self, Ident, DefId) -> Def,
    ) -> DefId {
        self.alloc_in_place(ident, kind, |this, ident, id| {
            if let Err(e) = this.resolver.define_type(&ident, id, kind) {
                Self::report_error(&ident.name, e, &mut this.errors);
            }
            f(this, ident, id)
        })
    }

    /// Creates a new lexical scope with the given `kind`. This is a helper
    /// function for dealing with modules, interfaces, valuetypes and
    /// annotations, all of which may contain nested type definitions.
    fn with_scope(
        &mut self,
        ident: Ident,
        kind: SymbolKind,
        f: impl FnOnce(&mut Self, Ident, DefId) -> Def,
    ) -> DefId {
        self.alloc_in_place(ident, kind, |this, ident, id| {
            let res = this.resolver.start_scope(&ident, id, kind);
            if let Err(e) = res {
                Self::report_error(&ident.name, e, &mut this.errors);
            }

            let def = f(this, ident, id);
            this.resolver.finish_scope();
            def
        })
    }

    /// Helper function for incrementing values in bitmasks and enums, where
    /// the previous value may or may not be known.
    fn increment_value(&self, expr: &Option<Expr>, prev: &mut Option<isize>) -> isize {
        let val = if let Some(expr) = expr {
            self.bound_expr(expr) as isize
        } else {
            prev.map_or(0, |v| v + 1)
        };
        _ = prev.insert(val);
        val
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

    fn lower_mod(&mut self, def: ic_syntax::ModuleDef) -> DefId {
        let annotations = self.lower_annotations(def.annotations);

        self.with_scope(def.ident, SymbolKind::Module, |this, ident, id| {
            let definitions = def
                .definitions
                .into_iter()
                .flat_map(|v| this.lower_item(v))
                .collect();

            Def {
                id,
                ident,
                annotations,
                span: def.span,
                kind: DefKind::Module(ModuleTy { definitions }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_struct(&mut self, def: ic_syntax::StructDef) -> DefId {
        let parent = def.parent.and_then(|v| self.lookup_path(v));
        let annotations = self.lower_annotations(def.annotations);

        self.construct_type(def.ident, SymbolKind::Struct, |this, ident, id| {
            let members = def
                .members
                .into_iter()
                .flat_map(|v| this.lower_field(v))
                .collect();

            Def {
                id,
                ident,
                annotations,
                span: def.span,
                kind: DefKind::Struct(StructTy { parent, members }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_type(&mut self, ty: ic_syntax::Type) -> Ty {
        use ic_syntax::Type;

        match ty {
            Type::Any(v) => Ty {
                kind: TyKind::Any,
                span: v.span,
            },
            Type::Fixed(v) => Ty {
                kind: TyKind::Fixed,
                span: v.span,
            },
            Type::Sequence(v) => Ty {
                kind: TyKind::Sequence {
                    ty: Box::new(self.lower_type(*v.ty)),
                    bound: v.bound.map(|e| self.bound_expr(&e)),
                },
                span: v.span,
            },
            Type::String(v) => Ty {
                kind: TyKind::String {
                    wide: v.wide,
                    bound: v.bound.map(|e| self.bound_expr(&e)),
                },
                span: v.span,
            },
            Type::Map(v) => Ty {
                kind: TyKind::Map {
                    key: Box::new(self.lower_type(*v.key)),
                    elem: Box::new(self.lower_type(*v.value)),
                    bound: v.bound.map(|e| self.bound_expr(&e)),
                },
                span: v.span,
            },
            Type::Path(v) => {
                // TODO: probably better to let the parser resolve these
                let path = &v.segments[0];
                let kind = match path.name.as_str() {
                    "boolean" => PrimitiveTy::Bool,
                    "char" => PrimitiveTy::Char,
                    "wchar" => PrimitiveTy::WChar,
                    "int8" => PrimitiveTy::Int8,
                    "octet" | "uint8" => PrimitiveTy::UInt8,
                    "int16" => PrimitiveTy::Int16,
                    "uint16" => PrimitiveTy::UInt16,
                    "int32" => PrimitiveTy::Int32,
                    "uint32" => PrimitiveTy::UInt32,
                    "int64" => PrimitiveTy::Int64,
                    "uint64" => PrimitiveTy::UInt64,
                    "float" => PrimitiveTy::Float32,
                    "double" => PrimitiveTy::Float64,
                    "long double" => PrimitiveTy::Float128,
                    _ => {
                        let span = util::path_span(&v);
                        let kind = self.lookup_path(v).map_or(TyKind::Any, TyKind::Adt);
                        return Ty { kind, span };
                    }
                };
                Ty {
                    kind: TyKind::Primitive(kind),
                    span: path.span,
                }
            }
        }
    }

    fn lower_annotation_def(&mut self, def: ic_syntax::AnnotationDef) -> DefId {
        use ic_syntax::AnnotationField;

        let mut members = vec![];
        let mut types = vec![];

        self.with_scope(def.ident, SymbolKind::Annotation, |this, ident, id| {
            for field in def.params {
                match field {
                    AnnotationField::Item(v) => {
                        let ids = this.lower_item(*v);
                        types.extend(ids);
                    }
                    AnnotationField::Member(v) => {
                        // TODO: default val
                        let ty = this.lower_type(v.ty);
                        let (ident, ty) = this.lower_declarator(v.decl, ty);
                        members.push(Member {
                            ident,
                            ty,
                            annotations: vec![],
                        });
                    }
                }
            }

            Def {
                id,
                ident,
                annotations: vec![],
                span: def.span,
                kind: DefKind::Annotation(AnnotationTy { members, types }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_field(&mut self, field: ic_syntax::Field) -> Vec<Member> {
        let annotations = self.lower_annotations(field.annotations);
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
        let annotations = self.lower_annotations(def.annotations);

        self.construct_type(def.ident, SymbolKind::Exception, |this, ident, id| {
            let members = def
                .members
                .into_iter()
                .flat_map(|v| this.lower_field(v))
                .collect();

            Def {
                id,
                ident,
                annotations,
                span: def.span,
                kind: DefKind::Except(ExceptTy { members }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_variant(&mut self, var: ic_syntax::UnionField) -> Variant {
        use ic_syntax::{Label, UnionElement};

        let mut is_default = false;
        let mut labels = vec![];
        let annotations = self.lower_annotations(var.annotations);

        for label in &var.labels {
            match label {
                Label::Case(v) => {
                    let num = self.eval_expr(v);
                    labels.push(num);
                }
                Label::Default(_) => {
                    is_default = true;
                }
            }
        }

        match var.field {
            UnionElement::Member(v) => {
                let ty = self.lower_type(*v.ty);
                let (ident, ty) = self.lower_declarator(v.decl, ty);

                Variant {
                    annotations,
                    ident,
                    ty,
                    labels,
                    is_default,
                }
            }
            UnionElement::Null(_) => todo!(),
        }
    }

    fn lower_union(&mut self, def: ic_syntax::UnionDef) -> DefId {
        let disc = self.lower_type(def.disc.ty);
        let annotations = self.lower_annotations(def.annotations);

        self.construct_type(def.ident, SymbolKind::Union, |this, ident, id| {
            let variants = def
                .fields
                .into_iter()
                .map(|v| this.lower_variant(v))
                .collect();

            Def {
                id,
                ident,
                annotations,
                span: def.span,
                kind: DefKind::Union(UnionTy { disc, variants }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_annotation(&mut self, ann: ic_syntax::AnnotationAppl) -> Ann {
        let path = ann.ident;
        let ty = None;
        let args = ann
            .args
            .into_iter()
            .map(|v| AnnArg {
                ident: v.ident,
                value: self.eval_expr(&v.value),
            })
            .collect();

        Ann { path, ty, args }
    }

    fn lower_annotations(&mut self, ann: Vec<ic_syntax::AnnotationAppl>) -> Vec<Ann> {
        ann.into_iter().map(|v| self.lower_annotation(v)).collect()
    }

    fn lower_enum_lit(&mut self, lit: ic_syntax::Enumerator, last: &mut Option<isize>) -> EnumLit {
        let value = self.increment_value(&lit.value, last);
        let annotations = self.lower_annotations(lit.annotations);

        EnumLit {
            ident: lit.ident,
            value,
            annotations,
        }
    }

    fn lower_enum(&mut self, def: ic_syntax::EnumDef) -> DefId {
        let mut last = None;
        let fields = def
            .fields
            .into_iter()
            .map(|lit| self.lower_enum_lit(lit, &mut last))
            .collect();

        let ty = Ty {
            kind: TyKind::Primitive(PrimitiveTy::UInt32),
            span: def.span,
        };
        let annotations = self.lower_annotations(def.annotations);

        self.construct_type(def.ident, SymbolKind::Struct, |this, ident, id| Def {
            id,
            ident,
            annotations,
            span: def.span,
            kind: DefKind::Enum(EnumTy { fields, ty }),
            flags: DefFlags::default(),
        })
    }

    fn lower_bitmask_flag(&mut self, lit: ic_syntax::Bit, last: &mut Option<isize>) -> BitFlag {
        let value = self.increment_value(&lit.value, last) as usize;
        let annotations = self.lower_annotations(lit.annotations);

        BitFlag {
            ident: lit.ident,
            value,
            annotations,
        }
    }

    fn lower_bitmask(&mut self, def: ic_syntax::BitmaskDef) -> DefId {
        let mut last_val = None;
        let ty = Ty {
            kind: TyKind::Primitive(PrimitiveTy::UInt32),
            span: def.span,
        };
        let annotations = self.lower_annotations(def.annotations);

        self.construct_type(def.ident, SymbolKind::Enum, |this, ident, id| {
            let flags = def
                .bits
                .into_iter()
                .map(|lit| this.lower_bitmask_flag(lit, &mut last_val))
                .collect();

            Def {
                id,
                ident,
                annotations,
                span: def.span,
                kind: DefKind::Bitmask(BitmaskTy { flags, ty }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_const(&mut self, def: ic_syntax::ConstDef) -> DefId {
        let value = self.eval_expr(&def.value);
        let ty = self.lower_type(def.ty);
        let (ident, ty) = self.lower_declarator(def.decl, ty);
        let annotations = self.lower_annotations(def.annotations);

        self.construct_type(ident, SymbolKind::Const, |_, ident, id| Def {
            id,
            ident,
            annotations,
            span: def.span,
            kind: DefKind::Const(ConstTy { value, ty }),
            flags: DefFlags::default(),
        })
    }

    fn create_alias(
        &mut self,
        decl: ic_syntax::Declarator,
        ty: Ty,
        span: Span,
        annotations: Vec<Ann>,
    ) -> DefId {
        let (ident, ty) = self.lower_declarator(decl, ty);
        self.construct_type(ident, SymbolKind::Typedef, |this, ident, id| Def {
            id,
            ident,
            annotations,
            span,
            kind: DefKind::Alias(AliasTy { ty }),
            flags: DefFlags::default(),
        })
    }

    // A typedef with multiple declarators will be expanded to multiple,
    fn lower_alias(&mut self, def: ic_syntax::AliasDef) -> Vec<DefId> {
        let ty = self.lower_type(def.ty);
        let annotations = self.lower_annotations(def.annotations);
        def.decl
            .into_iter()
            .map(|decl| self.create_alias(decl, ty.clone(), def.span, annotations.clone()))
            .collect()
    }

    fn lower_attrib(&mut self, def: ic_syntax::Attribute) -> Vec<()> {
        let ty = self.lower_type(def.ty);
        for decl in def.decl {
            let (_ident, _ty) = self.lower_declarator(decl, ty.clone());
        }
        vec![]
    }

    fn lower_interface(&mut self, def: ic_syntax::InterfaceDef) -> DefId {
        use ic_syntax::InterfaceMember;

        let mut prototypes = vec![];
        let mut attributes = vec![];
        let mut definitions = vec![];
        let is_local = def.local.is_some();
        let annotations = self.lower_annotations(def.annotations);

        let parents = def
            .inherits
            .into_iter()
            .flat_map(|v| self.lookup_path(v))
            .collect();

        self.with_scope(def.ident, SymbolKind::Interface, |this, ident, id| {
            for mem in def.members {
                match mem {
                    InterfaceMember::Attr(v) => {
                        let attrib = this.lower_attrib(v);
                        attributes.extend(attrib);
                    }
                    InterfaceMember::Proto(v) => {
                        let proto = this.lower_prototype(v);
                        prototypes.push(proto);
                    }
                    InterfaceMember::Item(item) => {
                        let ids = this.lower_item(item);
                        definitions.extend(ids);
                    }
                }
            }

            Def {
                id,
                ident,
                annotations,
                span: def.span,
                kind: DefKind::Interface(InterfaceTy {
                    parents,
                    prototypes,
                    attributes,
                    definitions,
                    is_local,
                }),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_prototype(&mut self, def: ic_syntax::Prototype) -> ProtoTy {
        let ident = def.ident;
        let ty = self.lower_type(def.ret);
        let params = def
            .params
            .into_iter()
            .map(|v| {
                let (ident, ty) = self.lower_declarator(v.decl, ty.clone());
                Parameter {
                    ident,
                    ty,
                    kind: v.kind.unwrap_or(ParamKind::In),
                }
            })
            .collect();

        ProtoTy { ident, ty, params }
    }

    fn lower_decl(&mut self, decl: ic_syntax::Decl) -> DefId {
        use ic_syntax::DeclKind;

        let (kind, symbol) = match decl.kind {
            DeclKind::Struct => (Decl::Struct, SymbolKind::Struct),
            DeclKind::Union => (Decl::Union, SymbolKind::Union),
            DeclKind::Native => (Decl::Native, SymbolKind::Valuetype),
            DeclKind::Interface => (Decl::Interface, SymbolKind::Interface),
            DeclKind::Valuetype => (Decl::Valuetype, SymbolKind::Valuetype),
        };

        self.ctx.definitions.alloc_with_id(|id| {
            if let Err(e) = self
                .resolver
                .declare_type(&decl.ident, Symbol::Decl(id, symbol))
            {
                Self::report_error(&decl.ident.name, e, &mut self.errors);
            }

            Def {
                id,
                ident: decl.ident,
                annotations: vec![],
                span: decl.span,
                kind: DefKind::Decl(kind),
                flags: DefFlags::default(),
            }
        })
    }

    fn lower_item(&mut self, item: ic_syntax::Item) -> Vec<DefId> {
        use ic_syntax::Item;

        let id = match item {
            Item::AnnotationValue(v) => self.lower_annotation_def(v),
            Item::ModuleValue(v) => self.lower_mod(v),
            Item::StructValue(v) => self.lower_struct(v),
            Item::UnionValue(v) => self.lower_union(v),
            Item::EnumValue(v) => self.lower_enum(v),
            Item::ExceptionValue(v) => self.lower_except(v),
            Item::BitmaskValue(v) => self.lower_bitmask(v),
            Item::ConstValue(v) => self.lower_const(v),
            Item::InterfaceValue(v) => self.lower_interface(v),
            Item::DeclValue(v) => self.lower_decl(v),
            Item::AliasValue(v) => return self.lower_alias(v),
            Item::ValuetypeValue(_) | Item::BitsetValue(_) => {
                // skipped for now
                return vec![];
            }
        };
        vec![id]
    }
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
        let ids = state.lower_item(item);
        state.order.extend(ids);
    }

    debug_assert!(
        all_unique(&state.order),
        "order of types contains duplicate entries",
    );

    // Perform some additional validation to ensure all types have been defined
    // and all scopes have been correctly closed.
    state.resolver.finish();

    (state.order, state.errors)
}
