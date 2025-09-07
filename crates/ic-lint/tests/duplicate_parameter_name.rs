// Copyright 2025 KONGSBERG

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:

// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.

// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.

// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.

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
use common::test_lint_hir;

#[test]
fn test_duplicate_parameter_names() {
    let input = r"
        interface TestInterface {
            void methodA(in long x, in long x);
            void methodB(in string name, out string result, in string name);
            void methodC(in long param1, out long param2, inout long param1);
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_case_insensitive_parameter_names() {
    let input = r"
        interface TestInterface {
            void method1(in long Param, in long param);
            void method2(in string NAME, out string name);
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_unique_parameter_names() {
    let input = r"
        interface TestInterface {
            void method1(in long x, in long y, in long z);
            void method2(in string input, out string output, inout long count);
        };
    ";

    let result = test_lint_hir(input);
    assert!(result.is_empty(), "Expected no duplicate parameter errors");
}

#[test]
fn test_duplicate_parameters_across_interfaces() {
    let input = r"
        interface InterfaceA {
            void method(in long x, in long x);
        };
        
        interface InterfaceB {
            void method(in long x, in long x);
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_duplicate_parameters_with_different_types() {
    let input = r"
        interface TestInterface {
            void method(in long value, in string value);
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_no_parameters() {
    let input = r"
        interface TestInterface {
            void methodA();
            long methodB();
        };
    ";

    let result = test_lint_hir(input);
    assert!(
        result.is_empty(),
        "Expected no errors for methods without parameters"
    );
}
