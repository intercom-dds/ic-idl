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
fn test_interface_types_require_qualification() {
    let input = r"
        interface IService {
            struct InternalData {
                long value;
            };
            
            enum Status {
                OK,
                ERROR
            };
        };
        
        // These should fail - unqualified access to interface types
        struct BadContainer1 {
            InternalData data;
        };
        
        struct BadContainer2 {
            Status status;
        };
    ";

    let diagnostics = common::parse_and_expect_errors(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_qualified_interface_access_works() {
    let input = r"
        interface IService {
            struct InternalData {
                long value;
            };
            
            enum Status {
                OK,
                ERROR
            };
        };
        
        // These should work - qualified access
        struct GoodContainer {
            IService::InternalData data;
            IService::Status status;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty(), "Qualified access should work");
}

#[test]
fn test_visibility_within_interface() {
    let input = r"
        interface IService {
            struct InternalData {
                long value;
            };
            
            struct Container {
                InternalData data; // Should work - we're inside the interface
            };
            
            typedef InternalData AliasedData; // Should work
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(
        result.errors.is_empty(),
        "Types should be visible within the same interface"
    );
}

#[test]
fn test_nested_interface_visibility() {
    let input = r"
        module Outer {
            interface IService {
                struct InternalData {
                    long value;
                };
            };
            
            struct Container {
                IService::InternalData data; // Should work - qualified access
            };
        };
        
        struct RootContainer {
            Outer::IService::InternalData data; // Should work - fully qualified
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(
        result.errors.is_empty(),
        "Qualified access through modules should work"
    );
}
