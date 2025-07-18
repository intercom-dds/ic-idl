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
fn test_union_with_octet_discriminator() {
    let idl = r"
        union MyUnion switch (octet) {
            case 0:
                long a;
            case 1:
                string b;
        };
    ";

    // Should have no errors - octet is a valid discriminator type
    common::parse_and_resolve_successfully(idl);
}

#[test]
fn test_union_with_various_discriminators() {
    let idl = r"
        enum MyEnum { A, B, C };
        
        union Union1 switch (boolean) {
            case TRUE: long x;
            case FALSE: string y;
        };
        
        union Union2 switch (char) {
            case 'a': long x;
            case 'b': string y;
        };
        
        union Union3 switch (short) {
            case 1: long x;
            case 2: string y;
        };
        
        union Union4 switch (unsigned long) {
            case 100: long x;
            case 200: string y;
        };
        
        union Union5 switch (MyEnum) {
            case A: long x;
            case B: string y;
        };
    ";

    // All should be valid discriminator types
    common::parse_and_resolve_successfully(idl);
}

#[test]
fn test_invalid_union_discriminator() {
    let idl = r"
        struct Point { long x, y; };
        
        union BadUnion switch (Point) {  // Structs can't be discriminators
            case 1: long value;
        };
    ";

    // Should have an error about invalid discriminator type
    let diagnostics = common::parse_and_expect_errors(idl);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_union_discriminator_case_insensitive() {
    let idl = r"
        union Union1 switch (OCTET) {    // Uppercase
            case 0: long x;
        };
        
        union Union2 switch (Octet) {    // Mixed case
            case 1: string y;
        };
        
        union Union3 switch (octet) {    // Lowercase
            case 2: double z;
        };
    ";

    // All should work - case insensitive
    common::parse_and_resolve_successfully(idl);
}

#[test]
fn test_nested_union_resolution() {
    let idl = r"
        module Test {
            union Inner switch (octet) {
                case 0: long a;
                case 1: string b;
            };
            
            union Outer switch (boolean) {
                case TRUE: Inner inner;
                case FALSE: double value;
            };
        };
    ";

    // Should resolve correctly
    common::parse_and_resolve_successfully(idl);
}
