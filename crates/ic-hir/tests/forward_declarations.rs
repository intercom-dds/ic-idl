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

#[test]
fn test_forward_declaration_followed_by_definition() {
    // Test that a forward declaration followed by a full definition works correctly
    let input = r"
        struct A;
        struct A {
            long field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);

    // Should have one struct definition (not two)
    let struct_count = hir
        .order
        .iter()
        .filter(|&&id| {
            matches!(
                &hir.context.definitions.get(id).kind,
                ic_hir::hir::DefKind::Struct(_)
            )
        })
        .count();
    assert_eq!(struct_count, 1);
}

#[test]
fn test_multiple_forward_declarations() {
    // Test that multiple forward declarations are allowed
    let input = r"
        struct A;
        struct A;
        struct A {
            long field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_forward_declaration_after_definition() {
    // Test that a forward declaration after a full definition works
    let input = r"
        struct A {
            long field;
        };
        struct A;
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_conflicting_struct_definitions() {
    // Test that two full struct definitions conflict
    let input = r"
        struct A {
            long field1;
        };
        struct A {
            long field2;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should have a conflicting definitions error
    assert!(!hir.errors.is_empty());
}

#[test]
fn test_union_forward_declaration() {
    // Test union forward declaration
    let input = r"
        union U;
        union U switch (long) {
            case 1: long x;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_interface_forward_declaration() {
    // Test interface forward declaration
    let input = r"
        interface I;
        interface I {
            void method();
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_valuetype_forward_declaration() {
    // Test valuetype forward declaration
    let input = r"
        valuetype V;
        valuetype V long;
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_mismatched_forward_declaration_types() {
    // Test that forward declaring as one type and defining as another is an error
    let input = r"
        struct A;
        union A switch (long) {
            case 1: long x;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should have a conflicting definitions error
    assert!(!hir.errors.is_empty());
}

#[test]
fn test_forward_declaration_with_usage() {
    // Test that forward declarations can be used before being fully defined
    let input = r"
        struct B;
        
        struct A {
            B* ptr;
        };
        
        struct B {
            long value;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_nested_forward_declarations() {
    // Test forward declarations inside modules
    let input = r"
        module M {
            struct S;
            struct S {
                long field;
            };
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}
