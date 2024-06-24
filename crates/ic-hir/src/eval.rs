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

use std::ops::Neg;

use ic_syntax::visit::Visitor;
use ic_syntax::{Expr, LitKind, Literal, Op, OpKind, Span};

/// Evaluates arithmetic expressions.
struct Eval;

trait EvalExpr<T> {
    fn transform(&self) -> T;
}

enum Expr2 {
    Bool(bool),
    Int32(i32),
}

impl EvalExpr<i32> for Literal {
    fn transform(&self) -> i32 {
        match self.kind {
            LitKind::Bool => 0,
            LitKind::Int => 3,
            LitKind::Float => todo!(),
            LitKind::Char => todo!(),
            LitKind::String => todo!(),
        }
    }
}

impl EvalExpr<i32> for Expr {
    fn transform(&self) -> i32 {
        match self {
            Expr::Lit(lit) => match lit.kind {
                LitKind::Bool => 0,
                LitKind::Int => 3,
                LitKind::Float => todo!(),
                LitKind::Char => todo!(),
                LitKind::String => todo!(),
            },
            Expr::Path(_) => todo!(),
            Expr::Unary { op, expr } => match op.kind {
                OpKind::Add => expr.transform(),
                OpKind::Sub => -expr.transform(),
                OpKind::Not => !expr.transform(),
                _ => unreachable!("invalid unary operator"),
            },
            Expr::Binary { lhs, op, rhs } => {
                let lhs = lhs.transform();
                let rhs = rhs.transform();

                match op.kind {
                    OpKind::Add => lhs.wrapping_add(rhs),
                    OpKind::Sub => lhs.wrapping_sub(rhs),
                    OpKind::Multiply => lhs.wrapping_mul(rhs),
                    OpKind::Divide => lhs.wrapping_div(rhs),
                    OpKind::Modulo => lhs.wrapping_rem(rhs),
                    OpKind::LShift => lhs.overflowing_shl(rhs as u32).0,
                    OpKind::RShift => lhs.overflowing_shr(rhs as u32).0,
                    OpKind::Or => lhs | rhs,
                    OpKind::Xor => lhs ^ rhs,
                    OpKind::And => lhs & rhs,
                    OpKind::Not => unreachable!("invalid binary operator"),
                }
            }
            Expr::InitList(_) => unreachable!("invalid expression"),
        }
    }
}

impl<'a> Visitor<'a> for Eval {
    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Unary { op, expr } => match op.kind {
                OpKind::Add => todo!(),
                OpKind::Sub => Expr2::Int32(0_i32.wrapping_neg()),
                OpKind::Multiply => Expr2::Int32(1_i32.wrapping_mul(2)),
                OpKind::Divide => Expr2::Int32(1_i32.wrapping_div(2)),
                OpKind::Modulo => Expr2::Int32(1_i32.wrapping_rem(2)),
                OpKind::LShift => Expr2::Int32(1_i32.overflowing_shl(2).0),
                OpKind::RShift => Expr2::Int32(1_i32.overflowing_shr(2).0),
                OpKind::Or => Expr2::Int32(1_i32 | 2),
                OpKind::Xor => Expr2::Int32(1_i32 ^ 2),
                OpKind::And => Expr2::Int32(1_i32 & 2),
                OpKind::Not => Expr2::Int32(!1_i32),
            },
            Expr::Binary { lhs, op, rhs } => todo!(),
            Expr::InitList(_) => todo!(),
            Expr::Lit(_) | Expr::Path(_) => todo!(),
        };
    }
}

// pub fn eval<T>(expr: &Expr) -> T
// where
//     Expr: EvalExpr<T>,
// {
//     match expr {
//         Expr::Lit(lit) => lit.transform(),
//         Expr::Path(_) => todo!(),
//         Expr::Unary { op, expr } => match op.kind {
//             OpKind::Add => expr.transform(),
//             OpKind::Sub => -expr.transform(),
//             OpKind::Not => !expr.transform(),
//             _ => unreachable!("invalid unary operator"),
//         },
//         Expr::Binary { lhs, op, rhs } => {
//             let lhs = lhs.transform();
//             let rhs = rhs.transform();
//
//             match op.kind {
//                 OpKind::Add => lhs.wrapping_add(rhs),
//                 OpKind::Sub => lhs.wrapping_sub(rhs),
//                 OpKind::Multiply => lhs.wrapping_mul(rhs),
//                 OpKind::Divide => lhs.wrapping_div(rhs),
//                 OpKind::Modulo => lhs.wrapping_rem(rhs),
//                 OpKind::LShift => lhs.overflowing_shl(rhs as u32).0,
//                 OpKind::RShift => lhs.overflowing_shr(rhs as u32).0,
//                 OpKind::Or => lhs | rhs,
//                 OpKind::Xor => lhs ^ rhs,
//                 OpKind::And => lhs & rhs,
//                 OpKind::Not => unreachable!("invalid binary operator"),
//             }
//         }
//         Expr::InitList(_) => unreachable!("invalid expression"),
//     }
// }

#[test]
fn test_eval() {
    let expr = Expr::Binary {
        lhs: ic_alloc::ptr::P(Expr::Lit(ic_syntax::Literal {
            kind: LitKind::Int,
            span: Span::default(),
        })),
        op: Op {
            span: Span::default(),
            kind: OpKind::Multiply,
        },
        rhs: ic_alloc::ptr::P(Expr::Lit(ic_syntax::Literal {
            kind: LitKind::Int,
            span: Span::default(),
        })),
    };
    let foo = EvalExpr::<i32>::transform(&expr);
    assert_eq!(foo, 9);
}
