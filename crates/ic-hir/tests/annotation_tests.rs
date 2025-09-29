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

//! Comprehensive tests for annotation functionality including:
//! - Basic annotation application
//! - Enum and constant resolution in annotation scope
//! - Multi-parameter annotations with defaults
//! - Named vs positional arguments
//! - Error cases and validation

mod common;

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn test_basic_annotation() {
    let input = r#"
        @annotation my_doc {
            string value;
        };

    @my_doc("This is a test struct")
        struct TestStruct {
            long field;
        };
    "#;

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_annotation_scope_enum_resolution() {
    let input = r"
        @annotation FooBar {
            enum MyEnum { ZERO, ONE, TWO };
            MyEnum value;
        };

    @FooBar(ONE)
        struct Asd {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_annotation_enum_scope_with_local_conflict() {
    let input = r"
        @annotation FooBar {
            enum MyEnum { ZERO, ONE, TWO };
            MyEnum value;
        };

    // Local enum with different values
    enum MyEnum { ALPHA, BETA, ONE };

    // Should resolve to annotation's ONE, not local ONE
    @FooBar(ONE)
        struct Asd {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_annotation_with_external_enum() {
    let input = r"
        enum MyEnum {
            ONE,
            TWO,
            THREE
        };

    @annotation FooBar {
        MyEnum value;
    };

    @FooBar(ONE)
        struct TestStruct {
            string field;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Find the struct definition
    let struct_def_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            matches!(&def.kind, DefKind::Struct(_)) && def.ident.name == "TestStruct"
        })
        .expect("TestStruct not found");
    let struct_def = result.context.definitions.get(*struct_def_id);

    // Verify the annotation argument
    assert_eq!(struct_def.annotations.len(), 1);
    let ann = &struct_def.annotations[0];
    assert_eq!(ann.ident.name, "FooBar");
    assert_eq!(ann.args.len(), 1);

    // The value should be a reference to the ONE enumerator
    if let Numeric::Const(enum_ref) = &ann.args[0].value {
        let enum_def = result.context.definitions.get(enum_ref);
        assert_eq!(enum_def.ident.name, "ONE");
    } else {
        panic!("Expected Numeric::Const for enum value");
    }
}

#[test]
fn test_annotation_const_scope_resolution() {
    let input = r#"
        @annotation Config {
            const long DEFAULT_TIMEOUT = 30;
            const string DEFAULT_NAME = "default";
            long timeout default DEFAULT_TIMEOUT;
            string name default DEFAULT_NAME;
        };

        // Should resolve DEFAULT_TIMEOUT from annotation scope
        @Config(DEFAULT_TIMEOUT, DEFAULT_NAME)
        struct ConfiguredItem {};
    "#;

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_annotation_const_arithmetic_in_scope() {
    let input = r"
        @annotation MyRange {
            const long MIN = 0;
            const long MAX = 100;
            const long STEP = 10;
            long min default MIN;
            long max default MAX;
        };

        // Should resolve MAX from annotation scope and do arithmetic
        @MyRange(MIN + STEP, MAX - STEP)
        struct RangedValue {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_annotation_scope_fallback_to_local() {
    let input = r"
        @annotation FooBar {
            enum MyEnum { ZERO, ONE, TWO };
            MyEnum value;
        };

        const long THREE = 3;

        // Should fall back to local scope for THREE
        @FooBar(THREE)
        struct Asd {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_multi_param_annotation_warning() {
    let input = r"
        @annotation my_range {
            long min;
            long max;
        };

        // This should produce a warning
        @my_range(0, 10)
        struct BadRange {
            long value;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_multi_param_annotation_named_ok() {
    let input = r"
        @annotation my_range {
            long min;
            long max;
        };

        // This is correct - named arguments
        @my_range(min=0, max=10)
        struct GoodRange {
            long value;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_single_param_annotation_positional_ok() {
    let input = r"
        @annotation my_optional {
            boolean value;
        };

        @my_optional(false)
        struct OptionalTest {
            long value;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_mixed_named_positional_warning() {
    let input = r"
        @annotation test {
            long a;
            long b;
            long c;
        };

        // Mixed named and positional - should warn
        @test(5, b=10, c=15)
        struct Mixed {
            long value;
        };
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_annotation_with_defaults() {
    let input = r"
        @annotation config {
            long timeout;
            boolean retry default true;
        };

        // Single positional arg is OK when there's only one param without default
        @config(30)
        struct Config {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

#[test]
fn test_annotation_defaults_applied() {
    let input = r#"
        @annotation Config {
            long timeout default 30;
            boolean retry default true;
            string name default "default";
        };

        // Should use all defaults
        @Config()
        struct UseAllDefaults {};

        // Should override timeout but use other defaults
        @Config(60)
        struct OverrideTimeout {};

        // Should override all with named args
        @Config(timeout=90, retry=false, name="custom")
        struct OverrideAll {};
    "#;

    let (result, _, _) = common::parse_and_resolve(input);
    assert!(result.errors.is_empty());

    // Test UseAllDefaults
    let use_defaults_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            def.ident.name == "UseAllDefaults"
        })
        .expect("UseAllDefaults not found");
    let use_defaults = result.context.definitions.get(*use_defaults_id);

    assert_eq!(use_defaults.annotations.len(), 1);
    let ann = &use_defaults.annotations[0];
    assert_eq!(ann.args.len(), 3);

    // Test OverrideTimeout
    let override_timeout_id = result
        .order
        .iter()
        .find(|&&def_id| {
            let def = result.context.definitions.get(def_id);
            def.ident.name == "OverrideTimeout"
        })
        .expect("OverrideTimeout not found");
    let override_timeout = result.context.definitions.get(*override_timeout_id);

    assert_eq!(override_timeout.annotations.len(), 1);
    let ann = &override_timeout.annotations[0];
    assert_eq!(ann.args.len(), 3);
}

#[test]
fn test_annotation_in_module_scope() {
    let input = r"
        module foo {
            @annotation Bar {
                enum Level { LOW, MEDIUM, HIGH };
                Level level default MEDIUM;
            };
        };

        // Should resolve Level::HIGH from annotation scope
        @foo::Bar(HIGH)
        struct TestStruct {};
    ";

    let diagnostics = common::compile_idl_with_warnings(input);
    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics but got: {diagnostics}",
    );
}

