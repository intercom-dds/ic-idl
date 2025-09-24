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

// Test forward declaration validation

mod common;

#[test]
fn test_valid_forward_decl_then_define() {
    let input = r"
        struct Foo;  // Forward declaration
        struct Foo { // Definition
            int32 value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_valid_define_then_forward_decl() {
    let input = r"
        struct Bar { // Definition
            string name;
        };
        struct Bar;  // Forward declaration after definition is allowed
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_multiple_forward_declarations() {
    let input = r"
        struct Thing;  // First forward declaration
        struct Thing;  // Second forward declaration
        struct Thing;  // Third forward declaration
        struct Thing { // Definition
            double value;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors - multiple forward declarations are allowed
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_mismatched_forward_declaration() {
    let input = r"
        union MyType;   // Forward declared as union
        struct MyType { // But defined as struct!
            int32 field;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have an error about mismatched types
    assert!(
        !result.errors.is_empty(),
        "Expected error for mismatched forward declaration"
    );

    // Just check that we have errors - we can't easily inspect the message
}

#[test]
fn test_multiple_mismatched_forward_declarations() {
    let input = r"
        struct Complex;    // First forward declaration as struct
        union Complex;     // Second forward declaration as union - conflicts!
        interface Complex { // Definition as interface - conflicts with both!
            void method();
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have errors about conflicts
    assert!(
        !result.errors.is_empty(),
        "Expected errors for conflicting declarations"
    );
}

#[test]
fn test_forward_declaration_without_definition() {
    let input = r"
        struct Missing;  // Forward declaration with no definition
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have an error about missing definition
    assert!(
        !result.errors.is_empty(),
        "Expected error for undefined forward declaration"
    );
}

#[test]
fn test_interface_forward_declaration() {
    let input = r"
        interface IFoo;
        interface IFoo {
            void doSomething();
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_valuetype_forward_declaration() {
    let input = r"
        valuetype MyValueType;
        valuetype MyValueType {
            public int32 x;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn test_native_declarations_not_undefined() {
    let input = r"
        // Native declarations should remain as declarations
        native NativeHandle;
        
        // Can use native types in other definitions
        struct Container {
            NativeHandle handle;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(
        result.errors.is_empty(),
        "Native declarations should not produce errors: {:?}",
        result.errors
    );
}

#[test]
fn test_undefined_forward_declarations_with_native() {
    let input = r"
        // These should error (forward declarations without definitions)
        struct UndefinedStruct;
        interface UndefinedInterface;
        
        // This should NOT error (native declaration)
        native NativeType;
        
        // Using the types
        struct Container {
            NativeType native_field;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);

    // Should have exactly 2 errors for the undefined struct and interface
    assert_eq!(
        result.errors.len(),
        2,
        "Expected 2 errors for undefined forward declarations, got: {:?}",
        result.errors
    );

    // Verify the errors are about the right types
    let error_messages: Vec<_> = result.errors.iter().map(|e| format!("{e}")).collect();

    assert!(
        error_messages
            .iter()
            .any(|msg| msg.contains("UndefinedStruct")),
        "Should have error about UndefinedStruct"
    );
    assert!(
        error_messages
            .iter()
            .any(|msg| msg.contains("UndefinedInterface")),
        "Should have error about UndefinedInterface"
    );
    assert!(
        !error_messages.iter().any(|msg| msg.contains("NativeType")),
        "Should NOT have error about NativeType"
    );
}
