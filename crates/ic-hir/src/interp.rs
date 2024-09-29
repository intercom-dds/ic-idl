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

use std::fmt::Debug;
use std::ops::{Neg, Not};
use std::rc::Rc;

use crate::hir::{DefId, Numeric};
use crate::lower::Lower;
use crate::resolve::{self, Resolver, Symbol, SymbolKind};

pub struct Interp<'a> {
    pub(crate) lower: &'a Lower<'a>,
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
            _ => unreachable!("tried to negate non-primitive type"),
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
            _ => unreachable!("tried to negate non-primitive or floating-point type"),
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
            _ => unreachable!("invalid operator in unary expression"),
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
            OpKind::Not => unreachable!("expected binary op, found bitwise NOT"),
        }
    }

    pub(crate) fn to_value(&mut self, expr: &ic_syntax::Expr) -> i64 {
        use ic_syntax::{Expr, LiteralValue};

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
            Expr::Path(v) => Numeric::Const(self.lower.resolver.resolve_path(v).unwrap()),
            Expr::Unary(v) => Numeric::Int64(self.eval_unary(v)),
            Expr::Binary(v) => Numeric::Int64(self.eval_binary(v)),
            Expr::InitList(_) => todo!(),
        }
    }

    pub(crate) fn eval_expr_ty<T>(&mut self, expr: &ic_syntax::Expr) -> Numeric
    where
        T: TryFrom<i64>,
        T::Error: Debug,
        Numeric: From<T>,
    {
        use ic_syntax::{Expr, LiteralValue};

        match expr {
            Expr::Literal(v) => match &v.value {
                LiteralValue::Bool(v) => Numeric::Bool(*v),
                LiteralValue::Int(v) => Numeric::from(T::try_from(*v as i64).unwrap()),
                LiteralValue::Char(v) => Numeric::Char(*v),
                LiteralValue::String(ref v) => Numeric::String(v.clone()),
                LiteralValue::Float(_) => todo!(),
            },
            // TODO: this should always be a constant, enumerator oo bitflag.
            Expr::Path(v) => Numeric::Const(DefId::_do_not_use()),
            Expr::Unary(v) => Numeric::Int64(self.eval_unary(v)),
            Expr::Binary(v) => Numeric::Int64(self.eval_binary(v)),
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
            Expr::Path(v) => todo!(),
            Expr::InitList(_) => todo!(),
        };
        Ok(num)
    }
}
