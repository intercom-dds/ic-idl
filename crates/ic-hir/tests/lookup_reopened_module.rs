// Copyright 2025 KONGSBERG
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

#![cfg(test)]

mod common;
use common::parse_and_resolve;

#[test]
fn test_lookup_in_reopened_module() {
    let input = r"
        module Foo {
            struct First {
                long x;
            };
        };
        
        module Foo {
            struct Second {
                long y;
            };
        };
    ";

    let (hir, _, _) = parse_and_resolve(input);

    // These should all work
    assert!(
        hir.context.lookup_symbol("Foo").is_some(),
        "Failed to find Foo"
    );
    assert!(
        hir.context.lookup_symbol("Foo::First").is_some(),
        "Failed to find Foo::First"
    );
    assert!(
        hir.context.lookup_symbol("Foo::Second").is_some(),
        "Failed to find Foo::Second"
    );

    // Test the new lookup_modules method
    let all_foo_ids = hir.context.lookup_modules("Foo");
    assert_eq!(
        all_foo_ids.len(),
        2,
        "Should find both Foo module definitions"
    );
}

#[test]
fn test_xtypes_lookup_in_reopened_module() {
    let input = r"
        module DDS {
            module XTypes {
                struct TypeIdentifier {
                    long id;
                };
            };
        };
        
        module DDS {
            module XTypes {
                struct TypeObject {
                    long type_id;
                };
            };
        };
    ";

    let (hir, _, _) = parse_and_resolve(input);

    // This is what rename_xtypes does
    let xtypes_lookup = hir.context.lookup_symbol("DDS::XTypes");
    assert!(
        xtypes_lookup.is_some(),
        "Failed to find DDS::XTypes via lookup_symbol"
    );

    // Also test the nested lookups
    assert!(
        hir.context
            .lookup_symbol("DDS::XTypes::TypeIdentifier")
            .is_some(),
        "Failed to find TypeIdentifier"
    );
    assert!(
        hir.context
            .lookup_symbol("DDS::XTypes::TypeObject")
            .is_some(),
        "Failed to find TypeObject"
    );
}
