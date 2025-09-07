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

use insta::assert_snapshot;

mod common;
use common::test_lint_hir;

#[test]
fn prefer_enum_member_simple() {
    let source = r"
enum Status {
    PENDING = 0,
    ACTIVE = 1,
    COMPLETED = 2
};

const Status S1 = 0;  // Should warn: prefer PENDING
const Status S2 = 1;  // Should warn: prefer ACTIVE
const Status S3 = 2;  // Should warn: prefer COMPLETED
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn using_member_names_no_warning() {
    let source = r"
enum Color {
    RED = 0,
    GREEN = 1,
    BLUE = 2
};

const Color C1 = RED;       // Good: using member name
const Color C2 = GREEN;     // Good: using member name
const Color C3 = BLUE;      // Good: using member name
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings when using member names, but got: {output}"
    );
}

#[test]
fn hex_and_octal_values() {
    let source = r"
enum Flags {
    NONE = 0x00,
    READ = 0x01,
    WRITE = 0x02,
    EXECUTE = 0x04
};

const Flags F1 = 0x00;  // Should warn: prefer NONE
const Flags F2 = 0x01;  // Should warn: prefer READ
const Flags F3 = 1;     // Should warn: prefer READ (decimal form)
const Flags F4 = 0x04;  // Should warn: prefer EXECUTE
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_with_gaps() {
    let source = r"
enum HttpStatus {
    OK = 200,
    NOT_FOUND = 404,
    SERVER_ERROR = 500
};

const HttpStatus S1 = 200;  // Should warn: prefer OK
const HttpStatus S2 = 404;  // Should warn: prefer NOT_FOUND
const HttpStatus S3 = 500;  // Should warn: prefer SERVER_ERROR
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn negative_values() {
    let source = r"
enum Temperature {
    FREEZING = -273,
    COLD = -10,
    NORMAL = 20,
    HOT = 40
};

const Temperature T1 = -273;  // Should warn: prefer FREEZING
const Temperature T2 = -10;   // Should warn: prefer COLD
const Temperature T3 = 20;    // Should warn: prefer NORMAL
const Temperature T4 = 40;    // Should warn: prefer HOT
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn mixed_usage() {
    let source = r"
enum Priority {
    LOW = 1,
    MEDIUM = 2,
    HIGH = 3
};

const Priority P1 = LOW;     // Good: using member name
const Priority P2 = 2;       // Should warn: prefer MEDIUM
const Priority P3 = HIGH;    // Good: using member name
const Priority P4 = 1;       // Should warn: prefer LOW
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn constant_references_not_warned() {
    let source = r"
enum Mode {
    OFF = 0,
    ON = 1,
    AUTO = 2
};

const long ZERO = 0;
const Mode M1 = OFF;        // Good: using member name
const Mode M2 = ZERO;       // Not checked: constant reference
";

    // Should only have warnings for invalid_enum_literal, not prefer_enum_member_name
    let output = test_lint_hir(source);
    assert!(!output.contains("prefer_enum_member_name"));
}

#[test]
fn multiple_enums_different_values() {
    let source = r"
enum Status {
    OFF = 0,
    ON = 1
};

enum Level {
    LOW = 0,
    HIGH = 1
};

const Status S = 0;  // Should warn: prefer OFF
const Level L = 0;   // Should warn: prefer LOW
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_members_themselves_not_warned() {
    let source = r"
enum Color {
    RED = 0,      // This should not trigger the lint
    GREEN = 1,    // This should not trigger the lint
    BLUE = 2      // This should not trigger the lint
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for enum member definitions, but got: {output}"
    );
}

#[test]
fn non_enum_constants_not_warned() {
    let source = r#"
const long MY_VALUE = 42;
const string NAME = "test";
const float PI = 3.14;

struct Point {
    long x;
    long y;
};

const Point ORIGIN = { 0, 0 };
"#;

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for non-enum constants, but got: {output}"
    );
}
