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

use common::lint_hir;

#[test]
#[ignore = "Annotation lowering not implemented"]
fn no_deprecated_usage() {
    let report = lint_hir(
        r"
struct Point {
    long x;
    long y;
};

struct Line {
    Point start;
    Point end;
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_struct_usage() {
    let report = lint_hir(
        r"
@deprecated
struct OldPoint {
    long x;
    long y;
};

struct Line {
    OldPoint start;  // Should warn
    OldPoint end;    // Should warn
};
",
    );

    assert_eq!(report.errors.len(), 2);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("deprecated type"));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_with_message() {
    let report = lint_hir(
        r#"
@deprecated("Use NewAPI instead")
interface OldAPI {
    void doSomething();
};

typedef OldAPI ServiceRef;  // Should warn
"#,
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("Use NewAPI instead"));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn deprecated_const_usage() {
    let report = lint_hir(
        r"
@deprecated
const long OLD_VERSION = 1;

const long CURRENT_VERSION = OLD_VERSION + 1;  // Should warn
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("deprecated constant"));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn obsolete_annotation() {
    let report = lint_hir(
        r"
@obsolete
enum OldStatus {
    ACTIVE,
    INACTIVE
};

struct Record {
    OldStatus status;  // Should warn
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("deprecated"));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn nested_deprecated_usage() {
    let report = lint_hir(
        r"
@deprecated
struct OldData {
    long value;
};

typedef sequence<OldData> OldDataList;  // Should warn about OldData

struct Container {
    OldDataList items;  // Should warn about OldData (through typedef)
};
",
    );

    assert!(!report.errors.is_empty()); // At least one warning for OldData usage
}
