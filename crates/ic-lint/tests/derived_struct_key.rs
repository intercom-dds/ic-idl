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
fn test_derived_struct_with_key() {
    let source = r"
struct Base {
    @key long id;
    string name;
};

struct Derived : Base {
    @key string code;  // Error: derived struct cannot have @key
    long value;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_base_struct_with_key() {
    let source = r"
struct Base {
    @key long id;
    string name;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_derived_struct_without_key() {
    let source = r"
struct Base {
    @key long id;
    string name;
};

struct Derived : Base {
    string description;
    long value;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_multiple_derived_structs_with_keys() {
    let source = r"
struct Base {
    @key long id;
};

struct FirstDerived : Base {
    @key string first_key;
};

struct SecondDerived : Base {
    @key long second_key;
    @key string third_key;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_nested_inheritance_with_key() {
    let source = r"
struct GrandParent {
    @key long id;
};

struct Parent : GrandParent {
    string name;
};

struct Child : Parent {
    @key string child_key;  // Error: still a derived struct
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_derived_struct_with_optional_key() {
    let source = r"
struct Base {
    long id;
};

struct Derived : Base {
    @optional @key string code;  // Multiple issues: @key in derived struct
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_multiple_inheritance_chain() {
    let source = r"
struct A {
    @key long a_id;
};

struct B : A {
    string b_name;
};

struct C : B {
    @key string c_key;  // Error
};

struct D : C {
    @key long d_key;    // Error
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_struct_without_inheritance() {
    let source = r"
struct Independent {
    @key long id;
    @key string code;
    string name;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}
