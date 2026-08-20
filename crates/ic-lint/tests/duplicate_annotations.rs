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
fn valid_annotations() {
    let source = r"
@annotation First {};
@annotation Second {};

@First
@Second
struct Foo {
    @optional
    @min(0)
    long field;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn duplicate_annotation_on_struct() {
    let source = r"
@empty
@empty
struct Foo {
    long field;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_annotation_on_field() {
    let source = r"
struct Foo {
    @min(0)
    @max(100)
    @min(10)
    long field;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_on_interface_method() {
    let source = r"
interface Service {
    @oneway
    @oneway
    void notify();
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_annotations_on_all_hir_targets() {
    let source = r"
@annotation Meta {
    @ext::suppress @ext::suppress long value;
};
union Choice switch (@ext::suppress @ext::suppress long) {
    case 0: @ext::suppress @ext::suppress long value;
};
bitset Bits {
    @ext::suppress @ext::suppress bitfield<1> value;
};
interface Service {
    @ext::suppress @ext::suppress attribute long value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_semantic_annotations() {
    let source = r#"
@final
@extensibility(MUTABLE)
struct Foo {
    @id(1)
    @hashid("value")
    long value;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn duplicate_user_annotation_is_valid() {
    let source = r"
@annotation MyAnn {
    long value;
};

@MyAnn(1)
@MyAnn(2)
struct Foo {
    long field;
};
";

    assert!(test_lint_hir(source).is_empty());
}

#[test]
fn repeatable_builtin_annotations_are_valid() {
    let source = r#"
@doc("first")
@doc("second")
@verbatim(text = "first")
@verbatim(text = "second")
@derive("First")
@derive("Second")
struct Foo {
    long field;
};
"#;

    assert!(test_lint_hir(source).is_empty());
}
