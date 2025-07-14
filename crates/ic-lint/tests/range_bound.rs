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

use common::lint_hir;

#[test]
#[ignore] // Ignore until annotation lowering is working
fn valid_range_bounds() {
    let report = lint_hir(
        r#"
struct Foo {
    @range(0, 255)
    octet field1;
    
    @min(-128) @max(127)
    int8 field2;
    
    @range(-32768, 32767)
    short field3;
};
"#,
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn invalid_min_bound() {
    let report = lint_hir(
        r#"
struct Foo {
    @min(-200)
    int8 field;
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("less than type minimum"));
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn invalid_max_bound() {
    let report = lint_hir(
        r#"
struct Foo {
    @max(300)
    octet field;
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("greater than type maximum"));
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn invalid_range_order() {
    let report = lint_hir(
        r#"
struct Foo {
    @range(100, 50)
    octet field;
};
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("minimum 100 is greater than maximum 50"));
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn range_exceeds_type_bounds() {
    let report = lint_hir(
        r#"
struct Foo {
    @range(-1000, 1000)
    int8 field;
};
"#,
    );

    assert_eq!(report.errors.len(), 2); // Both min and max are out of bounds
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn const_with_range() {
    let report = lint_hir(
        r#"
@range(0, 100)
const short MAX_VALUE = 50;

@min(200)
const octet MIN_VALUE = 250;
"#,
    );

    assert_eq!(report.errors.len(), 0); // First is valid, second would overflow but that's a different check
}

#[test]
#[ignore] // Ignore until annotation lowering is working
fn typedef_with_range() {
    let report = lint_hir(
        r#"
@range(0, 1000)
typedef short SmallInt;

@range(-10000, 10000)
typedef int8 TinyInt; // This should fail
"#,
    );

    assert_eq!(report.errors.len(), 2); // Both min and max out of bounds for int8
}
