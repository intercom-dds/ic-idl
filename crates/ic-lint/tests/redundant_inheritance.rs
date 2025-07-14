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
fn valid_inheritance() {
    let output = test_lint(
        r#"
interface Base {};
interface Derived : Base {};
interface MultiDerived : Base, Derived {};
"#,
    );

    assert!(output.is_empty());
}

#[test]
fn redundant_interface_inheritance() {
    let output = test_lint(
        r#"
interface Base {};
interface Derived : Base, Base {};
"#,
    );

    assert!(output.contains("inherits from 'Base' multiple times"));
    assert!(output.contains("redundant inheritance"));
}

#[test]
fn redundant_qualified_inheritance() {
    let output = test_lint(
        r#"
module M {
    interface Base {};
};

interface Derived : M::Base, M::Base {};
"#,
    );

    assert!(output.contains("inherits from 'M::Base' multiple times"));
}

#[test]
fn redundant_valuetype_inheritance() {
    let output = test_lint(
        r#"
valuetype Base {};
valuetype Derived : Base supports Base {};
"#,
    );

    assert!(output.contains("inherits from 'Base' multiple times"));
}

#[test]
fn multiple_redundant_parents() {
    let output = test_lint(
        r#"
interface A {};
interface B {};
interface C : A, B, A, B {};
"#,
    );

    // Should report both A and B as redundant
    assert!(output.contains("inherits from 'A' multiple times"));
    assert!(output.contains("inherits from 'B' multiple times"));
}
