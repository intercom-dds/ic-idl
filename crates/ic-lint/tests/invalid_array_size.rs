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

use common::test_lint;

#[test]
fn reasonable_array_sizes() {
    let output = test_lint(
        r#"
struct Foo {
    long small[10];
    long medium[1000];
    long large[100000];
};
"#,
    );

    assert!(output.is_empty());
}

#[test]
fn very_large_array() {
    let output = test_lint(
        r#"
struct Foo {
    long huge[10000000];  // 10 million
};
"#,
    );

    assert!(output.contains("exceeds reasonable limit"));
    assert!(output.contains("consider using a sequence"));
}

#[test]
fn negative_array_size() {
    let output = test_lint(
        r#"
struct Foo {
    long invalid[-10];
};
"#,
    );

    assert!(output.contains("negative array size"));
    assert!(output.contains("must be positive"));
}

#[test]
fn multi_dimensional_large_array() {
    let output = test_lint(
        r#"
struct Foo {
    long matrix[2000000][10];  // 2 million rows
};
"#,
    );

    assert!(output.contains("exceeds reasonable limit"));
}

#[test]
fn typedef_large_array() {
    let output = test_lint(
        r#"
typedef long BigArray[5000000];
"#,
    );

    assert!(output.contains("exceeds reasonable limit"));
}

#[test]
fn const_array_large() {
    let output = test_lint(
        r#"
const long DATA[2000000] = {1, 2, 3};
"#,
    );

    assert!(output.contains("exceeds reasonable limit"));
}
