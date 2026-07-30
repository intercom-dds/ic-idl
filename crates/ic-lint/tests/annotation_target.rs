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

use common::{test_lint, test_lint_hir};
use insta::assert_snapshot;

#[test]
fn accepts_declared_targets() {
    let source = r"
@ext::annotation_target(STRUCT_DEF | UNION_DEF)
@annotation aggregate_only {};

@aggregate_only
struct Foo {};

@aggregate_only
union Bar switch (long) {
    case 0: long value;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "expected no warnings, got:\n{output}");
}

#[test]
fn rejects_undeclared_targets() {
    let source = r"
@ext::annotation_target(STRUCT_DEF | UNION_DEF)
@annotation aggregate_only {};

@aggregate_only
module invalid_module {};

struct Foo {
    @aggregate_only long invalid_member;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn recognizes_shorthand_target_meta_annotation() {
    let source = r"
module ext {
    @annotation_target(STRUCT_DEF)
    @annotation struct_only {};
};

@ext::struct_only
module Invalid {};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn annotations_without_target_are_unrestricted() {
    let source = r"
@annotation unrestricted {};

@unrestricted
module Fine {
    @unrestricted
    struct AlsoFine {
        @unrestricted long value;
    };
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "expected no warnings, got:\n{output}");
}

#[test]
fn validates_preserved_member_kinds() {
    let source = r"
@ext::annotation_target(ANNOTATION_MEMBER | PROTOTYPE | ATTRIBUTE_DEF)
@annotation members_only {};

@annotation configured {
    @members_only long value;
};

interface Service {
    @members_only void call();
    @members_only attribute long value;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "expected no warnings, got:\n{output}");
}

#[test]
fn rejects_annotations_on_prototype_parameters() {
    let source = r"
interface Service {
    void call(@key in long value);
};
";

    assert_snapshot!(test_lint(source));
}
