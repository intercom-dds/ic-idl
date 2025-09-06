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

#[test]
fn test_char_arithmetic_binary() {
    let input = r"
        const long a = 'A' + 1;
        const long b = 2 - 'B';
        const long c = 'C' * 3;
        const long d = 100 / 'D';
        const long e = 'E' % 10;
        const long f = 'F' & 0xFF;
        const long g = 'G' | 0x20;
        const long h = 'H' ^ 0x01;
        const long i = 'I' << 2;
        const long j = 'J' >> 1;
    ";

    let output = common::test_lint(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_char_arithmetic_unary() {
    let input = r"
        const long a = -'A';
        const long b = +'B';
        const long c = ~'C';
    ";

    let output = common::test_lint(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_char_no_arithmetic_no_warning() {
    let input = r#"
        const char letter = 'A';
        const long code = 65;
        const string text = "ABC";
    "#;

    let output = common::test_lint(input);
    assert!(!output.contains("char literal used in arithmetic"));
}

#[test]
fn test_char_arithmetic_nested() {
    let input = r"
        const long a = ('A' + 1) * 2;
        const long b = 3 - ('B' & 0xFF);
        const long c = ~('C' | 0x20);
    ";

    let output = common::test_lint(input);
    insta::assert_snapshot!(output);
}

#[test]
fn test_char_arithmetic_in_struct() {
    let input = r"
        struct Data {
            @id('A' + 1)
            long field1;
            
            @value(10 - 'B')
            long field2;
        };
    ";

    let output = common::test_lint(input);
    insta::assert_snapshot!(output);
}
