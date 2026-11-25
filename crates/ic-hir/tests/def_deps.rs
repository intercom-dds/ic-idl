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

use std::collections::{BTreeSet, HashSet};

use ic_hir::ResolvedGraph;
use ic_hir::hir::{DefId, DefKind};

const STRUCT_MEMBER_DEFAULT: &str = r"
    const long DEFAULT_VALUE = 42;
    struct S {
        @default(DEFAULT_VALUE) long x;
    };
";

const STRUCT_ANNOTATION: &str = r"
    const long DEFAULT_VALUE = 42;
    @annotation limit {
        long value;
    };
    @limit(DEFAULT_VALUE)
    struct S {
        long x;
    };
";

const UNION_VARIANT_DEFAULT: &str = r"
    const long DEFAULT_VALUE = 42;
    union S switch (long) {
        case 1: @default(DEFAULT_VALUE) long x;
    };
";

const UNION_DISCRIMINATOR_DEFAULT: &str = r"
    const long DEFAULT_VALUE = 42;
    union S switch (@default(DEFAULT_VALUE) long) {
        case 1: long x;
    };
";

const STRUCT_MEMBER_RANGE: &str = r"
    const long LOW = 1;
    const long HIGH = 10;
    struct S {
        @range(min=LOW, max=HIGH) long x;
    };
";

const TYPEDEF_CUSTOM_ANNOTATION: &str = r"
    const long LOW = 1;
    const long HIGH = 10;
    @annotation bounds {
        long lower;
        long upper;
    };
    @bounds(lower=LOW, upper=HIGH)
    typedef long S;
";

const STRUCT_MEMBER_LITERAL_ANNOTATIONS: &str = r"
    @annotation bounds {
        long lower;
        long upper;
    };
    struct S {
        @default(5) @range(min=1, max=10) @bounds(lower=1, upper=10) long x;
    };
";

const EXCEPTION_MEMBER_DEFAULT: &str = r"
    const long DEFAULT_VALUE = 42;
    exception S {
        @default(DEFAULT_VALUE) long x;
    };
";

fn find_def(result: &ResolvedGraph, name: &str) -> DefId {
    result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == name)
        .map_or_else(|| panic!("definition '{name}' not found"), |(id, _)| id)
}

#[track_caller]
fn assert_ty_deps_contain(input: &str, consts: &[&str]) {
    let result = common::parse_and_resolve_successfully(input);
    let deps = result.context.ty_deps(find_def(&result, "S"));

    for name in consts {
        assert!(deps.contains(&find_def(&result, name)), "missing '{name}'");
    }
}

#[track_caller]
fn assert_deps_where_contain(input: &str, consts: &[&str]) {
    let result = common::parse_and_resolve_successfully(input);
    let deps = result.context.deps_where(find_def(&result, "S"), |_| true);

    for name in consts {
        assert!(deps.contains(&find_def(&result, name)), "missing '{name}'");
    }
}

fn names(result: &ResolvedGraph, deps: &HashSet<DefId>) -> BTreeSet<String> {
    deps.iter()
        .map(|id| result.context.type_of(*id).ident.name.clone())
        .collect()
}

#[track_caller]
fn assert_deps(input: &str, ty_deps: &[&str], deps: &[&str]) {
    let result = common::parse_and_resolve_successfully(input);
    let s = find_def(&result, "S");

    let expected = |names: &[&str]| names.iter().map(ToString::to_string).collect();
    assert_eq!(
        names(&result, &result.context.ty_deps(s)),
        expected(ty_deps),
        "ty_deps"
    );
    assert_eq!(
        names(&result, &result.context.deps_where(s, |_| true)),
        expected(deps),
        "deps_where"
    );
}

#[test]
fn primitive_members_have_no_deps() {
    assert_deps(
        r"
        struct S {
            long a;
            string b;
            sequence<double> c;
        };
        ",
        &[],
        &[],
    );
}

#[test]
fn struct_member_types() {
    assert_deps(
        r"
        struct A {};
        struct B {};
        struct C {};
        struct D {};
        struct E {};
        struct S {
            A a;
            sequence<B> b;
            C c[3];
            map<D, E> d;
        };
        ",
        &["A", "B", "C", "D", "E"],
        &["A", "B", "C", "D", "E"],
    );
}

#[test]
fn struct_parent_is_direct_ref() {
    assert_deps(
        r"
        struct Base {};
        struct M {};
        struct S : Base {
            M m;
        };
        ",
        &["M"],
        &["Base", "M"],
    );
}

#[test]
fn union_disc_variants_and_labels() {
    assert_deps(
        r"
        const long LABEL = 1;
        enum Kind { A, B };
        struct V {};
        typedef long Disc;
        union S switch (Disc) {
            case LABEL: V v;
            case 2: Kind k;
        };
        ",
        &["Disc", "Kind", "LABEL", "V"],
        &["Disc", "Kind", "LABEL", "V"],
    );
}

#[test]
fn interface_refs() {
    assert_deps(
        r"
        interface Base {};
        exception Err {};
        exception GetErr {};
        exception SetErr {};
        struct AttrTy {};
        struct RetTy {};
        struct ParamTy {};
        interface S : Base {
            attribute AttrTy attr getraises(GetErr) setraises(SetErr);
            RetTy op(in ParamTy p) raises(Err);
        };
        ",
        &["AttrTy", "ParamTy", "RetTy"],
        &[
            "AttrTy", "Base", "Err", "GetErr", "ParamTy", "RetTy", "SetErr",
        ],
    );
}

#[test]
fn valuetype_refs() {
    assert_deps(
        r"
        valuetype Base {};
        interface Iface {};
        struct M {};
        struct AttrTy {};
        valuetype S : Base supports Iface {
            public M m;
            attribute AttrTy attr;
        };
        ",
        &["AttrTy", "M"],
        &["AttrTy", "Base", "Iface", "M"],
    );
}

#[test]
fn exception_member_types() {
    assert_deps(
        r"
        struct M {};
        exception S {
            M m;
        };
        ",
        &["M"],
        &["M"],
    );
}

#[test]
fn typedef_target() {
    assert_deps(
        r"
        struct T {};
        typedef sequence<T> S;
        ",
        &["T"],
        &["T"],
    );
}

#[test]
fn const_type_and_value() {
    assert_deps(
        r"
        typedef long Num;
        const Num BASE = 1;
        const Num S = BASE;
        ",
        &["BASE", "Num"],
        &["BASE", "Num"],
    );
}

#[test]
fn bitset_parent_is_direct_ref() {
    assert_deps(
        r"
        bitset Base {
            bitfield<2> a;
        };
        bitset S : Base {
            bitfield<3> b;
        };
        ",
        &[],
        &["Base"],
    );
}

#[test]
fn annotation_param_type_and_default() {
    assert_deps(
        r"
        enum Kind { A, B };
        const long DEFAULT = 1;
        @annotation S {
            Kind kind;
            long value default DEFAULT;
        };
        ",
        &["DEFAULT", "Kind"],
        &["DEFAULT", "Kind"],
    );
}

#[test]
fn children_are_not_deps() {
    assert_deps(
        r"
        module S {
            struct Inner {};
        };
        ",
        &[],
        &[],
    );
    assert_deps("enum S { A, B };", &[], &[]);
    assert_deps("bitmask S { A, B };", &[], &[]);
}

#[test]
fn deps_where_applies_filter() {
    let result = common::parse_and_resolve_successfully(
        r"
        struct Base {};
        enum Kind { A };
        struct S : Base {
            Kind k;
        };
        ",
    );
    let s = find_def(&result, "S");

    let deps = result
        .context
        .deps_where(s, |def| !matches!(def.kind, DefKind::Struct(_)));
    assert_eq!(names(&result, &deps), BTreeSet::from(["Kind".to_string()]));
}

#[test]
fn ty_deps_struct_member_default() {
    assert_ty_deps_contain(STRUCT_MEMBER_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn deps_where_struct_member_default() {
    assert_deps_where_contain(STRUCT_MEMBER_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn ty_deps_struct_annotation() {
    assert_ty_deps_contain(STRUCT_ANNOTATION, &["DEFAULT_VALUE"]);
}

#[test]
fn deps_where_struct_annotation() {
    assert_deps_where_contain(STRUCT_ANNOTATION, &["DEFAULT_VALUE"]);
}

#[test]
fn ty_deps_union_variant_default() {
    assert_ty_deps_contain(UNION_VARIANT_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn deps_where_union_variant_default() {
    assert_deps_where_contain(UNION_VARIANT_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn ty_deps_union_discriminator_default() {
    assert_ty_deps_contain(UNION_DISCRIMINATOR_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn deps_where_union_discriminator_default() {
    assert_deps_where_contain(UNION_DISCRIMINATOR_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn ty_deps_exception_member_default() {
    assert_ty_deps_contain(EXCEPTION_MEMBER_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn deps_where_exception_member_default() {
    assert_deps_where_contain(EXCEPTION_MEMBER_DEFAULT, &["DEFAULT_VALUE"]);
}

#[test]
fn ty_deps_struct_member_range() {
    assert_ty_deps_contain(STRUCT_MEMBER_RANGE, &["LOW", "HIGH"]);
}

#[test]
fn deps_where_struct_member_range() {
    assert_deps_where_contain(STRUCT_MEMBER_RANGE, &["LOW", "HIGH"]);
}

#[test]
fn ty_deps_typedef_custom_annotation() {
    assert_ty_deps_contain(TYPEDEF_CUSTOM_ANNOTATION, &["LOW", "HIGH"]);
}

#[test]
fn deps_where_typedef_custom_annotation() {
    assert_deps_where_contain(TYPEDEF_CUSTOM_ANNOTATION, &["LOW", "HIGH"]);
}

#[test]
fn literal_annotation_args_add_no_deps() {
    let result = common::parse_and_resolve_successfully(STRUCT_MEMBER_LITERAL_ANNOTATIONS);
    let s = find_def(&result, "S");

    assert!(result.context.ty_deps(s).is_empty());
    assert!(result.context.deps_where(s, |_| true).is_empty());
}

#[test]
fn deps_where_excludes_filtered_annotation_const() {
    let result = common::parse_and_resolve_successfully(STRUCT_MEMBER_DEFAULT);
    let s = find_def(&result, "S");
    let value = find_def(&result, "DEFAULT_VALUE");

    let deps = result
        .context
        .deps_where(s, |def| !matches!(def.kind, DefKind::Const(_)));
    assert!(!deps.contains(&value));
}

#[test]
fn builtin_annotation_enum_arg_is_not_a_dep() {
    assert_deps(
        r"
        @autoid(HASH)
        struct S {
            @try_construct(TRIM) string<5> x;
        };
        ",
        &[],
        &[],
    );
}
