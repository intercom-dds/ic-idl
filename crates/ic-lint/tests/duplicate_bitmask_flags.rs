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
fn valid_bitmask_flags() {
    let source = r"
bitmask Permissions {
    READ,
    WRITE,
    EXECUTE,
    DELETE
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_flag_names() {
    let source = r"
bitmask FileFlags {
    READONLY,
    HIDDEN,
    SYSTEM,
    READONLY  // Duplicate flag
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_duplicates() {
    let source = r"
bitmask Options {
    ENABLE_A,
    ENABLE_B,
    ENABLE_A,  // First duplicate
    ENABLE_C,
    ENABLE_B,  // Second duplicate
    ENABLE_A   // Third occurrence
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_with_explicit_values() {
    let source = r"
bitmask Status {
    ACTIVE = 0,
    PENDING = 1,
    ACTIVE = 2,  // Duplicate name, different value
    COMPLETED = 3
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn same_value_different_names() {
    let source = r"
bitmask Aliases {
    FLAG_A = 1,
    FLAG_B = 2,
    FLAG_A_ALIAS = 1,  // Same value as FLAG_A, allowed
    FLAG_C = 4
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn empty_bitmask() {
    let source = r"
bitmask EmptyFlags {
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn single_flag_bitmask() {
    let source = r"
bitmask SingleFlag {
    ONLY_FLAG
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn case_sensitive_duplicates() {
    let source = r"
bitmask CaseSensitive {
    flag,
    FLAG,     // Different case, not a duplicate
    Flag,     // Another different case
    flag      // Duplicate of first one
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_bitmasks() {
    let source = r"
bitmask FirstFlags {
    FLAG_A,
    FLAG_B,
    FLAG_A    // Duplicate in first bitmask
};

bitmask SecondFlags {
    FLAG_A,   // Same name as in FirstFlags, but different bitmask
    FLAG_B,
    FLAG_C
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn nested_module_bitmasks() {
    let source = r"
module Outer {
    bitmask OuterFlags {
        FLAG,
        FLAG    // Duplicate
    };
    
    module Inner {
        bitmask InnerFlags {
            FLAG,    // Same name as outer, but different scope
            OTHER
        };
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn all_duplicates() {
    let source = r"
bitmask AllDuplicates {
    SAME,
    SAME,
    SAME,
    SAME
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn hex_value_duplicates() {
    let source = r"
bitmask HexFlags {
    FLAG_A = 0x01,
    FLAG_B = 0x02,
    FLAG_A = 0x04,  // Duplicate name
    FLAG_C = 0x08
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn octal_value_duplicates() {
    let source = r"
bitmask OctalFlags {
    FLAG_1 = 01,
    FLAG_2 = 02,
    FLAG_1 = 04,  // Duplicate name
    FLAG_3 = 010
};
";

    assert_snapshot!(test_lint_hir(source));
}
