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
fn valid_enum_default_type() {
    let source = r"
enum Color {
    RED = 0,
    GREEN = 1,
    BLUE = 2
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no lint warnings, but got: {}", output);
}

#[test]
fn valid_enum_implicit_values() {
    let source = r"
enum Status {
    PENDING,    // 0
    ACTIVE,     // 1
    COMPLETED   // 2
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no lint warnings, but got: {}", output);
}

#[test]
fn duplicate_explicit_values() {
    let source = r"
enum Priority {
    LOW = 1,
    MEDIUM = 2,
    HIGH = 1    // Duplicate value
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_implicit_values() {
    let source = r"
enum Mixed {
    A,          // 0
    B = 0,      // Explicit 0, duplicate with A
    C,          // 1
    D = 2,      // 2
    E           // 3
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn out_of_range_values() {
    let source = r"
enum LargeValues {
    SMALL = 1,
    MEDIUM = 100000,
    LARGE = 4294967296  // Out of range for 32-bit
};
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
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn hex_and_octal_values() {
    let source = r"
enum Flags {
    NONE = 0x0,
    READ = 0x1,
    WRITE = 0x2,
    EXECUTE = 0x4,
    ALL = 0x7
};

enum Permissions {
    USER_READ = 0400,
    USER_WRITE = 0200,
    USER_EXEC = 0100,
    GROUP_READ = 040,
    GROUP_WRITE = 020,
    GROUP_EXEC = 010
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no lint warnings, but got: {}", output);
}

#[test]
fn enum_value_gaps() {
    let source = r"
enum Sparse {
    FIRST = 1,
    SECOND = 10,
    THIRD = 100,
    FOURTH = 1000
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no lint warnings, but got: {}", output);
}

#[test]
fn duplicate_names_different_scopes() {
    let source = r"
module A {
    enum Status {
        OK = 0,
        ERROR = 1
    };
};

module B {
    enum Status {
        OK = 0,      // Different enum, should be fine
        ERROR = 1
    };
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no lint warnings, but got: {}", output);
}
