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

mod common;

use common::{lint_hir, test_lint_hir};
use insta::assert_snapshot;

#[test]
fn valid_array_size() {
    let source = r"
struct Data {
    long small[10];        // 40 bytes
    long medium[1000];     // 4000 bytes
    octet large[16000];    // 16000 bytes (just under 16KB limit)
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn large_array_size() {
    let source = r"
struct LargeArrays {
    long huge[5000];  // 5000 × 4 bytes = 20000 bytes > 16KB
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_large_arrays() {
    let source = r"
struct MultipleArrays {
    long arr1[5000];      // 20000 bytes
    double arr2[3000];    // 24000 bytes
    boolean arr3[10000];  // 10000 bytes - OK
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn const_expression_array_size() {
    let source = r"
const long MILLION = 1000000;
const long TWO = 2;
struct ConstArrays {
    long large[MILLION * TWO];  // 2 million
    long small[MILLION / 10];   // 100k, should be fine
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn string_with_large_bound() {
    // String bounds are just constraints, not allocations, so we don't warn about them
    let source = r"
struct LargeString {
    string<20000> huge_str;  // 20000 chars × 1 byte = 20KB - OK, just a max length
    wstring<5000> wide_str;  // 5000 chars × 4 bytes = 20KB - OK, just a max length
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn typedef_array() {
    let source = r"
typedef long BigArray[5000];  // 20000 bytes
struct UsesBigArray {
    BigArray data;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multidimensional_array() {
    let source = r"
struct MultiDim {
    long matrix[2000][1000];  // 2 million total elements
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn different_type_sizes() {
    let source = r"
struct TypeSizes {
    octet octets[17000];          // 17000 bytes - too big
    short shorts[8500];           // 17000 bytes - too big  
    long longs[4200];             // 16800 bytes - too big
    unsigned long long llongs[2100]; // 16800 bytes - too big
    double doubles[2100];         // 16800 bytes - too big
    
    octet ok_octets[16000];       // 16000 bytes - OK
    short ok_shorts[8000];        // 16000 bytes - OK
    long ok_longs[4000];          // 16000 bytes - OK
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn sequence_and_map_with_large_bounds() {
    // Sequence and map bounds are just constraints, not allocations, so we don't warn about them
    let source = r"
struct LargeBounds {
    sequence<long, 100000> huge_seq;     // OK, just a max length
    map<string, long, 50000> huge_map;   // OK, just a max length
};
";
    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}
