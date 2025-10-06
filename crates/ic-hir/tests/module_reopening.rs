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
fn test_module_reopening() {
    let idl = r"
        module Foo {
            struct Bar {
                long x;
            };
        };
        
        module Foo {
            struct Baz {
                Bar bar;
            };
        };
    ";

    let hir = common::parse_and_resolve_successfully(idl);

    // Each module declaration gets its own DefId, so we should find 2 modules named Foo
    let mut module_count = 0;
    let mut total_structs = 0;

    for def in &hir {
        if def.ident.name == "Foo" {
            module_count += 1;
            if let ic_hir::hir::DefKind::Module(module) = &def.kind {
                // Count the number of definitions in this module instance
                total_structs += module.definitions.len();
            }
        }
    }

    assert_eq!(module_count, 2, "Should find 2 module declarations for Foo");
    assert_eq!(
        total_structs, 2,
        "Total structs across all Foo modules should be 2 (Bar and Baz)"
    );
}

#[test]
fn test_module_reopening_with_references() {
    let idl = r"
        module DDS {
            struct TypeA {
                long value;
            };
        };
        
        module DDS {
            struct TypeB {
                TypeA a;  // Should resolve to DDS::TypeA
            };
        };
    ";

    // Should have no errors
    common::parse_and_resolve_successfully(idl);
}

#[test]
fn test_nested_module_reopening() {
    let idl = r"
        module Outer {
            module Inner {
                struct First {
                    long x;
                };
            };
        };
        
        module Outer {
            module Inner {
                struct Second {
                    First first;
                };
            };
        };
    ";

    // Should have no errors
    common::parse_and_resolve_successfully(idl);
}

#[test]
fn test_module_reopening_different_case() {
    // IDL is case-insensitive, so FOO and Foo should be the same module
    let idl = r"
        module FOO {
            struct TypeA {
                long x;
            };
        };
        
        module Foo {
            struct TypeB {
                TypeA a;  // Should resolve because type visibility is inherited
            };
        };
    ";

    // Should have no errors - TypeA is visible in the second module declaration
    // But should have a warning about case inconsistency
    let diagnostics = common::compile_idl_with_warnings(idl);
    insta::assert_snapshot!(diagnostics);
}
