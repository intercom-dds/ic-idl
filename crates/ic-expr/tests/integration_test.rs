// Copyright 2024 KONGSBERG

use ic_expr::c_adapter::{CContext, CLiteral};
use ic_expr::idl_adapter::{IdlContext, IdlLiteral, Numeric};
use ic_expr::{Binary, Expr, Op, OverflowBehavior, Ternary, Unary, eval};

#[test]
fn test_c_and_idl_produce_same_results() {
    // Test that both adapters produce equivalent results for the same expression
    // Expression: (10 + 20) * 3 - 5

    // C version
    let c_expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(CLiteral::Int(10)),
                op: Op::Add,
                rhs: Expr::Lit(CLiteral::Int(20)),
            })),
            op: Op::Mul,
            rhs: Expr::Lit(CLiteral::Int(3)),
        })),
        op: Op::Sub,
        rhs: Expr::Lit(CLiteral::Int(5)),
    }));

    let mut c_ctx = CContext::new(|_| None);
    let c_result = eval(&c_expr, &mut c_ctx).unwrap();
    assert_eq!(c_result.0, 85); // (10 + 20) * 3 - 5 = 30 * 3 - 5 = 90 - 5 = 85

    // IDL version with Int32
    let idl_expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(10))),
                op: Op::Add,
                rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(20))),
            })),
            op: Op::Mul,
            rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(3))),
        })),
        op: Op::Sub,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(5))),
    }));

    let mut idl_ctx = IdlContext::new(|_| None);
    let idl_result = eval(&idl_expr, &mut idl_ctx).unwrap();
    assert!(matches!(idl_result, Numeric::Int32(85)));
}

#[test]
fn test_boolean_operations() {
    // Test: (5 > 3) && (10 != 10) should be false
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(5)),
            op: Op::Gt,
            rhs: Expr::Lit(CLiteral::Int(3)),
        })),
        op: Op::And,
        rhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(10)),
            op: Op::NotEq,
            rhs: Expr::Lit(CLiteral::Int(10)),
        })),
    }));

    let mut ctx = CContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert_eq!(result.0, 0); // false
}

#[test]
fn test_ternary_expression() {
    // Test: X > 0 ? X * 2 : -X
    let make_expr = |_x_val: i128| {
        Expr::Ternary(Box::new(Ternary {
            cond: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(CLiteral::Ident("X".to_string())),
                op: Op::Gt,
                rhs: Expr::Lit(CLiteral::Int(0)),
            })),
            then: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(CLiteral::Ident("X".to_string())),
                op: Op::Mul,
                rhs: Expr::Lit(CLiteral::Int(2)),
            })),
            els: Expr::Unary(Box::new(Unary {
                op: Op::Sub,
                expr: Expr::Lit(CLiteral::Ident("X".to_string())),
            })),
        }))
    };

    // Test with positive X
    let expr = make_expr(10);
    let mut ctx = CContext::new(|name| if name == "X" { Some(10) } else { None });
    let result = eval(&expr, &mut ctx).unwrap();
    assert_eq!(result.0, 20); // X > 0, so X * 2 = 20

    // Test with negative X
    let expr = make_expr(-10);
    let mut ctx = CContext::new(|name| if name == "X" { Some(-10) } else { None });
    let result = eval(&expr, &mut ctx).unwrap();
    assert_eq!(result.0, 10); // X <= 0, so -X = 10
}

#[test]
fn test_bitwise_operations() {
    // Test: (0xFF & 0x0F) | (0xA0 ^ 0x50)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(0xFF)),
            op: Op::BitAnd,
            rhs: Expr::Lit(CLiteral::Int(0x0F)),
        })),
        op: Op::BitOr,
        rhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(0xA0)),
            op: Op::BitXor,
            rhs: Expr::Lit(CLiteral::Int(0x50)),
        })),
    }));

    let mut ctx = CContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    // (0xFF & 0x0F) | (0xA0 ^ 0x50) = 0x0F | 0xF0 = 0xFF
    assert_eq!(result.0, 0xFF);
}

#[test]
fn test_overflow_behavior() {
    use ic_expr::EvalConfig;

    // Test overflow with wrapping (default)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(127))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int8(1))),
    }));

    let mut ctx = IdlContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert!(matches!(result, Numeric::Int32(128))); // Promoted to Int32, no overflow

    // Test actual Int32 overflow
    let expr2 = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(i32::MAX))),
        op: Op::Add,
        rhs: Expr::Lit(IdlLiteral::Numeric(Numeric::Int32(1))),
    }));

    let mut ctx2 = IdlContext::new(|_| None);
    let result2 = eval(&expr2, &mut ctx2).unwrap();
    assert!(matches!(result2, Numeric::Int32(i32::MIN))); // Wrapped around

    // Test overflow with error behavior
    let config = EvalConfig {
        overflow: OverflowBehavior::Error,
        max_shift: 63,
    };
    let mut ctx = IdlContext::with_config(config, |_| None);
    let result = eval(&expr2, &mut ctx);
    assert!(result.is_err()); // Should error on overflow
}

#[test]
fn test_short_circuit_evaluation() {
    // Test that || short-circuits on true
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(CLiteral::Int(1)), // true
        op: Op::Or,
        // This would error if evaluated
        rhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Int(1)),
            op: Op::Mod,
            rhs: Expr::Lit(CLiteral::Int(0)),
        })),
    }));

    let mut ctx = CContext::new(|_| None);
    let result = eval(&expr, &mut ctx).unwrap();
    assert_eq!(result.0, 1); // true, right side never evaluated
}
