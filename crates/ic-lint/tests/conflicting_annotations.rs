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
fn test_struct_optional_key_conflict() {
    let input = r"
        struct User {
            @key
            @optional
            long id;
            
            @optional @key string username;
            
            @key
            string email;
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_union_optional_key_conflict() {
    let input = r"
        union Command {
            @key @optional long: Execute();
            @optional string: Print();
            @key long: Save();
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_no_conflict_optional_only() {
    let input = r"
        struct Config {
            @optional string name;
            @optional long timeout;
        };
    ";

    let result = test_lint_hir(input);
    assert!(
        result.is_empty(),
        "Expected no conflicts for @optional only"
    );
}

#[test]
fn test_no_conflict_key_only() {
    let input = r"
        struct Entity {
            @key long id;
            @key string type;
            string data;
        };
    ";

    let result = test_lint_hir(input);
    assert!(result.is_empty(), "Expected no conflicts for @key only");
}

#[test]
fn test_mixed_annotations_no_conflict() {
    let input = r"
        struct Record {
            @key long id;
            @optional string description;
            @deprecated string old_field;
            string name;
        };
    ";

    let result = test_lint_hir(input);
    assert!(
        result.is_empty(),
        "Expected no conflicts when annotations don't overlap"
    );
}

#[test]
fn test_multiple_structs_with_conflicts() {
    let input = r"
        struct A {
            @key @optional long id;
        };
        
        struct B {
            @optional long x;
            @key @optional long y;
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}

#[test]
fn test_nested_struct_member() {
    let input = r"
        struct Inner {
            @key @optional long value;
        };
        
        struct Outer {
            Inner inner;
            @key @optional string name;
        };
    ";

    insta::assert_snapshot!(test_lint_hir(input));
}
