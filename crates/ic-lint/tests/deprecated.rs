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
use common::test_lint_hir;

#[test]
#[ignore = "Annotation lowering not implemented"]
fn no_deprecated_usage() {
    let source = r"
struct Point {
    long x;
    long y;
};

struct Line {
    Point start;
    Point end;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_struct_usage() {
    let source = r"
@deprecated
struct OldPoint {
    long x;
    long y;
};

struct Line {
    OldPoint start;  // Should warn
    OldPoint end;    // Should warn
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_field_usage() {
    let source = r"
struct Config {
    @deprecated long old_timeout;
    long timeout;
};

interface Service {
    void configure(in Config cfg);
    @deprecated void old_method();
    void new_method();
};

struct Usage {
    Config config;
    // The following should trigger warnings when we access config.old_timeout
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_with_message() {
    let source = r#"
@deprecated("Use NewAPI instead")
interface OldAPI {
    void doSomething();
};

@deprecated
typedef string OldString;

struct Implementation {
    OldAPI api;      // Should warn with message
    OldString name;  // Should warn
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_inheritance() {
    let source = r"
@deprecated
interface OldBase {
    void method();
};

interface NewDerived : OldBase {  // Should warn about inheriting from deprecated
    void newMethod();
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_enum_usage() {
    let source = r"
@deprecated
enum OldStatus { OK, ERROR, UNKNOWN };

enum NewStatus { SUCCESS, FAILURE, PENDING };

union Result switch (OldStatus) {  // Should warn
    case OK: long value;
    case ERROR: string error;
    default: void;
};
";

    assert_snapshot!(test_lint_hir(source));
}
