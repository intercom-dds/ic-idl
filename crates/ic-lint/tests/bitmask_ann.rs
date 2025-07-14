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

use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn test_bitmask_in_annotation() {
    let source = r"
        @annotation MyAnnotation {
            bitmask<unsigned long> Flags {
                FLAG_A = 0x01,
                FLAG_B = 0x02,
                FLAG_C = 0x04
            };
            unsigned long default_flags;
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_normal_bitmask_outside_annotation() {
    let source = r"
        bitmask<unsigned long> GlobalFlags {
            ENABLED = 0x01,
            VERBOSE = 0x02,
            DEBUG = 0x04
        };
        
        @annotation Settings {
            unsigned long flags;
        };
    ";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Should not warn for bitmasks outside annotations"
    );
}

#[test]
fn test_nested_bitmask_in_annotation() {
    let source = r"
        @annotation ComplexAnnotation {
            struct Config {
                string name;
            };
            
            bitmask<octet> Options {
                OPT_A = 1,
                OPT_B = 2
            };
            
            bitmask<unsigned short> MoreOptions {
                MORE_A = 0x10,
                MORE_B = 0x20
            };
        };
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_empty_annotation() {
    let source = r"
        @annotation EmptyAnnotation {
        };
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for empty annotations");
}
