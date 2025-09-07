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

#[test]
fn test_struct_inherit_from_forward_decl() {
    let input = r"
        struct Base;  // Forward declaration
        
        struct Derived : Base {  // Should fail - inheriting from incomplete type
            int32 value;
        };
        
        struct Base {  // Definition comes later
            int32 x;
        };
    ";

    let (result, _, diagnostics) = common::parse_and_resolve(input);

    // Should have an error about inheriting from incomplete type
    assert!(!result.errors.is_empty());

    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_struct_inherit_from_complete_type() {
    let input = r"
        struct Base {  // Complete definition
            int32 x;
        };
        
        struct Derived : Base {  // Should succeed - inheriting from complete type
            int32 y;
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
fn test_interface_inherit_from_forward_decl() {
    let input = r"
        interface Base;  // Forward declaration
        
        interface Derived : Base {  // Should fail - inheriting from incomplete type
            void method();
        };
        
        interface Base {  // Definition comes later
            void baseMethod();
        };
    ";

    let (result, _, diagnostics) = common::parse_and_resolve(input);

    // Should have an error about inheriting from incomplete type
    assert!(!result.errors.is_empty());

    insta::assert_snapshot!(diagnostics);
}
