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
use common::{lint_hir, test_lint_hir};

#[test]
fn valid_enum_constant_by_name() {
    let source = r"
enum Status {
    PENDING = 0,
    ACTIVE = 1,
    COMPLETED = 2
};

const Status CURRENT_STATUS = ACTIVE;  // Valid: using enum member
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn invalid_enum_constant_value() {
    let source = r"
enum Color {
    RED = 0,
    GREEN = 1,
    BLUE = 2
};

const Color MY_COLOR = 3;  // Invalid: 3 is not a valid Color value
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn invalid_enum_constant_negative() {
    let source = r"
enum Priority {
    LOW = 1,
    MEDIUM = 2,
    HIGH = 3
};

const Priority MY_PRIORITY = -1;  // Invalid: -1 is not a valid Priority value
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_invalid_enum_constants() {
    let source = r"
enum Status {
    IDLE = 0,
    RUNNING = 1,
    STOPPED = 2
};

const Status STATUS1 = 5;   // Invalid
const Status STATUS2 = 10;  // Invalid
const Status STATUS3 = 1;   // Valid: RUNNING
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_with_gaps_invalid() {
    let source = r"
enum ErrorCode {
    NONE = 0,
    NOT_FOUND = 404,
    SERVER_ERROR = 500
};

const ErrorCode CODE1 = 200;  // Invalid: 200 is not defined
const ErrorCode CODE2 = 403;  // Invalid: 403 is not defined
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_with_hex_values() {
    let source = r"
enum Flags {
    NONE = 0x00,
    READ = 0x01,
    WRITE = 0x02,
    EXECUTE = 0x04
};

const Flags FLAG1 = 0x01;  // Valid: READ
const Flags FLAG2 = 0x03;  // Invalid: 0x03 is not a defined flag
const Flags FLAG3 = 0x04;  // Valid: EXECUTE
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn non_enum_constants_not_checked() {
    let source = r#"
const long VALUE1 = 100;
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
        "Expected no lint warnings, but got: {output}"
    );
}

#[test]
fn enum_constant_in_different_scopes() {
    let source = r"
module A {
    enum Status {
        OK = 0,
        ERROR = 1
    };
    
    const Status S1 = 0;  // Valid: OK
    const Status S2 = 2;  // Invalid: 2 is not defined
};

module B {
    enum Status {
        PENDING = 0,
        DONE = 1
    };
    
    const Status S1 = 0;  // Valid: PENDING  
    const Status S2 = 2;  // Invalid: 2 is not defined
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn constant_references_not_checked() {
    let source = r"
enum Mode {
    OFF = 0,
    ON = 1,
    AUTO = 2
};

const Mode DEFAULT_MODE = OFF;
const Mode CURRENT_MODE = DEFAULT_MODE;  // References another constant, should not be checked
";

    let report = lint_hir(source);
    // The lint only checks direct integer values, not constant references
    assert!(
        report.errors.is_empty(),
        "Expected no errors for constant references"
    );
}

#[test]
fn large_enum_values() {
    let source = r"
enum LargeEnum {
    SMALL = 100,
    MEDIUM = 10000,
    LARGE = 1000000
};

const LargeEnum VAL1 = 100;      // Valid: SMALL
const LargeEnum VAL2 = 10001;    // Invalid
const LargeEnum VAL3 = 1000000;  // Valid: LARGE
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn enum_with_octal_values() {
    let source = r"
enum Permissions {
    USER_READ = 0400,
    USER_WRITE = 0200,
    USER_EXEC = 0100,
    GROUP_ALL = 070
};

const Permissions PERM1 = 0400;  // Valid: USER_READ (256 in decimal)
const Permissions PERM2 = 0300;  // Invalid: 192 in decimal is not defined
const Permissions PERM3 = 070;   // Valid: GROUP_ALL (56 in decimal)
";

    assert_snapshot!(test_lint_hir(source));
}
