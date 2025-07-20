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

use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn valid_oneway_operation() {
    let source = r"
interface Service {
    oneway void notify(in string message);
    oneway void shutdown();
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn oneway_with_return_type() {
    let source = r"
interface Service {
    oneway string getMessage();  // oneway cannot return a value
    oneway long calculate(in long x, in long y);
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn oneway_with_out_parameter() {
    let source = r"
interface Service {
    oneway void process(in string input, out string result);  // oneway cannot have out parameters
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn oneway_with_inout_parameter() {
    let source = r"
interface Service {
    oneway void update(inout long value);  // oneway cannot have inout parameters
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn oneway_with_exceptions() {
    let source = r"
exception ProcessError {
    string reason;
};

interface Service {
    oneway void process(in string data) raises (ProcessError);  // oneway cannot raise exceptions
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn oneway_multiple_issues() {
    let source = r"
exception Error {
    string msg;
};

interface Service {
    oneway boolean doWork(in long input, out string output) raises (Error);  // Multiple issues
};
";

    assert_snapshot!(test_lint(source));
}
