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

use common::test_lint;

#[test]
fn valid_annotations() {
    let output = test_lint(
        r#"
@id(1)
@version("1.0")
struct Foo {
    @optional
    @min(0)
    long field;
};
"#,
    );

    assert!(output.is_empty());
}

#[test]
fn duplicate_annotation_on_struct() {
    let output = test_lint(
        r"
@id(1)
@id(2)
struct Foo {
    long field;
};
",
    );

    assert!(output.contains("duplicate annotation '@id'"));
}

#[test]
fn duplicate_annotation_on_field() {
    let output = test_lint(
        r"
struct Foo {
    @min(0)
    @max(100)
    @min(10)
    long field;
};
",
    );

    assert!(output.contains("duplicate annotation '@min'"));
}

#[test]
fn conflicting_optional_required() {
    let output = test_lint(
        r"
struct Foo {
    @optional
    @required
    long field;
};
",
    );

    assert!(output.contains("conflicting annotations"));
    assert!(output.contains("@optional") && output.contains("@required"));
}

#[test]
fn conflicting_readonly_readwrite() {
    let output = test_lint(
        r"
interface Service {
    @readonly
    @readwrite
    attribute long value;
};
",
    );

    assert!(output.contains("conflicting annotations"));
    assert!(output.contains("@readonly") && output.contains("@readwrite"));
}

#[test]
fn duplicate_on_interface_method() {
    let output = test_lint(
        r"
interface Service {
    @oneway
    @oneway
    void notify();
};
",
    );

    assert!(output.contains("duplicate annotation '@oneway'"));
}

#[test]
fn duplicate_qualified_annotation() {
    let output = test_lint(
        r"
annotation MyAnn {
    long value;
};

@MyAnn(1)
@MyAnn(2)
struct Foo {
    long field;
};
",
    );

    assert!(output.contains("duplicate annotation '@MyAnn'"));
}
