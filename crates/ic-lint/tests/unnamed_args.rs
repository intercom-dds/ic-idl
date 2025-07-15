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
fn valid_single_unnamed_args() {
    let report = lint_hir(
        r"
struct Foo {
    @min(0)
    @max(100)
    @bit(5)
    @id(42)
    long field;
};
",
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn valid_named_args() {
    let report = lint_hir(
        r#"
annotation MyAnn {
    long value;
    string description;
};

@MyAnn(value=42, description="test")
struct Foo {
    long field;
};
"#,
    );

    assert_eq!(report.errors.len(), 0);
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn valid_range_unnamed() {
    let report = lint_hir(
        r"
struct Foo {
    @range(0, 100)
    long field;
};
",
    );

    assert_eq!(report.errors.len(), 0); // range allows 2 unnamed args
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn invalid_multiple_unnamed_args() {
    let report = lint_hir(
        r"
annotation MyAnn {
    long value1;
    long value2;
};

@MyAnn(10, 20)
struct Foo {
    long field;
};
",
    );

    assert_eq!(report.errors.len(), 1);
    let error_output = format!("{:?}", report.errors[0]);
    assert!(error_output.contains("should be named"));
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn invalid_mixed_args() {
    let report = lint_hir(
        r"
annotation MyAnn {
    long value1;
    long value2;
    long value3;
};

@MyAnn(10, value2=20, 30)
struct Foo {
    long field;
};
",
    );

    assert_eq!(report.errors.len(), 1); // Mixed named/unnamed with multiple params
}

#[test]
#[ignore = "Annotation lowering not implemented"]
fn builtin_annotation_extra_args() {
    let report = lint_hir(
        r"
struct Foo {
    @min(0, 10, 20)  // min only takes 1 argument
    long field;
};
",
    );

    assert_eq!(report.errors.len(), 1);
}
