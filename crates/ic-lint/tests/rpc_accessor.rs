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

mod common;

use common::{lint_hir, test_lint_hir};
use insta::assert_snapshot;

#[test]
fn accessor_collision() {
    let source = r"
@DDSService
interface S {
    attribute long value;
    long get_attribute_value();
    void SET_ATTRIBUTE_VALUE(in long value);
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn readonly_setter_no_collision() {
    let source = r"
@DDSService
interface S {
    readonly attribute long value;
    void set_attribute_value(in long value);
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn readonly_getter_collision() {
    let source = r"
@DDSService
interface S {
    readonly attribute long value;
    long get_attribute_value();
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_attributes() {
    let source = r"
@DDSService
interface S {
    attribute long first, second;
    void set_attribute_second(in long second);
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn plain_interface_no_collision() {
    let source = r"
interface S {
    attribute long value;
    long get_attribute_value();
    void set_attribute_value(in long value);
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn unrelated_names() {
    let source = r"
@DDSService
interface S {
    attribute long value;
    long get_value();
    void set_attribute_other(in long other);
    long get_attribute_value_2();
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn inherited_attribute_collision() {
    let source = r"
@DDSService
interface Base {
    attribute long value;
};

@DDSService
interface Derived : Base {
    long get_attribute_value();
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn inherited_operation_collision() {
    let source = r"
@DDSService
interface Base {
    void set_attribute_value(in long value);
};

@DDSService
interface Derived : Base {
    attribute long value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn sibling_bases_collision() {
    let source = r"
@DDSService
interface Left {
    attribute long value;
};

@DDSService
interface Right {
    long get_attribute_value();
};

@DDSService
interface Derived : Left, Right {};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn diamond_reported_once() {
    let source = r"
@DDSService
interface Base {
    attribute long value;
    long get_attribute_value();
};

@DDSService
interface Left : Base {};

@DDSService
interface Right : Base {};

@DDSService
interface Derived : Left, Right {};
";

    assert_snapshot!(test_lint_hir(source));
}
