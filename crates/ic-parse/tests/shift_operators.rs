// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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
use ic_syntax::{ExprKind, Item, Op};

#[test]
fn test_shift_operators_in_constants() {
    let result = from_str(
        r"
        const long LEFT_SHIFT = 1 << 4;
        const long RIGHT_SHIFT = 64 >> 2;
        const long COMPLEX = (1 << 8) + (256 >> 4);
    ",
    );
    assert!(result.errors.is_empty());

    // Verify the constants were parsed with shift operators
    let items = result.tree;
    assert_eq!(items.len(), 3);

    // Check that shift operators are present in the AST
    for item in &items {
        if let Item::Const(c) = item
            && let ExprKind::Binary(b) = &c.value.value
        {
            // Get the constant name from the declarator
            if let ic_syntax::Declarator::Name(ident) = &c.declarator {
                match &ident.name[..] {
                    "LEFT_SHIFT" => assert_eq!(b.op.value, Op::LShift),
                    "RIGHT_SHIFT" => assert_eq!(b.op.value, Op::RShift),
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test_nested_templates() {
    let result = from_str(
        r"
        typedef sequence<sequence<string>> StringMatrix;
        typedef map<string, sequence<long>> StringToSeq;
        typedef sequence<map<long, sequence<octet>>> ComplexType;
    ",
    );
    assert!(result.errors.is_empty());

    // Verify all typedefs were parsed successfully
    assert_eq!(result.tree.len(), 3);

    for item in &result.tree {
        assert!(matches!(item, Item::Alias(_)));
    }
}

#[test]
fn test_shift_in_template_bounds() {
    let result = from_str(
        r"
        typedef sequence<long, (1 << 10)> KB_Array;
        typedef sequence<octet, (256 >> 2)> SixtyFour_Array;
    ",
    );
    assert!(result.errors.is_empty());
    assert_eq!(result.tree.len(), 2);
}

#[test]
fn test_ambiguous_cases() {
    // Test case where >> could be ambiguous
    let result = from_str(
        r"
        // In expression context, >> is shift
        const long SHIFT = 1024 >> 2;

        // In template context, >> is two separate >
        typedef sequence<sequence<long>> Matrix;

        // Shift in template bound requires parentheses
        typedef sequence<octet, (1024 >> 2)> Array;
    ",
    );
    assert!(result.errors.is_empty());
    assert_eq!(result.tree.len(), 3);
}

#[test]
fn test_shift_operators_in_template_bounds_without_parens() {
    // Test shift operators in template bounds WITHOUT parentheses
    // This is the key C++-like parsing challenge
    let result = from_str(
        r"
        // Basic case: 1 >> 2 followed by > to close template
        typedef sequence<int32, 1 >> 2> ShiftBound;

        // Left shift works similarly
        typedef sequence<octet, 1 << 4> LeftShiftBound;

        // More complex: nested template with shift in inner bound
        typedef sequence<sequence<int32, 1 >> 2>> NestedWithShift;

        // Multiple operations with shift
        typedef sequence<long, 256 >> 2 + 1> ShiftThenAdd;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 4);

    // Verify each typedef was parsed
    for item in &result.tree {
        assert!(matches!(item, Item::Alias(_)));
    }
}

#[test]
fn test_shift_vs_template_closer_disambiguation() {
    // Verify that >> is correctly interpreted based on context:
    // - As shift when followed by expression-starting token
    // - As two > when followed by , or > or non-expression
    let result = from_str(
        r"
        // >> followed by number -> shift operator
        typedef sequence<int32, 8 >> 2> A;

        // >> followed by identifier -> shift operator
        const long N = 4;
        typedef sequence<int32, 16 >> N> B;

        // >> followed by unary minus -> shift operator
        typedef sequence<int32, 16 >> -1 + 3> C;

        // >> followed by ( -> shift operator
        typedef sequence<int32, 16 >> (1 + 1)> D;

        // Nested template: inner >> is closers, not shift
        typedef sequence<sequence<long>> E;

        // Mix: shift in bound, then nested template closes
        typedef sequence<sequence<int32, 4 >> 1>> F;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 7); // 1 const + 6 typedefs
}

#[test]
fn test_shift_in_multi_arg_template() {
    // fixed<D, S> has two args - first arg position knows more args follow,
    // so `>> ident ,` is treated as shift (comma = next template arg)
    let result = from_str(
        r"
        const long N = 2;

        // Shift with literal in first arg
        typedef fixed<1 >> 2, 3> LiteralShift;

        // Shift with identifier in first arg - comma means next arg, not declarator
        typedef fixed<1 >> N, 3> IdentShift;

        // Both args with shifts
        typedef fixed<8 >> 2, 16 >> 4> BothShift;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 4);
}

#[test]
fn test_deeply_nested_templates_with_shifts() {
    let result = from_str(
        r"
        // Three levels of nesting
        typedef sequence<sequence<sequence<long>>> Triple;

        // Three levels with shift in innermost
        typedef sequence<sequence<sequence<int32, 8 >> 2>>> TripleWithShift;

        // Map with sequence value containing shift bound
        typedef map<string, sequence<octet, 1024 >> 4>> MapWithShift;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 3);
}

#[test]
fn test_template_closers_with_declarator_list() {
    // Test that >> followed by identifier then comma is treated as template closers,
    // not shift operator. The identifier is a declarator name, not part of expression.
    let result = from_str(
        r"
        // Multiple declarators after nested template
        typedef sequence<sequence<int32>> a, b, c;

        // Single declarator (for comparison)
        typedef sequence<sequence<long>> single;

        // With bound in inner, multiple declarators
        typedef sequence<sequence<octet, 8>> x, y;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 3);
}

#[test]
fn test_rshift_followed_by_scoped_name_with_leading_dcolon() {
    let result = from_str(
        r"
        typedef string<a >> b @foo :: c> MyString;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 1);

    let result = from_str(
        r"
        typedef string<x >> y @min(1) :: z :: w> ComplexString;
    ",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 1);
}
