// Copyright 2025 KONGSBERG
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
fn test_any_in_struct_member() {
    let source = r"
        struct Message {
            any payload;
            string id;
        };
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_sequence() {
    let source = r"
        typedef sequence<any> AnyList;
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_array() {
    let source = r"
        typedef any AnyArray[10];
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_union_variant() {
    let source = r"
        union Data switch (long) {
        case 0:
            string text;
        case 1:
            any value;
        default:
            double number;
        };
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_interface_operation() {
    let source = r"
        interface Service {
            any getData();
            void setData(in any value);
        };
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_valuetype() {
    let source = r"
        valuetype Holder {
            public any data;
            private string name;
        };
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_map() {
    let source = r"
        typedef map<string, any> Properties;
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_any_in_alias() {
    let source = r"
        typedef any AnyType;
        
        struct Container {
            AnyType item;
        };
    ";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_no_any_type() {
    // This test should not produce any warnings
    let source = r"
        struct Point {
            double x;
            double y;
        };
        
        typedef sequence<Point> PointList;
    ";

    let report = lint_hir(source);
    assert!(
        report.errors.is_empty(),
        "Expected no errors, but got: {:?}",
        report.errors
    );
    assert!(
        report.warnings.is_empty(),
        "Expected no warnings, but got: {:?}",
        report.warnings
    );
}
