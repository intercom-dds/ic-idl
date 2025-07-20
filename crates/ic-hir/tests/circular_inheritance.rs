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

use {ic_hir, ic_parse};

#[test]
fn test_circular_struct_inheritance_no_infinite_loop() {
    // Test that circular struct inheritance doesn't cause an infinite loop
    // in hygiene checking
    let input = r#"
        struct A : B {
            long a_field;
        };
        
        struct B : A {
            long b_field;
        };
    "#;

    // Parse
    let parse_result = ic_parse::from_str(input);

    // Lower to HIR
    let hir = ic_hir::from_ast(parse_result.tree);

    // The hygiene check should complete without hanging
    // If there's an infinite loop, this test will timeout
    assert!(!hir.order.is_empty());

    // We expect errors about circular inheritance, but the important thing
    // is that the hygiene check completes
}

#[test]
fn test_indirect_circular_inheritance_no_infinite_loop() {
    // Test indirect circular inheritance (A -> B -> C -> A)
    let input = r#"
        struct A : C {
            long a_field;
        };
        
        struct B : A {
            long b_field;
        };
        
        struct C : B {
            long c_field;
        };
    "#;

    // Parse
    let parse_result = ic_parse::from_str(input);

    // Lower to HIR
    let hir = ic_hir::from_ast(parse_result.tree);

    // The hygiene check should complete without hanging
    assert!(!hir.order.is_empty());
}

#[test]
fn test_self_inheritance_no_infinite_loop() {
    // Test self-inheritance (A : A)
    let input = r#"
        struct A : A {
            long field;
        };
    "#;

    // Parse
    let parse_result = ic_parse::from_str(input);

    // Lower to HIR
    let hir = ic_hir::from_ast(parse_result.tree);

    // The hygiene check should complete without hanging
    assert!(!hir.order.is_empty());
}
