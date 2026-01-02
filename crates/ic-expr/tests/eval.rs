// Copyright 2026 KONGSBERG
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

use ic_expr::{Binary, EvalContext, Expr, IntRank, Op, SpannedError, Unary, Value, eval};

struct SimpleCtx;

impl EvalContext<i64, (), ()> for SimpleCtx {
    fn eval_literal(&mut self, lit: &i64, _span: ()) -> Result<Value<()>, SpannedError<(), ()>> {
        Ok(Value::Int(i128::from(*lit), IntRank::I64))
    }
}

fn lit(v: i64) -> Expr<i64, ()> {
    Expr::Lit(v, ())
}

fn bin(lhs: Expr<i64, ()>, op: Op, rhs: Expr<i64, ()>) -> Expr<i64, ()> {
    Expr::Binary(Box::new(Binary { lhs, op, rhs }))
}

fn unary(op: Op, expr: Expr<i64, ()>) -> Expr<i64, ()> {
    Expr::Unary(Box::new(Unary { op, expr }))
}

#[test]
fn eval_addition() {
    let expr = bin(lit(2), Op::Add, lit(3));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(5));
}

#[test]
fn eval_subtraction() {
    let expr = bin(lit(10), Op::Sub, lit(3));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(7));
}

#[test]
fn eval_multiplication() {
    let expr = bin(lit(4), Op::Mul, lit(5));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(20));
}

#[test]
fn eval_division() {
    let expr = bin(lit(20), Op::Div, lit(4));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(5));
}

#[test]
fn eval_modulo() {
    let expr = bin(lit(17), Op::Mod, lit(5));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(2));
}

#[test]
fn eval_negation() {
    let expr = unary(Op::Sub, lit(42));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-42));
}

#[test]
fn eval_bitwise_and() {
    let expr = bin(lit(0b1100), Op::BitAnd, lit(0b1010));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(0b1000));
}

#[test]
fn eval_bitwise_or() {
    let expr = bin(lit(0b1100), Op::BitOr, lit(0b1010));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(0b1110));
}

#[test]
fn eval_left_shift() {
    let expr = bin(lit(1), Op::LShift, lit(4));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(16));
}

#[test]
fn eval_right_shift() {
    let expr = bin(lit(16), Op::RShift, lit(2));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(4));
}

#[test]
fn eval_nested_expr() {
    // (2 + 3) * 4
    let expr = bin(bin(lit(2), Op::Add, lit(3)), Op::Mul, lit(4));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(20));
}

#[test]
fn eval_div_by_zero() {
    let expr = bin(lit(1), Op::Div, lit(0));
    let result = eval(&expr, &mut SimpleCtx);
    assert!(result.is_err());
}

#[test]
fn eval_mod_by_zero() {
    let expr = bin(lit(1), Op::Mod, lit(0));
    let result = eval(&expr, &mut SimpleCtx);
    assert!(result.is_err());
}
