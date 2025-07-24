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
use common::test_lint;

#[test]
fn test_bitwise_precedence() {
    let idl = r"
// Different bitwise operators with confusing precedence
const long test1 = 1 | 2 & 3;      // & binds tighter than |
const long test2 = 1 | 2 ^ 3;      // ^ binds tighter than |  
const long test3 = 1 ^ 2 & 3;      // & binds tighter than ^
const long test4 = 1 | 2 ^ 3 & 4;  // Multiple levels

// With parentheses (still warns - could be improved in future)
const long test5 = 1 | (2 & 3);    
const long test6 = (1 | 2) & 3;    

// Same precedence - no warnings
const long test7 = 1 | 2 | 3;      
const long test8 = 1 & 2 & 3;      
const long test9 = 1 ^ 2 ^ 3;      
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_arithmetic_bitwise_mix() {
    let idl = r"
// Mixing arithmetic and bitwise operators
const long test1 = 1 & 2 + 3;      // + binds tighter than &
const long test2 = 1 | 2 * 3;      // * binds tighter than |
const long test3 = 1 ^ 2 - 3;      // - binds tighter than ^
const long test4 = 1 + 2 & 3;      // + binds tighter than &
const long test5 = 1 * 2 | 3;      // * binds tighter than |

// Complex expressions
const long test6 = 1 | 2 + 3 * 4;  // Both + and * bind tighter than |
const long test7 = 1 & 2 * 3 + 4;  // Both * and + bind tighter than &
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_no_warnings() {
    let idl = r"
// Expected arithmetic precedence - no warnings
const long test1 = 1 + 2 * 3;      // Well-known: * before +
const long test2 = 1 - 2 / 3;      // Well-known: / before -
const long test3 = 1 + 2 - 3;      // Same precedence
const long test4 = 1 * 2 / 3;      // Same precedence
const long test5 = 1 * 2 % 3;      // Same precedence

// Complex arithmetic expressions - no warnings
const long test6 = 1 + 2 * 3 - 4 / 5;
const long test7 = (1 + 2) * (3 - 4) / 5;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_in_different_contexts() {
    let idl = r"
// In struct field initializers
struct Config {
    long flags;
};

// In arrays
const long array[1 | 2 & 3] = { 0 };  // Should warn in array size

// In enum values (if assignment expressions were supported)
enum Flags {
    FLAG_A,
    FLAG_B
};

// In typedef
typedef long Flags;

// In module
module Test {
    const long nested = 1 | 2 & 3;  // Should warn in nested context
    
    struct Inner {
        long value;
    };
};

// In unions
union Value switch (long) {
    case 1:
        long a;
    case 2:
        long b;
};

// Multiple issues on same line
const long complex = 1 | 2 & 3 ^ 4 + 5;  // Multiple warnings
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_real_world_examples() {
    let idl = r"
// Common bit flag patterns that might trigger warnings
const long READ_PERMISSION  = 1 << 0;
const long WRITE_PERMISSION = 1 << 1;
const long EXEC_PERMISSION  = 1 << 2;

// This pattern is common but potentially confusing
const long DEFAULT_PERMS = READ_PERMISSION | WRITE_PERMISSION & 0xFF;

// Bit masking operations
const long MASKED_VALUE = 0xF0F0 & 0x1234 + 1;  // Arithmetic happens first!

// Flag checking patterns
const long HAS_RW = READ_PERMISSION | WRITE_PERMISSION;
const long TEST_FLAGS = HAS_RW & READ_PERMISSION | EXEC_PERMISSION;
";

    assert_snapshot!(test_lint(idl));
}

#[test]
fn test_edge_cases() {
    let idl = r"
// Unary operators don't trigger the warning
const long test1 = ~1 & 2;
const long test2 = -1 + 2;
const long test3 = +1 * 2;

// Nested expressions
const long test4 = (1 | 2) & (3 ^ 4);  // Parentheses present but still warns
const long test5 = 1 | (2 & (3 ^ 4));  // Deeply nested

// With identifiers
const long MASK = 0xFF;
const long FLAGS = 0x0F;
const long test6 = MASK & FLAGS + 1;   // Should warn

// Empty file should not crash
";

    assert_snapshot!(test_lint(idl));
}
