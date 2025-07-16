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

use common::test_lint_hir;
use insta::assert_snapshot;

#[test]
#[ignore = "Annotation lowering not implemented"]
fn valid_range_bounds() {
    let source = r"
struct Foo {
    @range(0, 255)
    octet field1;
    
    @min(-128) @max(127)
    int8 field2;
    
    @range(-32768, 32767)
    short field3;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn invalid_min_bound() {
    let source = r"
struct Foo {
    @min(-200)
    int8 field;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn invalid_max_bound() {
    let source = r"
struct Foo {
    @max(300)
    octet field;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn invalid_range_order() {
    let source = r"
struct Foo {
    @range(100, 50)
    octet field;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn range_exceeds_type_bounds() {
    let source = r"
struct Foo {
    @range(-1000, 1000)
    int8 field;
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn const_with_range() {
    let source = r"
@range(0, 100)
const short MAX_VALUE = 50;

@min(200)
const octet MIN_VALUE = 250;
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn typedef_with_range() {
    let source = r"
@range(0, 1000)
typedef short SmallInt;

@range(-10000, 10000)
typedef int8 TinyInt; // This should fail
";
    assert_snapshot!(test_lint_hir(source));
}
