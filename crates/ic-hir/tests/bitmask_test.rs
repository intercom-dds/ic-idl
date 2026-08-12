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
fn test_bitmask_basic() {
    let input = r"
        bitmask BasicBitMask {
            A,
            B,
            C
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the bitmask
    let basic_bitmask = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "BasicBitMask")
        .expect("BasicBitMask definition not found");

    let DefKind::Bitmask(bitmask) = &basic_bitmask.1.kind else {
        panic!("Expected bitmask definition")
    };

    assert_eq!(bitmask.flags.len(), 3);

    let field_def_a = result.context.definitions.get(bitmask.flags[0]);
    assert_eq!(field_def_a.ident.name, "A");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_a.kind, DefKind::Const).value),
        1
    );
    assert!(!field_def_a.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_b = result.context.definitions.get(bitmask.flags[1]);
    assert_eq!(field_def_b.ident.name, "B");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_b.kind, DefKind::Const).value),
        1 << 1
    );
    assert!(!field_def_b.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_c = result.context.definitions.get(bitmask.flags[2]);
    assert_eq!(field_def_c.ident.name, "C");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_c.kind, DefKind::Const).value),
        1 << 2
    );
    assert!(!field_def_c.flags.contains(DefFlags::IS_ENUMERATED));
}

#[test]
fn test_bitmask_value() {
    let input = r"
        const uint32 VALUE = 8;

        bitmask TestBitMask {
            @position(2)
            A,
            B,
            @position(VALUE)
            C,
            D
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the bitmask
    let basic_bitmask = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestBitMask")
        .expect("TestBitMask definition not found");

    let DefKind::Bitmask(bitmask) = &basic_bitmask.1.kind else {
        panic!("Expected bitmask definition")
    };
    assert_eq!(bitmask.flags.len(), 4);

    let field_def_a = result.context.definitions.get(bitmask.flags[0]);
    assert_eq!(field_def_a.ident.name, "A");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_a.kind, DefKind::Const).value),
        1 << 2
    );
    assert!(field_def_a.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_b = result.context.definitions.get(bitmask.flags[1]);
    assert_eq!(field_def_b.ident.name, "B");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_b.kind, DefKind::Const).value),
        1 << 3
    );
    assert!(!field_def_b.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_c = result.context.definitions.get(bitmask.flags[2]);
    assert_eq!(field_def_c.ident.name, "C");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_c.kind, DefKind::Const).value),
        1 << 8
    );
    assert!(field_def_c.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_d = result.context.definitions.get(bitmask.flags[3]);
    assert_eq!(field_def_d.ident.name, "D");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_d.kind, DefKind::Const).value),
        1 << 9
    );
    assert!(!field_def_d.flags.contains(DefFlags::IS_ENUMERATED));
}

#[test]
fn test_bitmask_value_non_standard() {
    let input = r"
        bitmask TestBitMask {
            A = 2,
            B,
            C = 10,
            D
        };
    ";

    let result = common::parse_and_resolve_successfully(input);

    // Find the bitmask
    let basic_bitmask = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestBitMask")
        .expect("TestBitMask definition not found");

    let DefKind::Bitmask(bitmask) = &basic_bitmask.1.kind else {
        panic!("Expected bitmask definition")
    };
    assert_eq!(bitmask.flags.len(), 4);

    let field_def_a = result.context.definitions.get(bitmask.flags[0]);
    assert_eq!(field_def_a.ident.name, "A");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_a.kind, DefKind::Const).value),
        1 << 2
    );
    assert!(field_def_a.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_b = result.context.definitions.get(bitmask.flags[1]);
    assert_eq!(field_def_b.ident.name, "B");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_b.kind, DefKind::Const).value),
        1 << 3
    );
    assert!(!field_def_b.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_c = result.context.definitions.get(bitmask.flags[2]);
    assert_eq!(field_def_c.ident.name, "C");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_c.kind, DefKind::Const).value),
        1 << 10
    );
    assert!(field_def_c.flags.contains(DefFlags::IS_ENUMERATED));

    let field_def_d = result.context.definitions.get(bitmask.flags[3]);
    assert_eq!(field_def_d.ident.name, "D");
    assert_eq!(
        result
            .context
            .integer_value(&expect_matches!(&field_def_d.kind, DefKind::Const).value),
        1 << 11
    );
    assert!(!field_def_d.flags.contains(DefFlags::IS_ENUMERATED));
}

#[test]
fn test_bitmask_out_of_bounds() {
    let input = r"
        @bit_bound(8)
        bitmask TestBitMask {
            A = 7,
            B = 8,
            @value(9)
            C
        };
    ";

    insta::assert_snapshot!(common::parse_and_expect_errors(input));
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
