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

use ic_hir::hir::{DefFlags, DefKind};

mod common;

#[test]
fn test_enum_basic() {
    let input = r"
        enum BasicEnum {
            A,
            B,
            C
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the enum
    let basic_enum = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BasicEnum")
        .expect("BasicEnum definition not found");

    let DefKind::Enum(e) = &basic_enum.1.kind else {
        panic!("Expected enum definition")
    };

    assert_eq!(e.fields.len(), 3);

    let field_def_a = result.context.definitions.get(e.fields[0]);
    assert_eq!(field_def_a.ident.name, "A");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_a.kind, DefKind::Const).value),
        0
    );
    assert!(!field_def_a.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_b = result.context.definitions.get(e.fields[1]);
    assert_eq!(field_def_b.ident.name, "B");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_b.kind, DefKind::Const).value),
        1
    );
    assert!(!field_def_b.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_c = result.context.definitions.get(e.fields[2]);
    assert_eq!(field_def_c.ident.name, "C");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_c.kind, DefKind::Const).value),
        2
    );
    assert!(!field_def_c.flags.contains(DefFlags::IS_ENUMERATED));
}

#[test]
fn test_enum_value() {
    let input = r"
        const int64 VALUE = 20;

        enum TestEnum {
            @value(10)
            A,
            B,
            @value(VALUE)
            C,
            D
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the enum
    let basic_enum = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestEnum")
        .expect("TestEnum definition not found");

    let DefKind::Enum(e) = &basic_enum.1.kind else {
        panic!("Expected enum definition")
    };

    assert_eq!(e.fields.len(), 4);

    let field_def_a = result.context.definitions.get(e.fields[0]);
    assert_eq!(field_def_a.ident.name, "A");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_a.kind, DefKind::Const).value),
        10
    );
    assert!(field_def_a.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_b = result.context.definitions.get(e.fields[1]);
    assert_eq!(field_def_b.ident.name, "B");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_b.kind, DefKind::Const).value),
        11
    );
    assert!(!field_def_b.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_c = result.context.definitions.get(e.fields[2]);
    assert_eq!(field_def_c.ident.name, "C");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_c.kind, DefKind::Const).value),
        20
    );
    assert!(field_def_c.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_d = result.context.definitions.get(e.fields[3]);
    assert_eq!(field_def_d.ident.name, "D");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_d.kind, DefKind::Const).value),
        21
    );
    assert!(!field_def_d.flags.contains(DefFlags::IS_ENUMERATED));
}

#[test]
fn test_enum_value_non_standard() {
    let input = r"
        const int64 VALUE = 20;

        enum TestEnum {
            A = 10,
            B,
            C = VALUE,
            D
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the enum
    let basic_enum = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestEnum")
        .expect("TestEnum definition not found");

    let DefKind::Enum(e) = &basic_enum.1.kind else {
        panic!("Expected enum definition")
    };

    assert_eq!(e.fields.len(), 4);

    let field_def_a = result.context.definitions.get(e.fields[0]);
    assert_eq!(field_def_a.ident.name, "A");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_a.kind, DefKind::Const).value),
        10
    );
    assert!(field_def_a.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_b = result.context.definitions.get(e.fields[1]);
    assert_eq!(field_def_b.ident.name, "B");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_b.kind, DefKind::Const).value),
        11
    );
    assert!(!field_def_b.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_c = result.context.definitions.get(e.fields[2]);
    assert_eq!(field_def_c.ident.name, "C");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_c.kind, DefKind::Const).value),
        20
    );
    assert!(field_def_c.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_d = result.context.definitions.get(e.fields[3]);
    assert_eq!(field_def_d.ident.name, "D");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_d.kind, DefKind::Const).value),
        21
    );
    assert!(!field_def_d.flags.contains(DefFlags::IS_ENUMERATED));
}

#[test]
fn test_enum_value_not_integer() {
    let input = r#"
        enum TestEnum {
            A = 'A',
            @value("abc")
            B
        };
    "#;

    let errors = common::parse_and_expect_errors(input);

    insta::assert_snapshot!(errors);
}

#[macro_export]
macro_rules! expect_matches {
    ($expr:expr, $pattern:path) => {
        match $expr {
            $pattern(v) => v,
            _ => panic!("expected to pattern match {}", stringify!($pattern)),
        }
    };
}
