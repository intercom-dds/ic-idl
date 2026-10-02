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
use common::test_lint_hir;

#[test]
fn string_to_int() {
    let source = r#"
struct Bad {
    @default("hello")
    long my_int;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn int_to_string() {
    let source = r"
struct Bad {
    @default(123)
    string my_string;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn bool_to_int() {
    let source = r"
struct Bad {
    @default(true)
    long my_int;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_int_default() {
    let source = r"
struct Good {
    @default(42)
    long my_int;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_string_default() {
    let source = r#"
struct Good {
    @default("hello")
    string my_string;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_float_default() {
    let source = r"
struct Good {
    @default(3.14)
    float my_float;
    @default(3)
    double integer_float;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_bool_default() {
    let source = r"
struct Good {
    @default(TRUE)
    boolean my_bool;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_sequence_default() {
    let source = r"
struct Good {
    @default({1, 2, 3})
    sequence<long> my_seq;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_array_default() {
    let source = r"
struct Good {
    @default({1, 2, 3})
    long my_array[3];
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_struct_default() {
    let source = r"
struct Duration {
    long long sec;
    unsigned long long nanosec;
};

struct Good {
    @default({10, 30})
    Duration my_duration;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_derived_struct_default() {
    let source = r#"
struct Base {
    long id;
    string name;
};

struct Middle : Base {
    long count;
};

struct Derived : Middle {
    long total;
};

struct Good {
    @default({10, "base", 30, 40})
    Derived value;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn struct_field_type_mismatch() {
    let source = r#"
struct Duration {
    long long sec;
    unsigned long long nanosec;
};

struct Bad {
    @default({"ten", 30})
    Duration my_duration;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_enum_default() {
    let source = r"
enum Color { RED, GREEN, BLUE };

struct Good {
    @default(GREEN)
    Color my_color;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn wrong_enum_value() {
    let source = r"
enum Color { RED, GREEN, BLUE };
enum Size { SMALL, MEDIUM, LARGE };

struct Bad {
    @default(SMALL)
    Color my_color;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn sequence_element_type_mismatch() {
    let source = r#"
struct Bad {
    @default({"a", "b"})
    sequence<long> my_seq;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn invalid_enum_int_value() {
    let source = r"
enum Color { RED, GREEN };

struct Bad {
    @default(9)
    Color my_color;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_enum_int_value() {
    let source = r"
enum Color { RED, GREEN, BLUE };

struct Good {
    @default(1)
    Color my_color;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_enum_through_typedef() {
    let source = r"
enum Color { RED, GREEN, BLUE };
typedef Color MyColor;

struct Good {
    @default(GREEN)
    MyColor my_color;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn invalid_enum_through_typedef() {
    let source = r"
enum Color { RED, GREEN };
typedef Color MyColor;

struct Bad {
    @default(9)
    MyColor my_color;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_const_referencing_enum() {
    let source = r"
enum Color { RED, GREEN, BLUE };
const Color MY_COLOR = RED;

struct Good {
    @default(MY_COLOR)
    Color my_color;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn valid_char_through_typedef() {
    let source = r"
typedef char lower_case;

struct Good {
    @default('s')
    lower_case my_char;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn int_above_range() {
    let source = r"
struct Bad {
    @default(300)
    uint8 my_byte;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn int_below_range() {
    let source = r"
struct Bad {
    @default(-1)
    uint8 my_byte;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_int_at_range_bounds() {
    let source = r"
struct Good {
    @default(255)
    uint8 my_max;
    @default(-128)
    int8 my_min;
    @default(18446744073709551615)
    uint64 my_big;
};
";

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn int_above_range_on_every_target() {
    let source = r"
@default(300)
typedef uint8 Byte;

union BadUnion switch (@default(300) uint8) {
case 1:
    @default(300)
    uint8 my_byte;
};

exception BadException {
    @default(300)
    uint8 my_byte;
};

valuetype BadValue {
    @default(300)
    public uint8 my_byte;
};

@annotation bad_annotation {
    @default(300)
    uint8 my_byte;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_nested_const_numeric() {
    let source = r#"

typedef long Number;

struct Simple {
    Number a;
    Number b;
};

struct SimpleMultiple
{
    @default(20.5) double a;
    @default(20.5) float b;
    @default(10) short c;
    @default("Hello!!") string d;
    @optional float e;
    @optional string f;
};

struct NestedStructs
{
    Simple simple_struct;
    SimpleMultiple simple_multiple;
};

const Number SIMPLE1 = 10;
const Number SIMPLE2 = 20;

const sequence<sequence<octet>> BYTEARRAY_SEQUENCE_CONST = {{0x05,0x06},{0x07,0x08}};
const map<octet, octet> MAP_OCTET_CONST = {{0x01, 0x02}};

struct Good {
    @default({
        simple_struct = {SIMPLE1, SIMPLE2},
        simple_multiple = {1, 2, 3, "Test!", 5, "6"}
    })
    NestedStructs a;
    
    @default({SIMPLE1, SIMPLE2}) Simple b;
    
    @default(BYTEARRAY_SEQUENCE_CONST) sequence<sequence<octet>> s;

    @default(MAP_OCTET_CONST) map<octet, octet> m;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn typedef_const_in_collection_mismatch() {
    let source = r"
typedef long Number;

const sequence<Number> NUMBERS = {1};

struct Bad {
    @default(NUMBERS) sequence<short> values;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_bounded_string_const() {
    let source = r#"
const string<5> SHORT = "abc";
const wstring<5> WIDE_SHORT = L"abc";

struct Good {
    @default(SHORT) string a;
    @default(SHORT) string<5> b;
    @default(SHORT) string<10> c;
    @default(WIDE_SHORT) wstring d;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn const_sequence_primitive_mismatch() {
    let source = r"
const sequence<octet> BYTES = {1};

struct Bad {
    @default(BYTES) sequence<long> values;
};
";

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn bounded_string_const_too_long() {
    let source = r#"
const string<10> LONG = "abc";

struct Bad {
    @default(LONG) string<5> value;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn unbounded_string_const_on_bounded_member() {
    let source = r#"
const string TEXT = "abc";

struct Bad {
    @default(TEXT) string<5> value;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn wide_string_const_on_string_member() {
    let source = r#"
const wstring TEXT = L"abc";

struct Bad {
    @default(TEXT) string value;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn valid_newtype_const() {
    let source = r#"
@ext::newtype typedef string NewString;

const NewString NEW = "abc";

struct Good {
    @default(NEW) NewString a;
};
"#;

    let output = test_lint_hir(source);
    assert!(output.is_empty(), "Expected no errors, but got: {output}");
}

#[test]
fn newtype_on_underlying_type_const() {
    let source = r#"
@ext::newtype typedef string NewString;

const NewString NEW = "abc";

struct Bad {
    @default(NEW) string a;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}

#[test]
fn newtype_on_other_newtype_const() {
    let source = r#"
@ext::newtype typedef string NewString;
@ext::newtype typedef string NewString2;

const NewString NEW = "abc";

struct Bad {
    @default(NEW) NewString2 a;
};
"#;

    assert_snapshot!(test_lint_hir(source));
}
