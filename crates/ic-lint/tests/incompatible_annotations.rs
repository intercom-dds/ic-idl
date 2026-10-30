// Copyright 2026 KONGSBERG

mod common;

use common::{lint_hir, test_lint_hir};

fn errors(source: &str) -> String {
    test_lint_hir(source)
        .lines()
        .filter(|line| line.starts_with('⨯'))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn incompatible_struct_member_annotations() {
    let input = r"
        struct Data {
            @optional @external long a;
            @shared @optional long b;
            @optional @default(0) long c;
            @non_serialized @key long d;
        };
    ";

    insta::assert_snapshot!(errors(input), @r"
    ⨯ incompatible-annotations: member `a` cannot be both @optional and @external
    ⨯ incompatible-annotations: member `b` cannot be both @optional and @shared
    ⨯ incompatible-annotations: member `c` cannot be both @optional and @default
    ⨯ incompatible-annotations: member `d` cannot be both @non_serialized and @key
    ");

    let output = test_lint_hir(input);
    assert!(output.contains("@optional annotation here"));
    assert!(output.contains("@external annotation here"));
    assert!(output.contains("incompatible annotations on member"));
    assert!(output.contains("remove either @optional or @external"));
}

#[test]
fn incompatible_union_annotations() {
    let input = r"
        union Data switch (@key @non_serialized long) {
            case 0: @external @optional long a;
            case 1: @optional @shared long b;
            default: @default(0) @optional long c;
        };
    ";

    insta::assert_snapshot!(errors(input), @r"
    ⨯ incompatible-annotations: union discriminator `Data` cannot be both @non_serialized and @key
    ⨯ incompatible-annotations: union variant `a` cannot be both @optional and @external
    ⨯ incompatible-annotations: union variant `b` cannot be both @optional and @shared
    ⨯ incompatible-annotations: union variant `c` cannot be both @optional and @default
    ");
}

#[test]
fn nested_topic_types() {
    let input = r"
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

    insta::assert_snapshot!(errors(input), @r"
    ⨯ incompatible-annotations: type `Data` cannot be both @nested and @topic
    ⨯ incompatible-annotations: type `Choice` cannot be both @nested and @topic
    ");
}

#[test]
fn incompatible_annotations_on_other_member_targets() {
    let input = r"
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

    insta::assert_snapshot!(errors(input), @r"
    ⨯ incompatible-annotations: typedef `ExternalAlias` cannot be both @optional and @external
    ⨯ incompatible-annotations: typedef `SharedAlias` cannot be both @optional and @shared
    ⨯ incompatible-annotations: typedef `DefaultAlias` cannot be both @optional and @default
    ⨯ incompatible-annotations: member `code` cannot be both @optional and @shared
    ⨯ incompatible-annotations: member `data` cannot be both @optional and @external
    ⨯ incompatible-annotations: annotation member `value` cannot be both @optional and @external
    ");
}

#[test]
fn disabled_annotations_are_compatible() {
    let input = r"
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

    assert!(lint_hir(input).errors.is_empty(), "{}", test_lint_hir(input));
}

#[test]
fn resolves_scoped_annotations_and_boolean_expressions() {
    let input = r"
        const boolean ENABLED = TRUE;
        module Types {
            struct Data {
                @::optional(value = ENABLED)
                @::external(value = ENABLED)
                long value;
            };
        };
    ";

    insta::assert_snapshot!(errors(input), @r"
    ⨯ incompatible-annotations: member `value` cannot be both @optional and @external
    ");
}

#[test]
fn user_defined_annotations_are_not_builtin_conflicts() {
    let input = r"
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

    assert!(lint_hir(input).errors.is_empty(), "{}", test_lint_hir(input));
}

#[test]
fn compatible_annotations_remain_valid() {
    let input = r"
        @topic
        struct Data {
            @key @external long id;
            @optional @must_understand long value;
            @non_serialized @default(0) long local;
        };
    ";

    assert!(lint_hir(input).errors.is_empty(), "{}", test_lint_hir(input));
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
