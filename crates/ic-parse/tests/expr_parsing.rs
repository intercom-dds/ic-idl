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

use ic_parse::from_str;
use ic_syntax::{Expr, ExprKind, Item, Literal, Op};

/// Helper to extract the expression from a const with a given value expression.
fn parse_const_expr(input: &str) -> Expr {
    // Wrap the expression in a struct with an array bound to parse it
    let full = format!("struct S {{ long x[{input}]; }};");
    let result = from_str(&full);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::Struct(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    let field = &def.fields[0];
    let ic_syntax::Declarator::Array(arr) = &field.declarators[0] else {
        panic!("expected array declarator")
    };
    arr.bounds[0].clone()
}

#[test]
fn parse_integer_literal() {
    let expr = parse_const_expr("42");
    match expr.value {
        ExprKind::Literal(lit) => {
            assert_eq!(lit, Literal::Int(42));
        }
        _ => panic!("expected literal, got {expr:?}"),
    }
}

#[test]
fn parse_hex_literal() {
    let expr = parse_const_expr("0xFF");
    match expr.value {
        ExprKind::Literal(lit) => {
            assert_eq!(lit, Literal::Int(255));
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parse_octal_literal() {
    let expr = parse_const_expr("0777");
    match expr.value {
        ExprKind::Literal(lit) => {
            assert_eq!(lit, Literal::Int(511));
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parse_unary_minus() {
    let expr = parse_const_expr("-42");
    match expr.value {
        ExprKind::Unary(unary) => {
            assert_eq!(unary.op.value, Op::Sub);
            match &unary.operand.value {
                ExprKind::Literal(lit) => {
                    assert_eq!(lit, &Literal::Int(42));
                }
                _ => panic!("expected literal"),
            }
        }
        _ => panic!("expected unary expression"),
    }
}

#[test]
fn parse_unary_plus() {
    let expr = parse_const_expr("+42");
    match expr.value {
        ExprKind::Unary(unary) => {
            assert_eq!(unary.op.value, Op::Add);
        }
        _ => panic!("expected unary expression"),
    }
}

#[test]
fn parse_unary_not() {
    let expr = parse_const_expr("~0xFF");
    match expr.value {
        ExprKind::Unary(unary) => {
            assert_eq!(unary.op.value, Op::Not);
        }
        _ => panic!("expected unary expression"),
    }
}

#[test]
fn parse_binary_add() {
    let expr = parse_const_expr("1 + 2");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Add);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_sub() {
    let expr = parse_const_expr("5 - 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Sub);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_mul() {
    let expr = parse_const_expr("4 * 5");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Multiply);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_div() {
    let expr = parse_const_expr("10 / 2");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Divide);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_mod() {
    let expr = parse_const_expr("10 % 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Modulo);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_or() {
    let expr = parse_const_expr("1 | 2");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Or);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_xor() {
    let expr = parse_const_expr("1 ^ 2");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Xor);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_and() {
    let expr = parse_const_expr("3 & 1");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::And);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_lshift() {
    let expr = parse_const_expr("1 << 4");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::LShift);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_rshift() {
    let expr = parse_const_expr("16 >> 2");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::RShift);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_mul_over_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let expr = parse_const_expr("1 + 2 * 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Add);
            // LHS should be 1
            match &binary.lhs.value {
                ExprKind::Literal(lit) => {
                    assert_eq!(lit, &Literal::Int(1));
                }
                _ => panic!("expected literal on lhs"),
            }
            // RHS should be 2 * 3
            match &binary.rhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::Multiply);
                }
                _ => panic!("expected binary on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_shift_over_or() {
    // 1 | 2 << 3 should parse as 1 | (2 << 3)
    let expr = parse_const_expr("1 | 2 << 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Or);
            match &binary.rhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::LShift);
                }
                _ => panic!("expected binary on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_and_over_xor() {
    // 1 ^ 2 & 3 should parse as 1 ^ (2 & 3)
    let expr = parse_const_expr("1 ^ 2 & 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Xor);
            match &binary.rhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::And);
                }
                _ => panic!("expected binary on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_xor_over_or() {
    // 1 | 2 ^ 3 should parse as 1 | (2 ^ 3)
    let expr = parse_const_expr("1 | 2 ^ 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Or, "outer should be OR");
            match &binary.rhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::Xor, "inner should be XOR");
                }
                _ => panic!("expected binary on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_shift_over_and() {
    // 1 & 2 << 3 should parse as 1 & (2 << 3)
    let expr = parse_const_expr("1 & 2 << 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::And, "outer should be AND");
            match &binary.rhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::LShift, "inner should be LSHIFT");
                }
                _ => panic!("expected binary on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_add_over_shift() {
    // 1 << 2 + 3 should parse as 1 << (2 + 3)
    let expr = parse_const_expr("1 << 2 + 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::LShift, "outer should be LSHIFT");
            match &binary.rhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::Add, "inner should be ADD");
                }
                _ => panic!("expected binary on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_left_associativity() {
    // 1 - 2 - 3 should parse as (1 - 2) - 3
    let expr = parse_const_expr("1 - 2 - 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Sub);
            // LHS should be (1 - 2)
            match &binary.lhs.value {
                ExprKind::Binary(inner) => {
                    assert_eq!(inner.op.value, Op::Sub);
                    match &inner.lhs.value {
                        ExprKind::Literal(lit) => {
                            assert_eq!(lit, &Literal::Int(1));
                        }
                        _ => panic!("expected literal"),
                    }
                }
                _ => panic!("expected binary on lhs"),
            }
            // RHS should be 3
            match &binary.rhs.value {
                ExprKind::Literal(lit) => {
                    assert_eq!(lit, &Literal::Int(3));
                }
                _ => panic!("expected literal on rhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_parenthesized() {
    let expr = parse_const_expr("(1 + 2) * 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Multiply);
            // LHS should be grouped (1 + 2)
            match &binary.lhs.value {
                ExprKind::Group(group) => match &group.value {
                    ExprKind::Binary(inner) => {
                        assert_eq!(inner.op.value, Op::Add);
                    }
                    _ => panic!("expected binary inside group"),
                },
                _ => panic!("expected group on lhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_complex_expression() {
    // (1 << 8) | (2 << 4) | 3
    let expr = parse_const_expr("(1 << 8) | (2 << 4) | 3");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Or);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_unary_in_binary() {
    // -1 + 2 should parse as (-1) + 2
    let expr = parse_const_expr("-1 + 2");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Add);
            match &binary.lhs.value {
                ExprKind::Unary(unary) => {
                    assert_eq!(unary.op.value, Op::Sub);
                }
                _ => panic!("expected unary on lhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_path_in_expression() {
    let expr = parse_const_expr("FOO + 1");
    match expr.value {
        ExprKind::Binary(binary) => {
            assert_eq!(binary.op.value, Op::Add);
            match &binary.lhs.value {
                ExprKind::Path(path) => {
                    assert_eq!(path.segments[0].name, "FOO");
                }
                _ => panic!("expected path on lhs"),
            }
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_scoped_path_in_expression() {
    let expr = parse_const_expr("Mod::CONST * 2");
    match expr.value {
        ExprKind::Binary(binary) => match &binary.lhs.value {
            ExprKind::Path(path) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].name, "Mod");
                assert_eq!(path.segments[1].name, "CONST");
            }
            _ => panic!("expected path on lhs"),
        },
        _ => panic!("expected binary expression"),
    }
}
