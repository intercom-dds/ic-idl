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

//! Tests for empty struct definitions inside interfaces.

#[test]
fn test_empty_struct_in_interface() {
    let input = r"
        interface foo {
            struct abc {};
        };
        
        struct bar {
            foo::abc value;
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(parsed.errors.is_empty());

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Empty struct in interface should be considered complete: {:?}",
        result.errors
    );
}

#[test]
fn test_non_empty_struct_in_interface() {
    let input = r"
        interface foo {
            struct abc {
                long x;
            };
        };
        
        struct bar {
            foo::abc value;
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(parsed.errors.is_empty());

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Non-empty struct in interface should work: {:?}",
        result.errors
    );
}
