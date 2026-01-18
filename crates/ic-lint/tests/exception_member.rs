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

use insta::assert_snapshot;

mod common;
use common::{lint_hir, test_lint_hir};

#[test]
fn test_exception_as_struct_member() {
    let source = r"
exception NetworkError {
    long code;
    string message;
};

struct ErrorLog {
    NetworkError error;
    string timestamp;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_as_union_variant() {
    let source = r"
exception DatabaseError {
    long code;
};

union Result switch (long) {
    case 1: long value;
    case 2: DatabaseError error;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_as_parameter() {
    let source = r"
exception ValidationError {
    string field;
};

interface ErrorHandler {
    void logError(in ValidationError error);
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_as_return_type() {
    let source = r"
exception TimeoutError {};

interface Service {
    TimeoutError getLastError();
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_as_attribute_type() {
    let source = r"
exception ConfigError {
    string message;
};

interface Config {
    attribute ConfigError lastError;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_as_typedef() {
    let source = r"
exception AuthError {
    long code;
};

typedef AuthError ErrorAlias;
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_exception_in_sequence() {
    let source = r"
exception IOError {
    string path;
};

struct ErrorReport {
    sequence<IOError> errors;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_in_array() {
    let source = r"
exception ParseError {
    long line;
};

struct Parser {
    ParseError errors[10];
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_multiple_exception_violations() {
    let source = r"
exception GenericError {
    string message;
};

struct Data {
    GenericError error1;
    sequence<GenericError> errors;
};

interface Service {
    void process(in GenericError err);
    GenericError getError();
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_in_raises_valid() {
    let source = r"
exception NetworkError {
    long code;
    string message;
};

exception TimeoutError {};

interface Service {
    void connect() raises (NetworkError, TimeoutError);
    long query() raises (NetworkError);
};
";

    let report = lint_hir(source);
    assert!(
        report.errors.is_empty(),
        "Expected no errors for exceptions in raises expressions"
    );
    assert!(report.warnings.is_empty());
}

#[test]
fn test_no_exception_usage() {
    let source = r"
struct Point {
    long x;
    long y;
};

interface Graphics {
    void drawPoint(in Point p);
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_exception_in_nested_types() {
    let source = r"
exception Error {
    long code;
};

struct Container {
    sequence<sequence<Error>> nested_errors;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_exception_in_valuetype() {
    let source = r"
exception CustomError {};

valuetype Data {
    public CustomError error;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_valid_exception_definition_only() {
    let source = r"
exception NetworkError {
    long code;
    string message;
};

exception TimeoutError : NetworkError {
    long elapsed_time;
};
";

    let report = lint_hir(source);
    assert!(
        report.errors.is_empty(),
        "Expected no errors for exception definitions without usage"
    );
    assert!(report.warnings.is_empty());
}

#[test]
fn test_typedef_to_struct() {
    let source = r"
struct Point {
    long x;
    long y;
};

typedef Point Position;

struct Container {
    Position pos;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_typedef_to_union() {
    let source = r"
union Value switch (long) {
    case 1: long intVal;
    case 2: string strVal;
};

typedef Value ValueAlias;

struct Data {
    ValueAlias val;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_typedef_to_primitive() {
    let source = r"
typedef long Integer;
typedef string Text;

struct Record {
    Integer id;
    Text name;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_typedef_to_sequence() {
    let source = r"
typedef sequence<long> IntList;

struct Data {
    IntList numbers;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}

#[test]
fn test_typedef_chain() {
    let source = r"
struct Point {
    long x;
    long y;
};

typedef Point Position;
typedef Position Location;

struct MapData {
    Location center;
};
";

    let report = lint_hir(source);
    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
}
