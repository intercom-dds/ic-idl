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

#[test]
fn test_malformed_string_no_panic() {
    // Test case from the bug report - missing opening quote
    let input =
        r#"const map<string, string> my_map = {{"abc", "abc"}, {"def", "def"}, {"ghi", ghi"}};"#;

    // Should not panic, but will have parse errors
    let result = ic_parse::from_str(input);
    assert!(!result.errors.is_empty());

    // The parser will report an error about the unterminated string token
    // We just verify that we get an error (no panic) and parsing fails
}

#[test]
fn test_unterminated_string() {
    // String without closing quote
    let input = r#"const string s = "hello world;"#;

    // Should not panic
    let result = ic_parse::from_str(input);
    assert!(!result.errors.is_empty());

    // The parser will report an error about the unterminated string token
}

#[test]
fn test_string_with_only_opening_quote() {
    // String with only opening quote
    let input = r#"const string s = ";"#;

    // Should not panic
    let result = ic_parse::from_str(input);
    assert!(!result.errors.is_empty());
}

#[test]
fn test_normal_string() {
    // Normal string for comparison
    let input = r#"const string s = "hello";"#;

    let result = ic_parse::from_str(input);
    assert!(result.errors.is_empty());
}

#[test]
fn test_string_with_newline() {
    // String with newline (mentioned in the issue)
    let input = "const string value = \"foo\n;";

    let result = ic_parse::from_str(input);
    assert!(!result.errors.is_empty());

    // The parser will report an error about the unterminated string token
}
