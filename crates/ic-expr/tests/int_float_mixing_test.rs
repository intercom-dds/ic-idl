// Copyright 2024 KONGSBERG

#![allow(clippy::approx_constant, clippy::cast_precision_loss)]

use ic_expr::idl_adapter::{IdlContext, IdlLiteral, Numeric};
use ic_expr::{Binary, Expr, Op, Ternary, Unary, eval};

#[test]
fn test_int_to_float_promotion() {
    let test_cases = vec![
        (Numeric::Bool(true), 1.0),
        (Numeric::Int8(42), 42.0),
        (Numeric::Octet(255), 255.0),
        (Numeric::Int16(-1000), -1000.0),
        (Numeric::UInt16(60000), 60000.0),
        (Numeric::Int32(-2_147_483_648), -2_147_483_648.0),
        (Numeric::UInt32(4_294_967_295), 4_294_967_295.0),
        (
            Numeric::Int64(-9_223_372_036_854_775_808i64),
            -9_223_372_036_854_775_808.0,
        ),
        (
            Numeric::UInt64(18_446_744_073_709_551_615u64),
            18_446_744_073_709_551_615.0,
        ),
        (Numeric::Char('A'), 65.0),
    ];

    let mut ctx = IdlContext::new(|_| None);

    for (int_val, expected) in test_cases {
        let expr = Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(int_val)),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(0.0))),
        }));

        let result = eval(&expr, &mut ctx).unwrap();
        match result {
            Numeric::Float(f) => assert!(
                (f - expected).abs() < f32::EPSILON,
                "Failed for {int_val:?}, got {f}, expected {expected}"
            ),
            _ => panic!("Expected Float, got {result:?}"),
        }
    }
}

#[test]
fn test_int_to_double_promotion() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int64(1_000_000))),
        op: Op::Mul,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(3.14159))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Double(d) => assert!((d - 3_141_590.0).abs() < 0.1),
        _ => panic!("Expected Double, got {result:?}"),
    }

    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::UInt64(u64::MAX))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(1.0))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    match result2 {
        Numeric::Double(d) => assert!(d > 1.8e19),
        _ => panic!("Expected Double, got {result2:?}"),
    }
}

#[test]
fn test_float_double_mixing() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(1.5))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(2.5))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Double(d) => assert!((d - 4.0).abs() < f64::EPSILON),
        _ => panic!("Expected Double, got {result:?}"),
    }
}

#[test]
fn test_mixed_arithmetic_with_floats() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(10))),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(0.5))),
        })),
        op: Op::Mul,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(2.0))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Double(d) => assert!((d - 21.0).abs() < f64::EPSILON),
        _ => panic!("Expected Double, got {result:?}"),
    }
}

#[test]
fn test_float_division() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
        op: Op::Div,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(3.0))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Float(f) => assert!((f - 3.33333).abs() < 0.0001),
        _ => panic!("Expected Float, got {result:?}"),
    }

    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(10.0))),
        op: Op::Div,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(3))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    match result2 {
        Numeric::Float(f) => assert!((f - 3.33333).abs() < 0.0001),
        _ => panic!("Expected Float, got {result2:?}"),
    }
}

#[test]
fn test_float_comparison() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(3.14))),
        op: Op::Gt,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(3))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Bool(true)));

    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(5))),
        op: Op::LtEq,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(5.0))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Bool(true)));
}

#[test]
fn test_float_unary_ops() {
    let expr = Expr::Unary(Box::new(Unary {
        op: Op::Sub,
        expr: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(3.14))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Float(f) => assert!((f - (-3.14)).abs() < f32::EPSILON),
        _ => panic!("Expected Float, got {result:?}"),
    }

    let expr2 = Expr::Unary(Box::new(Unary {
        op: Op::Sub,
        expr: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(2.71828))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    match result2 {
        Numeric::Double(d) => assert!((d - (-2.71828)).abs() < f64::EPSILON),
        _ => panic!("Expected Double, got {result2:?}"),
    }
}

#[test]
fn test_float_logical_ops() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(0.0))),
        op: Op::And,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(1))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Bool(false)));

    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(1.0))),
        op: Op::Or,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(0))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Bool(true)));
}

#[test]
fn test_complex_float_expressions() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
                op: Op::Add,
                rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(2.5))),
            })),
            op: Op::Mul,
            rhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(5.0))),
                op: Op::Sub,
                rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int64(2))),
            })),
        })),
        op: Op::Div,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(1.5))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Double(d) => assert!((d - 25.0).abs() < 0.0001),
        _ => panic!("Expected Double, got {result:?}"),
    }
}

#[test]
fn test_float_bitwise_ops_return_zero() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(3.14))),
        op: Op::BitAnd,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(5))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(0)));

    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(2.71))),
        op: Op::BitOr,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int64(42))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    assert!(matches!(result2, Numeric::Int32(0)));
}

#[test]
#[should_panic(expected = "modulo not supported for floating-point")]
fn test_float_modulo_error() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(10.0))),
        op: Op::Mod,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(3))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let _ = eval(&expr, &mut ctx).unwrap();
}

#[test]
#[should_panic(expected = "shift operations not supported for this type")]
fn test_float_shift_error() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(10.0))),
        op: Op::LShift,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(2))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let _ = eval(&expr, &mut ctx).unwrap();
}

#[test]
fn test_ternary_with_float() {
    let expr = Expr::Ternary(Box::new(Ternary {
        cond: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(true))),
        then: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(3.14))),
        els: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(42))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Float(f) => assert!((f - 3.14).abs() < f32::EPSILON),
        _ => panic!("Expected Float, got {result:?}"),
    }

    let expr2 = Expr::Ternary(Box::new(Ternary {
        cond: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(false))),
        then: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(42))),
        els: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(2.71828))),
    }));

    let result2 = eval(&expr2, &mut ctx).unwrap();
    match result2 {
        Numeric::Double(d) => assert!((d - 2.71828).abs() < f64::EPSILON),
        _ => panic!("Expected Double, got {result2:?}"),
    }
}

#[test]
fn test_float_precision_edge_cases() {
    let large_int = 16_777_217i32;
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(large_int))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(0.0))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Float(f) => {
            assert!((f - large_int as f32).abs() <= 1.0);
        }
        _ => panic!("Expected Float, got {result:?}"),
    }
}

#[test]
fn test_promotion_hierarchy() {
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Bool(true))),
                op: Op::Add,
                rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(1))),
            })),
            op: Op::Add,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Float(0.5))),
        })),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Double(0.25))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    match result {
        Numeric::Double(d) => assert!((d - 2.75).abs() < f64::EPSILON),
        _ => panic!("Expected Double, got {result:?}"),
    }
}
