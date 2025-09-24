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

mod common;
use common::parse_and_resolve;

#[test]
fn test_duplicate_bitset_detection() {
    let idl = r"
        bitset Flags {
            bitfield<1> enable;
            bitfield<1> ready;
        };
        
        // This should trigger a duplicate definition error
        bitset Flags {
            bitfield<2> status;
        };
    ";

    let (result, _, diagnostics) = parse_and_resolve(idl);
    assert!(
        !result.errors.is_empty(),
        "Expected duplicate definition error"
    );
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_case_insensitive_duplicate_bitset() {
    let idl = r"
        bitset Config {
            bitfield<8> mode;
        };
        
        // Different case should still be considered duplicate
        bitset CONFIG {
            bitfield<16> value;
        };
    ";

    let (result, _, diagnostics) = parse_and_resolve(idl);
    assert!(
        !result.errors.is_empty(),
        "Expected duplicate definition error for case-insensitive match"
    );
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_bitset_in_different_scopes() {
    let idl = r"
        bitset Status {
            bitfield<4> code;
        };
        
        module foo {
            // Same name in different scope should be OK
            bitset Status {
                bitfield<8> value;
            };
        };
    ";

    let (result, _, _) = parse_and_resolve(idl);
    assert!(result.errors.is_empty());
}

#[test]
fn test_multiple_bitsets_unique_names() {
    let idl = r"
        bitset Flags1 {
            bitfield<1> enable;
        };
        
        bitset Flags2 {
            bitfield<1> disable;
        };
        
        bitset Flags3 {
            bitfield<1> reset;
        };
    ";

    let (result, _, _) = parse_and_resolve(idl);
    assert!(result.errors.is_empty());
}
