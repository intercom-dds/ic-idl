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
fn valid_inheritance() {
    let source = r"
interface Base {};
interface Derived : Base {};
interface MultiDerived : Base, Derived {};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn redundant_interface_inheritance() {
    let source = r"
interface Base {};
interface Derived : Base, Base {};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn redundant_qualified_inheritance() {
    let source = r"
module M {
    interface Base {};
};

interface Derived : M::Base, M::Base {};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn redundant_valuetype_inheritance() {
    let source = r"
interface Bar {};
valuetype Base {};
valuetype Derived : Base supports Bar {};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn redundant_via_different_paths() {
    let source = r"
interface A {};
interface Derived : A, ::A {};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn shadowed_name_is_not_redundant() {
    let source = r"
interface A {};
module M {
    interface A {};
    interface Derived : A, ::A {};
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn multiple_redundant_parents() {
    let source = r"
interface A {};
interface B {};
interface C : A, B, A, B {};
";

    assert_snapshot!(test_lint_hir(source));
}
