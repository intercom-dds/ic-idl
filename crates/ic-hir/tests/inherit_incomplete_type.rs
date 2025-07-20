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
fn test_struct_inherit_from_incomplete_type() {
    // Test that inheriting from a forward-declared struct produces a clear error
    let input = r"
        struct Base;
        struct Derived : Base {
            long field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should have errors
    assert!(!hir.errors.is_empty());

    // Should have an error mentioning incomplete type
    let error_found = hir.errors.iter().any(|e| {
        let error_str = format!("{:?}", e);
        error_str.contains("cannot inherit from incomplete type")
    });
    assert!(
        error_found,
        "Expected error about inheriting from incomplete type"
    );
}

#[test]
fn test_interface_inherit_from_incomplete_type() {
    // Test that inheriting from a forward-declared interface produces a clear error
    let input = r"
        interface IBase;
        interface IDerived : IBase {
            void method();
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should have errors
    assert!(!hir.errors.is_empty());

    // Should have an error mentioning incomplete type
    let error_found = hir.errors.iter().any(|e| {
        let error_str = format!("{:?}", e);
        error_str.contains("cannot inherit from incomplete type")
    });
    assert!(
        error_found,
        "Expected error about inheriting from incomplete type"
    );
}

#[test]
fn test_struct_inherit_from_complete_type() {
    // Test that inheriting from a fully defined struct works correctly
    let input = r"
        struct Base {
            long base_field;
        };
        struct Derived : Base {
            long derived_field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_inherit_from_wrong_type() {
    // Test that inheriting a struct from an enum produces appropriate error
    let input = r"
        enum Color { RED, GREEN, BLUE };
        struct MyStruct : Color {
            long field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should have errors
    assert!(!hir.errors.is_empty());

    // Should have an error mentioning non-struct type
    let error_found = hir.errors.iter().any(|e| {
        let error_str = format!("{:?}", e);
        error_str.contains("cannot inherit from non-struct type")
    });
    assert!(
        error_found,
        "Expected error about inheriting from non-struct type"
    );
}

#[test]
fn test_struct_forward_decl_then_inherit() {
    // Test that forward declaration followed by definition then inheritance works
    let input = r"
        struct Base;
        struct Base {
            long field;
        };
        struct Derived : Base {
            long other_field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should compile without errors
    assert_eq!(hir.errors.len(), 0);
}

#[test]
fn test_inherit_from_later_defined_type() {
    // This should fail - inheriting from a type that's defined later in the file
    let input = r"
        struct Derived : Base {
            long derived_field;
        };
        
        struct Base {
            long base_field;
        };
    ";

    let parse_result = ic_parse::from_str(input);
    let hir = ic_hir::from_ast(parse_result.tree);

    // Should have errors about incomplete type
    assert!(
        !hir.errors.is_empty(),
        "Expected errors but got none. Warnings: {:?}",
        hir.warnings
    );

    let error_found = hir.errors.iter().any(|e| {
        let error_str = format!("{:?}", e);
        error_str.contains("incomplete") || error_str.contains("not defined")
    });
    assert!(
        error_found,
        "Expected error about incomplete type, got: {:?}",
        hir.errors
    );
}
