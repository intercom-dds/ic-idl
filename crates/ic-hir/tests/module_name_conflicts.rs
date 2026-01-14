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

use insta::assert_snapshot;

#[test]
fn module_struct_name_conflict() {
    let idl = r"
        module foo {};
        struct foo {};
    ";

    let diagnostics = common::parse_and_expect_errors(idl);
    assert_snapshot!(diagnostics);
}

#[test]
fn struct_module_name_conflict() {
    let idl = r"
        struct foo {};
        module foo {};
    ";

    let diagnostics = common::parse_and_expect_errors(idl);
    assert_snapshot!(diagnostics);
}

#[test]
fn module_interface_name_conflict() {
    let idl = r"
        module foo {};
        interface foo {};
    ";

    let diagnostics = common::parse_and_expect_errors(idl);
    assert_snapshot!(diagnostics);
}

#[test]
fn module_enum_name_conflict() {
    let idl = r"
        module foo {};
        enum foo { A, B };
    ";

    let diagnostics = common::parse_and_expect_errors(idl);
    assert_snapshot!(diagnostics);
}

#[test]
fn module_reopening_is_ok() {
    let idl = r"
        module foo {};
        module foo {};
    ";

    common::parse_and_resolve_successfully(idl);
}
