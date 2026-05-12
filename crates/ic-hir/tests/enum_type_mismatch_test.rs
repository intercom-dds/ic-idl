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

mod common;

#[test]
fn mixing_enum_types_in_const_is_rejected() {
    let input = r"
        enum Color { RED };
        enum Shape { CIRCLE };
        const Color MY_COLOR = CIRCLE;
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}

#[test]
fn mixing_enum_types_through_qualified_name_is_rejected() {
    let input = r"
        enum Color { RED };
        enum Shape { CIRCLE };
        const Color MY_COLOR = Shape::CIRCLE;
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}

#[test]
fn same_enum_const_assignment_is_allowed() {
    common::parse_and_resolve_successfully(
        r"
        enum Color { RED, GREEN, BLUE };
        const Color MY_COLOR = RED;
        const Color OTHER_COLOR = Color::GREEN;
    ",
    );
}

#[test]
fn enum_arithmetic_to_integer_const_is_allowed() {
    common::parse_and_resolve_successfully(
        r"
        enum Color { RED, GREEN, BLUE };
        const int32 FOO = GREEN + 1;
        const int32 BAR = Color::BLUE;
    ",
    );
}

#[test]
fn enum_alias_to_same_enum_is_allowed() {
    common::parse_and_resolve_successfully(
        r"
        enum Color { RED, GREEN };
        typedef Color ColorAlias;
        const ColorAlias MY_COLOR = RED;
    ",
    );
}

#[test]
fn enum_alias_to_different_enum_is_rejected() {
    let input = r"
        enum Color { RED };
        enum Shape { CIRCLE };
        typedef Color ColorAlias;
        const ColorAlias MY_COLOR = CIRCLE;
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}

#[test]
fn union_case_label_from_wrong_enum_is_rejected() {
    let input = r"
        enum Color { RED, GREEN };
        enum Size { SMALL, LARGE };
        union U switch (Color) {
            case SMALL: long s;
        };
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}

#[test]
fn union_case_label_from_same_enum_is_allowed() {
    common::parse_and_resolve_successfully(
        r"
        enum Color { RED, GREEN };
        union U switch (Color) {
            case RED: long r;
            case GREEN: long g;
        };
    ",
    );
}

#[test]
fn parenthesized_foreign_enum_in_const_is_rejected() {
    let input = r"
        enum Color { RED };
        enum Shape { CIRCLE };
        const Color C = (CIRCLE);
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}

#[test]
fn arithmetic_foreign_enum_in_const_is_rejected() {
    let input = r"
        enum Color { RED };
        enum Shape { CIRCLE };
        const Color C = CIRCLE + 0;
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}

#[test]
fn arithmetic_foreign_enum_in_union_case_label_is_rejected() {
    let input = r"
        enum Color { RED };
        enum Shape { CIRCLE };
        union U switch (Color) {
            case CIRCLE + 0: long x;
        };
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
}
