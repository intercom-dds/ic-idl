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
fn valid_range() {
    let source = r"
module MyModule {
    struct Temperature {
        @range(min=-273, max=1000) long celsius;
    };
};
";

    let output = common::lint_hir(source);
    assert!(output.warnings.is_empty());
    assert!(output.errors.is_empty());
}

#[test]
fn valid_min_max() {
    let source = r"
module MyModule {
    struct Age {
        @min(0) @max(150) short years;
    };
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for valid min/max, but got: {output}"
    );
}

#[test]
fn inverted_range() {
    let source = r"
module MyModule {
    struct BadRange {
        @range(min=100, max=0) long value;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn inverted_min_max() {
    let source = r"
module MyModule {
    struct BadMinMax {
        @min(100) @max(50) long value;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn malformed_range() {
    let source = r#"
module MyModule {
    struct MalformedRange {
        @range(min="not a number") long value;
    };
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn malformed_min() {
    let source = r#"
module MyModule {
    struct MalformedMin {
        @min("not a number") long value;
    };
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_with_only_min() {
    let source = r"
module MyModule {
    struct RangeOnlyMin {
        @range(min=0) unsigned long count;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_with_only_max() {
    let source = r"
module MyModule {
    struct RangeOnlyMax {
        @range(max=100) long percentage;
    };
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn multiple_ranges() {
    let source = r"
module MyModule {
    struct MultipleRanges {
        @range(min=0, max=100) @range(min=10, max=90) long value;
    };
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for multiple ranges, but got: {output}"
    );
}

#[test]
fn range_on_struct() {
    let source = r"
module MyModule {
    @range(min=0, max=100)
    struct Percentage {
        long value;
    };
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for range on struct, but got: {output}"
    );
}

#[test]
fn range_on_const() {
    let source = r"
module MyModule {
    @range(min=0, max=100)
    const long MAX_PERCENTAGE = 100;
};
";

    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for range on const, but got: {output}"
    );
}

#[test]
fn max_exceeds_type_bounds() {
    let source = r"
module MyModule {
    struct BadMax {
        @max(299) octet value;
    };
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn min_below_type_bounds() {
    let source = r"
module MyModule {
    struct BadMin {
        @min(-1) char value;
    };
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn range_exceeds_type_bounds() {
    let source = r"
module MyModule {
    struct BadRange {
        @range(min=-1, max=300) octet value;
    };
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_type_bounds() {
    let source = r"
module MyModule {
    struct GoodBounds {
        @min(0) @max(255) octet byte_value;
        @range(min=0, max=127) char char_value;
        @range(min=-32768, max=32767) short short_value;
        @range(min=0, max=65535) unsigned short ushort_value;
    };
};
";

    let output = common::lint_hir(source);
    assert!(output.warnings.is_empty());
    assert!(output.errors.is_empty());
}

#[test]
fn char_exceeds_max_bounds() {
    let source = r"
module MyModule {
    struct BadChar {
        @max(200) char value;
    };
};
";
    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn test_positive_min_works() {
    let source = r"
module MyModule {
    struct TestPositive {
        @min(10) octet value;
    };
};
";
    let output = test_lint_hir(source);
    assert!(
        output.is_empty(),
        "Expected no warnings for positive min, but got: {output}"
    );
}
