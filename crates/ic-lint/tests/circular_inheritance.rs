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

use common::lint_hir;

#[test]
fn valid_inheritance_chain() {
    let report = lint_hir(
        r"
interface A {};
interface B : A {};
interface C : B {};
interface D : C, A {};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
fn self_inheritance() {
    let report = lint_hir(
        r"
interface A : A {};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("circular inheritance"));
}

#[test]
fn circular_interface_inheritance() {
    let report = lint_hir(
        r"
interface A : B {};
interface B : A {};
",
    );

    assert!(!report.errors.is_empty()); // May report for both A and B
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("circular inheritance"));
}

#[test]
fn circular_struct_inheritance() {
    let report = lint_hir(
        r"
struct A : B {};
struct B : C {};
struct C : A {};
",
    );

    assert!(!report.errors.is_empty());
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("circular inheritance"));
}

#[test]
fn complex_circular_inheritance() {
    let report = lint_hir(
        r"
interface A : B, C {};
interface B : D {};
interface C {};
interface D : A {};
",
    );

    assert!(!report.errors.is_empty());
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("circular inheritance"));
}

#[test]
fn valuetype_circular_inheritance() {
    let report = lint_hir(
        r"
valuetype A : B {};
valuetype B : A {};
",
    );

    assert!(!report.errors.is_empty());
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("circular inheritance"));
}
