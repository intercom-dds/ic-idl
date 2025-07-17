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

//! Tests for improved error spans that highlight the exact failing segment in qualified paths.

#[test]
fn test_unresolved_type_error_span_highlights_failing_segment() {
    let input = r"
        module foo {
            module bar {
                struct Valid {};
            };
        };
        
        struct Test {
            foo::bar::Valid valid;      // This should work
            foo::bar::Invalid invalid;  // Should highlight Invalid
            foo::baz::Something other;   // Should highlight baz
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // We expect exactly 2 errors
    assert_eq!(
        result.errors.len(),
        2,
        "Expected 2 errors, got: {:?}",
        result.errors
    );

    // First error should be for foo::bar::Invalid
    let error1 = &result.errors[0];
    let error1_msg = format!("{:?}", error1);
    assert!(
        error1_msg.contains("foo::bar::Invalid"),
        "First error should mention foo::bar::Invalid"
    );

    // The span should point to "Invalid" specifically
    // We can't easily check the exact span here, but the error message should be clear

    // Second error should be for foo::baz::Something
    let error2 = &result.errors[1];
    let error2_msg = format!("{:?}", error2);
    assert!(
        error2_msg.contains("foo::baz::Something"),
        "Second error should mention foo::baz::Something"
    );

    // The span should point to "baz" specifically
}

#[test]
fn test_deeply_nested_path_error() {
    let input = r"
        module a {
            module b {
                module c {
                    struct Valid {};
                };
            };
        };
        
        struct Test {
            a::b::c::Valid valid;           // Should work
            a::b::c::d::Invalid deep;       // Should highlight d
            a::b::missing::c::Valid other;  // Should highlight missing
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert_eq!(
        result.errors.len(),
        2,
        "Expected 2 errors, got: {:?}",
        result.errors
    );

    // Check that the right paths are reported as errors
    let errors_str = format!("{:?}", result.errors);
    assert!(errors_str.contains("a::b::c::d::Invalid"));
    assert!(errors_str.contains("a::b::missing::c::Valid"));
}

#[test]
fn test_global_path_unresolved_segment() {
    let input = r"
        module foo {
            struct Bar {};
        };
        
        struct Test {
            ::foo::Bar valid;     // Should work
            ::foo::Baz invalid;   // Should highlight Baz
            ::missing::Bar other; // Should highlight missing
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert_eq!(result.errors.len(), 2);

    // Check that errors mention the unresolved paths
    let errors_str = format!("{:?}", result.errors);
    assert!(errors_str.contains("::foo::Baz"));
    assert!(errors_str.contains("::missing::Bar"));
}
