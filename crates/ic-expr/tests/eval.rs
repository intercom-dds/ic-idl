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

enum Lit {
    Int(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
}

struct SimpleCtx;

impl EvalContext<Lit, (), ()> for SimpleCtx {
    fn eval_literal(&mut self, lit: &Lit, _span: ()) -> Result<Value<()>, SpannedError<(), ()>> {
        match lit {
            Lit::Int(v) => Ok(Value::Int(i128::from(*v), IntRank::I64)),
            Lit::UInt8(v) => Ok(Value::UInt(u128::from(*v), IntRank::U8)),
            Lit::UInt16(v) => Ok(Value::UInt(u128::from(*v), IntRank::U16)),
            Lit::UInt32(v) => Ok(Value::UInt(u128::from(*v), IntRank::U32)),
            Lit::UInt64(v) => Ok(Value::UInt(u128::from(*v), IntRank::U64)),
        }
    }
}

fn lit(v: i64) -> Expr<Lit, ()> {
    Expr::Lit(Lit::Int(v), ())
}

fn u8lit(v: u8) -> Expr<Lit, ()> {
    Expr::Lit(Lit::UInt8(v), ())
}

fn u16lit(v: u16) -> Expr<Lit, ()> {
    Expr::Lit(Lit::UInt16(v), ())
}

fn u32lit(v: u32) -> Expr<Lit, ()> {
    Expr::Lit(Lit::UInt32(v), ())
}

fn u64lit(v: u64) -> Expr<Lit, ()> {
    Expr::Lit(Lit::UInt64(v), ())
}

fn bin(lhs: Expr<Lit, ()>, op: Op, rhs: Expr<Lit, ()>) -> Expr<Lit, ()> {
    Expr::Binary(Box::new(Binary { lhs, op, rhs }))
}

fn unary(op: Op, expr: Expr<Lit, ()>) -> Expr<Lit, ()> {
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

#[test]
fn eval_negate_uint8_i8_min() {
    let expr = unary(Op::Sub, u8lit(128));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-128));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -128);
            assert_eq!(rank, IntRank::I8);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint8_fits_in_i8() {
    let expr = unary(Op::Sub, u8lit(10));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-10));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -10);
            assert_eq!(rank, IntRank::I8);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint8_overflow() {
    let expr = unary(Op::Sub, u8lit(u8::MAX));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    match result {
        Value::UInt(v, rank) => {
            assert_eq!(v, 1);
            assert_eq!(rank, IntRank::U8);
        }
        _ => panic!("Expected UInt value for overflow case"),
    }
}

#[test]
fn eval_negate_uint16_i16_min() {
    let expr = unary(Op::Sub, u16lit(32768));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-32768));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -32768);
            assert_eq!(rank, IntRank::I16);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint16_fits_in_i16() {
    let expr = unary(Op::Sub, u16lit(1000));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-1000));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -1000);
            assert_eq!(rank, IntRank::I16);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint16_overflow() {
    let expr = unary(Op::Sub, u16lit(u16::MAX));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    match result {
        Value::UInt(v, rank) => {
            assert_eq!(v, 1);
            assert_eq!(rank, IntRank::U16);
        }
        _ => panic!("Expected UInt value for overflow case"),
    }
}

#[test]
fn eval_negate_uint32_i32_min() {
    let expr = unary(Op::Sub, u32lit(2147483648));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-2147483648));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -2147483648);
            assert_eq!(rank, IntRank::I32);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint32_fits_in_i32() {
    let expr = unary(Op::Sub, u32lit(1000));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-1000));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -1000);
            assert_eq!(rank, IntRank::I32);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint32_overflow() {
    let expr = unary(Op::Sub, u32lit(u32::MAX));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    match result {
        Value::UInt(v, rank) => {
            assert_eq!(v, 1);
            assert_eq!(rank, IntRank::U32);
        }
        _ => panic!("Expected UInt value for overflow case"),
    }
}

#[test]
fn eval_negate_uint64_i64_min() {
    let expr = unary(Op::Sub, u64lit(9223372036854775808));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-9223372036854775808));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -9223372036854775808);
            assert_eq!(rank, IntRank::I64);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint64_fits_in_i64() {
    let expr = unary(Op::Sub, u64lit(1000));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    assert_eq!(result.to_i128(), Some(-1000));
    match result {
        Value::Int(v, rank) => {
            assert_eq!(v, -1000);
            assert_eq!(rank, IntRank::I64);
        }
        _ => panic!("Expected Int value"),
    }
}

#[test]
fn eval_negate_uint64_overflow() {
    let expr = unary(Op::Sub, u64lit(u64::MAX));
    let result = eval(&expr, &mut SimpleCtx).unwrap();
    match result {
        Value::UInt(v, rank) => {
            assert_eq!(v, 1);
            assert_eq!(rank, IntRank::U64);
        }
        _ => panic!("Expected UInt value for overflow case"),
    }
}
