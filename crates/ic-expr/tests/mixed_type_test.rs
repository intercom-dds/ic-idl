// Copyright 2024 KONGSBERG

#![allow(clippy::float_cmp)]

use ic_expr::idl_adapter::{IdlContext, IdlLiteral, Numeric};
use ic_expr::{Binary, Expr, Op, Ternary, Unary, eval};

#[test]
fn test_mixed_type_arithmetic_promotion() {
    // Test: Int8 + Int16 should promote both to Int32
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int16(20))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(30)));

    // Test: Bool + Char should promote both to Int32
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(true))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Char('A'))), // 'A' = 65
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(66))); // 1 + 65 = 66
}

#[test]
fn test_unsigned_signed_mixing() {
    // Test: UInt32 + Int32 with same rank -> UInt32
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(100))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(50))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::UInt32(150)));

    // Test: Int32 - UInt32 -> UInt32 (may wrap)
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
        op: Op::Sub,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(20))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    // 10 - 20 in unsigned arithmetic wraps around
    assert!(matches!(result2, Numeric::UInt32(4_294_967_286)));
}

#[test]
fn test_promotion_preserves_value() {
    // Test that promotion preserves values correctly
    // Int8(-128) promoted to Int32 should be -128, not some wrapped value
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(-128))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(0))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(-128)));

    // UInt16(65535) promoted to Int32 should be 65535
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(65535))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(0))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(65535)));
}

#[test]
fn test_complex_mixed_expressions() {
    // Test: (Int8 + UInt16) * Int32
    // Int8(10) + UInt16(1000) -> both promote to Int32 -> Int32(1010)
    // Int32(1010) * Int32(2) -> Int32(2020)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(1000))),
        })),
        op: Op::Mul,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(2))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(2020)));
}

#[test]
fn test_shift_operations_with_mixed_types() {
    // Left operand is promoted, right operand is not
    // Int8(1) << UInt64(3) -> Int32(1) << 3 -> Int32(8)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(1))),
        op: Op::LShift,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt64(3))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(8)));

    // UInt16(256) >> Int8(4) -> Int32(256) >> 4 -> Int32(16)
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(256))),
        op: Op::RShift,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(4))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(16)));
}

#[test]
fn test_bitwise_ops_with_promotion() {
    // Int8 & Int16 -> both promote to Int32
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(0x0F))),
        op: Op::BitAnd,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int16(0xFF))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(0x0F)));

    // Octet | UInt16 -> both promote to Int32
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Octet(0xF0))),
        op: Op::BitOr,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(0x0F))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(0xFF)));
}

#[test]
fn test_comparison_with_mixed_types() {
    // Comparisons should still do type conversions
    // Int8(127) < UInt32(128) -> Int32(127) < UInt32(128) -> UInt32(127) < UInt32(128) -> true
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(127))),
        op: Op::Lt,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(128))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Bool(true)));

    // Int32(-1) < UInt32(1) -> UInt32(big number) < UInt32(1) -> false
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(-1))),
        op: Op::Lt,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(1))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Bool(false)));
}

#[test]
fn test_ternary_with_mixed_types() {
    // Ternary doesn't promote its branches, it just returns the selected value
    // true ? Int8(10) : UInt16(20) -> Int8(10)
    let expr = Expr::Ternary(Box::new(Ternary {
        cond: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(true))),
        then: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
        els: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(20))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int8(10)));

    // false ? Int32(10) : UInt32(20) -> UInt32(20)
    let expr2 = Expr::Ternary(Box::new(Ternary {
        cond: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(false))),
        then: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
        els: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(20))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::UInt32(20)));
}

#[test]
fn test_float_promotion_in_mixed_expressions() {
    // Int32 + Float -> Float
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(2.5))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Float(f) if (f - 12.5).abs() < f32::EPSILON));

    // Float + Double -> Double
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(1.0))),
        op: Op::Mul,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(std::f64::consts::PI))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Double(f) if (f - std::f64::consts::PI).abs() < 0.0001));
}

#[test]
fn test_unary_ops_with_promotion() {
    // -Int8(10) -> -Int32(10) -> Int32(-10)
    let expr = Expr::Unary(Box::new(Unary {
        op: Op::Sub,
        expr: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(-10)));

    // ~Octet(0xF0) -> ~Int32(0xF0) -> Int32(~0xF0)
    let expr2 = Expr::Unary(Box::new(Unary {
        op: Op::BitNot,
        expr: Expr::Lit(IdlLiteral::Numeric(Numeric::Octet(0xF0))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    // ~0xF0 in 32-bit = 0xFFFFFF0F, which is -241 in signed
    assert!(matches!(result2, Numeric::Int32(-241)));
}

#[test]
fn test_nested_mixed_type_expressions() {
    // Complex nested expression: ((Int8 + UInt16) * Int32) / (Octet - Int16)
    // (10 + 1000) * 5 / (200 - 50)
    // (Int32(1010) * Int32(5)) / (Int32(200) - Int32(50))
    // Int32(5050) / Int32(150) = Int32(33)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
                op: Op::Add,
                rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt16(1000))),
            })),
            op: Op::Mul,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(5))),
        })),
        op: Op::Div,
        rhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Octet(200))),
            op: Op::Sub,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int16(50))),
        })),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(33)));
}

#[test]
fn test_int64_promotion_edge_cases() {
    // UInt32 + Int64 -> Int64 (since Int64 can represent all UInt32 values)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(u32::MAX))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int64(1))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int64(4_294_967_296))); // u32::MAX + 1

    // UInt64 + Int64 -> UInt64 (different signedness, same rank)
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt64(100))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int64(50))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::UInt64(150)));
}

#[test]
fn test_edge_case_negation() {
    // -UInt32(1) -> Int64(-1) (to avoid overflow)
    let expr = Expr::Unary(Box::new(Unary {
        op: Op::Sub,
        expr: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(1))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int64(-1)));

    // -Bool(true) -> Int32(-1)
    let expr2 = Expr::Unary(Box::new(Unary {
        op: Op::Sub,
        expr: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(true))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(-1)));
}

#[test]
fn test_logical_ops_with_mixed_types() {
    // Logical ops should convert operands to bool
    // Int8(0) && UInt32(1) -> false && true -> false
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(0))),
        op: Op::And,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(1))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Bool(false)));

    // Float(0.0) || Double(3.14) -> false || true -> true
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(0.0))),
        op: Op::Or,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(std::f64::consts::PI))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Bool(true)));
}

#[test]
fn test_comparison_returns_bool() {
    // All comparison operations should return Bool regardless of operand types
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(std::f32::consts::PI))),
        op: Op::GtEq,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(2.71))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Bool(true)));

    // Even with mixed integer types
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(-10))),
        op: Op::NotEq,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt64(10))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Bool(true)));
}

#[test]
fn test_c_style_char_arithmetic() {
    // In C, char participates in arithmetic as an integer
    // 'A' + 1 -> 65 + 1 -> 66 -> 'B'
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Char('A'))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(1))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    // Char promotes to Int32 for arithmetic
    assert!(matches!(result, Numeric::Int32(66)));

    // 'z' - 'a' = 122 - 97 = 25
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Char('z'))),
        op: Op::Sub,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Char('a'))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(25)));
}

#[test]
fn test_mixed_signedness_division() {
    // Division with mixed signedness
    // Int32(-10) / UInt32(3) -> UInt32(big) / UInt32(3)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(-10))),
        op: Op::Div,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(3))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    // -10 as UInt32 is 4294967286, divided by 3 is 1431655762
    assert!(matches!(result, Numeric::UInt32(1_431_655_762)));

    // UInt32(10) / Int32(-3) -> UInt32(10) / UInt32(big)
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt32(10))),
        op: Op::Div,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(-3))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    // -3 as UInt32 is 4294967293, 10 / 4294967293 = 0
    assert!(matches!(result2, Numeric::UInt32(0)));
}
