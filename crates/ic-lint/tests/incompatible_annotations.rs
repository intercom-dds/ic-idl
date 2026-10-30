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
fn incompatible_struct_member_annotations() {
    let source = r"
struct Data {
    @optional @external long a;
    @shared @optional long b;
    @optional @default(0) long c;
    @non_serialized @key long d;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn incompatible_union_annotations() {
    let source = r"
union Data switch (@key @non_serialized long) {
    case 0: @external @optional long a;
    case 1: @optional @shared long b;
    default: @default(0) @optional long c;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn nested_topic_types() {
    let source = r"
module Types {
    @nested @topic
    struct Data {
        long value;
    };

    @topic @nested
    union Choice switch (long) {
        default: long value;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn incompatible_annotations_on_other_member_targets() {
    let source = r"
@optional @external
typedef long ExternalAlias;

@shared @optional
typedef long SharedAlias;

@optional @default(0)
typedef long DefaultAlias;

exception Failure {
    @optional @shared long code;
};

valuetype Value {
    @optional @external public long data;
};

@annotation Settings {
    @optional @external long value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn disabled_annotations_are_compatible() {
    let source = r"
@nested(FALSE) @topic
struct Data {
    @optional(FALSE) @external long a;
    @optional @external(FALSE) long b;
    @optional(FALSE) @shared long c;
    @optional @shared(FALSE) long d;
    @optional(FALSE) @default(0) long e;
    @non_serialized(FALSE) @key long f;
    @non_serialized @key(FALSE) long g;
};

@topic @nested(FALSE)
union Choice switch (@non_serialized(FALSE) @key long) {
    case 0: @optional(FALSE) @external long a;
    case 1: @optional @external(FALSE) long b;
    case 2: @optional(FALSE) @shared long c;
    case 3: @optional @shared(FALSE) long d;
    default: @optional(FALSE) @default(0) long e;
};

@optional(FALSE) @external
typedef long Alias;
";

    assert!(
        lint_hir(source).errors.is_empty(),
        "{}",
        test_lint_hir(source)
    );
}

#[test]
fn resolves_scoped_annotations_and_boolean_expressions() {
    let source = r"
const boolean ENABLED = TRUE;
module Types {
    struct Data {
        @::optional(value = ENABLED)
        @::external(value = ENABLED)
        long value;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn user_defined_annotations_are_not_builtin_conflicts() {
    let source = r"
module Custom {
    @annotation optional {};
    @annotation external {};
    @annotation shared {};
    @annotation default { long value; };
    @annotation nested {};
    @annotation topic {};
    @annotation non_serialized {};
    @annotation key {};

    @nested @topic
    struct Data {
        @optional @external long a;
        @optional @shared long b;
        @optional @default(0) long c;
        @non_serialized @key long d;
        @::optional @external long e;
        @optional @::external long f;
    };
};
";

    assert!(
        lint_hir(source).errors.is_empty(),
        "{}",
        test_lint_hir(source)
    );
}

#[test]
fn compatible_annotations_remain_valid() {
    let source = r"
@topic
struct Data {
    @key @external long id;
    @optional @must_understand long value;
    @non_serialized @default(0) long local;
};
";

    assert!(
        lint_hir(source).errors.is_empty(),
        "{}",
        test_lint_hir(source)
    );
}

#[test]
fn lint_is_a_semantic_error() {
    let lint = ic_lint::all_lints()
        .into_iter()
        .find(|lint| lint.name == "incompatible-annotations")
        .unwrap();
    assert_eq!(lint.category, ic_lint::Category::Semantic);

    let report = lint_hir("struct Data { @optional @external long value; };");
    assert_eq!(report.errors.len(), 1);
    assert!(report.warnings.is_empty());
}
