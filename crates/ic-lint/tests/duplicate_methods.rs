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

use insta::assert_snapshot;

mod common;
use common::{lint_hir, test_lint_hir};

#[test]
fn test_duplicate_method_direct() {
    let source = r"
interface Base {
    void method();
};

interface Derived : Base {
    void method();  // Error: conflicts with inherited method
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_duplicate_method_conflict_types() {
    let source = r"
interface Base {
    void method();
};

interface Derived : Base {
    long method();  // Error: conflicting return type
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_duplicate_method_conflict_params() {
    let source = r"
interface Base {
    void method(in long x);
};

interface Derived : Base {
    void method(in string x);  // Error: conflicting parameter type
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_duplicate_method_multiple_inheritance() {
    let source = r"
interface A {
    void method();
};

interface B {
    long method();  // Different return type
};

interface C : A, B {  // Error: inherits conflicting methods
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_duplicate_method_indirect_inheritance() {
    let source = r"
interface Base {
    void method();
};

interface Middle : Base {
};

interface Derived : Middle {
    long method();  // Error: conflicts with method from Base
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_duplicate_method_diamond_inheritance() {
    let source = r"
interface Base {
    void method();
};

interface A : Base {
};

interface B : Base {
};

interface C : A, B {  // Should not error - same method from Base
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_no_duplicate_methods() {
    let source = r"
interface Base {
    void method1();
    void method2();
};

interface Derived : Base {
    void method3();
    void method4();
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_duplicate_method_nested_interfaces() {
    let source = r"
interface Outer {
    interface Inner {
        void method();
    };
    
    interface InnerDerived : Inner {
        long method();  // Error: conflicts with Inner::method
    };
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_duplicate_method_same_interface() {
    let source = r"
interface Test {
    void method();
    long method();  // Should be caught by validate.rs, not this lint
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_duplicate_method_forward_decl() {
    let source = r"
interface Base;
interface Derived;

interface Base {
    void method();
};

interface Derived : Base {
    long method();  // Error: conflicts with inherited method
};
";

    assert_snapshot!(test_lint_hir(source));
}
