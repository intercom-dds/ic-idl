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
fn test_multi_param_annotation_warning() {
    let input = r"
        @annotation range {
            long min;
            long max;
        };
        
        // This should produce a warning
        @range(0, 10)
        struct BadRange {
            long value;
        };
    ";

    let (_, warning_msg) = common::parse_and_get_warnings(input);

    // Should have a warning about multiple parameters requiring named arguments
    assert!(warning_msg.contains("@range has 2 parameters and requires named arguments"));
}

#[test]
fn test_multi_param_annotation_named_ok() {
    let input = r"
        @annotation range {
            long min;
            long max;
        };
        
        // This is correct - named arguments
        @range(min=0, max=10)
        struct GoodRange {
            long value;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);
    // Should resolve without warnings
    assert_eq!(result.order.len(), 2); // annotation def + struct
}

#[test]
fn test_single_param_annotation_positional_ok() {
    let input = r"
        @annotation optional {
            boolean value;
        };
        
        // Single parameter - positional is OK
        @optional(false)
        struct OptionalTest {
            long value;
        };
    ";

    let result = common::parse_and_resolve_successfully(input);
    // Should resolve without warnings
    assert_eq!(result.order.len(), 2); // annotation def + struct
}

#[test]
fn test_mixed_named_positional_warning() {
    let input = r"
        @annotation test {
            long a;
            long b;
            long c;
        };
        
        // Mixed named and positional - should warn
        @test(5, b=10, c=15)
        struct Mixed {
            long value;
        };
    ";

    let (_, warning_msg) = common::parse_and_get_warnings(input);

    // Should have a warning about multiple parameters requiring named arguments
    assert!(warning_msg.contains("@test has 3 parameters and requires named arguments"));
}

#[test]
fn test_annotation_with_defaults() {
    let input = r"
        @annotation config {
            long timeout;
            boolean retry default true;
        };
        
        // Even with defaults, multiple params require named args
        @config(30)
        struct BadConfig {};
    ";

    let (_, warning_msg) = common::parse_and_get_warnings(input);

    // Should have a warning about multiple parameters requiring named arguments
    assert!(warning_msg.contains("@config has 2 parameters and requires named arguments"));
}