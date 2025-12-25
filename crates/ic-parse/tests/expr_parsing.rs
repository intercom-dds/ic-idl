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
use ic_syntax::{Expr, Item, LiteralValue, OpKind};

/// Helper to extract the expression from a const with a given value expression.
fn parse_const_expr(input: &str) -> Expr {
    // Wrap the expression in a struct with an array bound to parse it
    let full = format!("struct S {{ long x[{input}]; }};");
    let result = from_str(&full);
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::StructValue(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    let field = &def.members[0];
    let ic_syntax::Declarator::Array(arr) = &field.names[0] else {
        panic!("expected array declarator")
    };
    arr.bounds[0].clone()
}

#[test]
fn parse_integer_literal() {
    let expr = parse_const_expr("42");
    match expr {
        Expr::Literal(lit) => {
            assert_eq!(lit.value, LiteralValue::Int(42));
        }
        _ => panic!("expected literal, got {expr:?}"),
    }
}

#[test]
fn parse_hex_literal() {
    let expr = parse_const_expr("0xFF");
    match expr {
        Expr::Literal(lit) => {
            assert_eq!(lit.value, LiteralValue::Int(255));
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parse_octal_literal() {
    let expr = parse_const_expr("0777");
    match expr {
        Expr::Literal(lit) => {
            assert_eq!(lit.value, LiteralValue::Int(511));
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parse_unary_minus() {
    let expr = parse_const_expr("-42");
    match expr {
        Expr::Unary(unary) => {
            assert_eq!(unary.op.kind, OpKind::Sub);
            match &unary.expr {
                Expr::Literal(lit) => {
                    assert_eq!(lit.value, LiteralValue::Int(42));
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
    match expr {
        Expr::Unary(unary) => {
            assert_eq!(unary.op.kind, OpKind::Add);
        }
        _ => panic!("expected unary expression"),
    }
}

#[test]
fn parse_unary_not() {
    let expr = parse_const_expr("~0xFF");
    match expr {
        Expr::Unary(unary) => {
            assert_eq!(unary.op.kind, OpKind::Not);
        }
        _ => panic!("expected unary expression"),
    }
}

#[test]
fn parse_binary_add() {
    let expr = parse_const_expr("1 + 2");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Add);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_sub() {
    let expr = parse_const_expr("5 - 3");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Sub);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_mul() {
    let expr = parse_const_expr("4 * 5");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Multiply);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_div() {
    let expr = parse_const_expr("10 / 2");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Divide);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_mod() {
    let expr = parse_const_expr("10 % 3");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Modulo);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_or() {
    let expr = parse_const_expr("1 | 2");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Or);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_xor() {
    let expr = parse_const_expr("1 ^ 2");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Xor);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_and() {
    let expr = parse_const_expr("3 & 1");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::And);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_lshift() {
    let expr = parse_const_expr("1 << 4");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Lshift);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_binary_rshift() {
    let expr = parse_const_expr("16 >> 2");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Rshift);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_precedence_mul_over_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let expr = parse_const_expr("1 + 2 * 3");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Add);
            // LHS should be 1
            match &binary.lhs {
                Expr::Literal(lit) => {
                    assert_eq!(lit.value, LiteralValue::Int(1));
                }
                _ => panic!("expected literal on lhs"),
            }
            // RHS should be 2 * 3
            match &binary.rhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::Multiply);
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Or);
            match &binary.rhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::Lshift);
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Xor);
            match &binary.rhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::And);
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Or, "outer should be OR");
            match &binary.rhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::Xor, "inner should be XOR");
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::And, "outer should be AND");
            match &binary.rhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::Lshift, "inner should be LSHIFT");
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Lshift, "outer should be LSHIFT");
            match &binary.rhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::Add, "inner should be ADD");
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Sub);
            // LHS should be (1 - 2)
            match &binary.lhs {
                Expr::Binary(inner) => {
                    assert_eq!(inner.op.kind, OpKind::Sub);
                    match &inner.lhs {
                        Expr::Literal(lit) => {
                            assert_eq!(lit.value, LiteralValue::Int(1));
                        }
                        _ => panic!("expected literal"),
                    }
                }
                _ => panic!("expected binary on lhs"),
            }
            // RHS should be 3
            match &binary.rhs {
                Expr::Literal(lit) => {
                    assert_eq!(lit.value, LiteralValue::Int(3));
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Multiply);
            // LHS should be grouped (1 + 2)
            match &binary.lhs {
                Expr::Group(group) => match &group.expr {
                    Expr::Binary(inner) => {
                        assert_eq!(inner.op.kind, OpKind::Add);
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Or);
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_unary_in_binary() {
    // -1 + 2 should parse as (-1) + 2
    let expr = parse_const_expr("-1 + 2");
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Add);
            match &binary.lhs {
                Expr::Unary(unary) => {
                    assert_eq!(unary.op.kind, OpKind::Sub);
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
    match expr {
        Expr::Binary(binary) => {
            assert_eq!(binary.op.kind, OpKind::Add);
            match &binary.lhs {
                Expr::Path(path) => {
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
    match expr {
        Expr::Binary(binary) => match &binary.lhs {
            Expr::Path(path) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].name, "Mod");
                assert_eq!(path.segments[1].name, "CONST");
            }
            _ => panic!("expected path on lhs"),
        },
        _ => panic!("expected binary expression"),
    }
}
