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
fn test_struct_forward_declaration() {
    let input = r#"
        struct Foo;
        struct Foo {
            long x;
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Verify we have one struct definition
    let structs: Vec<_> = result
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, ic_hir::hir::DefKind::Struct(_)))
        .collect();
    assert_eq!(structs.len(), 1, "Expected exactly one struct definition");
}

#[test]
fn test_interface_forward_declaration() {
    let input = r#"
        interface Service;
        interface Service {
            void method();
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );

    // Verify we have one interface definition
    let interfaces: Vec<_> = result
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, ic_hir::hir::DefKind::Interface(_)))
        .collect();
    assert_eq!(
        interfaces.len(),
        1,
        "Expected exactly one interface definition"
    );
}

#[test]
fn test_duplicate_definitions_same_scope() {
    let input = r#"
        struct Foo {
            long x;
        };
        
        struct Foo {  // Duplicate!
            double y;
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        !result.errors.is_empty(),
        "Expected error for duplicate struct"
    );
    let error_msg = format!("{:?}", result.errors[0]);
    assert!(
        error_msg.contains("duplicate definition"),
        "Error message doesn't contain 'duplicate definition': {}",
        error_msg
    );
}

#[test]
fn test_duplicate_names_different_scopes() {
    let input = r#"
        struct Point {
            long x;
            long y;
        };
        
        module graphics {
            struct Point {  // OK - different scope
                double x;
                double y;
                double z;
            };
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors for same name in different scopes, got: {:?}",
        result.errors
    );

    // Verify we have two struct definitions
    let structs: Vec<_> = result
        .context
        .definitions
        .iter()
        .filter(|(_, def)| matches!(def.kind, ic_hir::hir::DefKind::Struct(_)))
        .collect();
    assert_eq!(structs.len(), 2, "Expected exactly two struct definitions");
}

#[test]
fn test_forward_declaration_type_mismatch() {
    let input = r#"
        struct Foo;
        interface Foo {  // Type mismatch!
            void method();
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        !result.errors.is_empty(),
        "Expected error for type mismatch"
    );
}
