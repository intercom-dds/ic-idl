// Copyright 2026 KONGSBERG
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

use common::test_lint_preproc;
use insta::assert_snapshot;

#[test]
fn extra_tokens_after_endif() {
    assert_snapshot!(test_lint_preproc(
        r"
#ifdef FOO
#endif extra tokens
struct Foo {};
",
    ));
}

#[test]
fn extra_tokens_after_else() {
    assert_snapshot!(test_lint_preproc(
        r"
#ifdef FOO
#else extra
#endif
struct Foo {};
",
    ));
}

#[test]
fn extra_tokens_after_ifdef() {
    assert_snapshot!(test_lint_preproc(
        r"
#ifdef FOO extra tokens
#endif
struct Foo {};
",
    ));
}

#[test]
fn extra_tokens_after_ifndef() {
    assert_snapshot!(test_lint_preproc(
        r"
#ifndef FOO extra
#endif
struct Foo {};
",
    ));
}

#[test]
fn extra_tokens_after_undef() {
    assert_snapshot!(test_lint_preproc(
        r"
#define FOO
#undef FOO extra tokens
struct Foo {};
",
    ));
}

#[test]
fn extra_tokens_after_include() {
    assert_snapshot!(test_lint_preproc(
        r#"
#include "nonexistent.idl" extra
struct Foo {};
"#,
    ));
}

#[test]
fn extra_tokens_after_line() {
    assert_snapshot!(test_lint_preproc(
        r"
#line 100 extra
struct Foo {};
",
    ));
}
