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
fn sequence_with_annotation() {
    let source = r"
struct Foo {
    sequence<@key long> field;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn map_with_key_annotation() {
    let source = r"
struct Foo {
    map<@key string, long> field;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn map_with_value_annotation() {
    let source = r"
struct Foo {
    map<string, @key long> field;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn map_with_both_annotations() {
    let source = r"
struct Foo {
    map<@key string, @key long> field;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn nested_sequence_with_annotation() {
    let source = r"
struct Foo {
    sequence<sequence<@key long>> field;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn valid_sequence_no_annotation() {
    let source = r"
struct Foo {
    sequence<long> field;
};
";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for valid sequence, but got: {output}"
    );
}

#[test]
fn valid_map_no_annotation() {
    let source = r"
struct Foo {
    map<string, long> field;
};
";

    let output = test_lint(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for valid map, but got: {output}"
    );
}
