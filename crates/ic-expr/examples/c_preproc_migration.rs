// Copyright 2024 KONGSBERG
//
// Example showing how ic-preproc would migrate to use the generic expression evaluation

use ic_expr::c_adapter::{CContext, CLiteral};
use ic_expr::{Binary, Expr, Op, SimpleInt, Ternary, Unary, eval};

/// Example showing how to convert from ic_lexer tokens to CLiteral
fn token_to_literal(token: &str) -> CLiteral {
    if let Ok(n) = token.parse::<i128>() {
        CLiteral::Int(n)
    } else if token.starts_with('\'') && token.ends_with('\'') {
        // Character literal
        let ch = token.chars().nth(1).unwrap_or('\0');
        CLiteral::Char(ch)
    } else {
        // Identifier (macro or special)
        CLiteral::Ident(token.to_string())
    }
}

/// Example of evaluating a C preprocessor expression
fn evaluate_cpp_expr() {
    // Example: FOO * 2 + (BAR > 10 ? 100 : 0)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Ident("FOO".to_string())),
            op: Op::Mul,
            rhs: Expr::Lit(CLiteral::Int(2)),
        })),
        op: Op::Add,
        rhs: Expr::Ternary(Box::new(Ternary {
            cond: Expr::Binary(Box::new(Binary {
                lhs: Expr::Lit(CLiteral::Ident("BAR".to_string())),
                op: Op::Gt,
                rhs: Expr::Lit(CLiteral::Int(10)),
            })),
            then: Expr::Lit(CLiteral::Int(100)),
            els: Expr::Lit(CLiteral::Int(0)),
        })),
    }));

    // Create context with macro resolver
    let mut ctx = CContext::new(|name| match name {
        "FOO" => Some(42),
        "BAR" => Some(15),
        "__LINE__" => Some(123),
        _ => None, // Undefined macros evaluate to 0
    });

    // Evaluate the expression
    match eval(&expr, &mut ctx) {
        Ok(SimpleInt(result)) => {
            println!("Expression result: {}", result);
            // FOO * 2 + (BAR > 10 ? 100 : 0)
            // = 42 * 2 + (15 > 10 ? 100 : 0)
            // = 84 + 100
            // = 184
            assert_eq!(result, 184);
        }
        Err(e) => {
            println!("Evaluation error: {}", e);
        }
    }
}

/// Example showing how to handle #if directive evaluation
fn evaluate_if_directive() {
    // #if defined(DEBUG) && (VERSION > 2)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(CLiteral::Ident("DEBUG".to_string())),
        op: Op::And,
        rhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Ident("VERSION".to_string())),
            op: Op::Gt,
            rhs: Expr::Lit(CLiteral::Int(2)),
        })),
    }));

    let mut ctx = CContext::new(|name| match name {
        "DEBUG" => Some(1), // Defined
        "VERSION" => Some(3),
        _ => Some(0), // Undefined
    });

    match eval(&expr, &mut ctx) {
        Ok(SimpleInt(result)) => {
            if result != 0 {
                println!("#if condition is true - include this section");
            } else {
                println!("#if condition is false - skip this section");
            }
        }
        Err(e) => {
            println!("Error evaluating #if: {}", e);
        }
    }
}

/// Example with error handling
fn demonstrate_error_handling() {
    // Division by zero: X / (Y - Y)
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(CLiteral::Int(100)),
        op: Op::Div,
        rhs: Expr::Binary(Box::new(Binary {
            lhs: Expr::Lit(CLiteral::Ident("Y".to_string())),
            op: Op::Sub,
            rhs: Expr::Lit(CLiteral::Ident("Y".to_string())),
        })),
    }));

    let mut ctx = CContext::new(|name| match name {
        "Y" => Some(42),
        _ => None,
    });

    match eval(&expr, &mut ctx) {
        Ok(_) => println!("Unexpected success"),
        Err(ic_expr::Error::DivisionByZero) => {
            println!("Caught division by zero error as expected");
        }
        Err(e) => println!("Other error: {}", e),
    }
}

/// Example of shift operations with bounds checking
fn demonstrate_shift_operations() {
    // 1 << SHIFT_AMOUNT
    let expr = Expr::Binary(Box::new(Binary {
        lhs: Expr::Lit(CLiteral::Int(1)),
        op: Op::LShift,
        rhs: Expr::Lit(CLiteral::Ident("SHIFT_AMOUNT".to_string())),
    }));

    // Test with valid shift
    let mut ctx = CContext::new(|name| match name {
        "SHIFT_AMOUNT" => Some(5),
        _ => None,
    });

    match eval(&expr, &mut ctx) {
        Ok(SimpleInt(result)) => {
            println!("1 << 5 = {}", result);
            assert_eq!(result, 32);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test with invalid shift (too large)
    let mut ctx2 = CContext::new(|name| match name {
        "SHIFT_AMOUNT" => Some(200), // Too large!
        _ => None,
    });

    match eval(&expr, &mut ctx2) {
        Ok(_) => println!("Unexpected success"),
        Err(ic_expr::Error::InvalidShift(amount)) => {
            println!("Caught invalid shift error for amount: {}", amount);
        }
        Err(e) => println!("Other error: {}", e),
    }
}

fn main() {
    println!("=== C Preprocessor Expression Evaluation Examples ===\n");

    println!("1. Complex expression evaluation:");
    evaluate_cpp_expr();

    println!("\n2. #if directive evaluation:");
    evaluate_if_directive();

    println!("\n3. Error handling (division by zero):");
    demonstrate_error_handling();

    println!("\n4. Shift operations with bounds checking:");
    demonstrate_shift_operations();
}
