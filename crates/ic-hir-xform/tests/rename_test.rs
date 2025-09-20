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

mod common;

use ic_emit::case::Case;
use ic_hir::hir::{DefKind, EnumTy, InterfaceTy, StructTy, UnionTy, ValueTy};
use ic_hir::visit::Visitor;
use ic_hir_xform::{Target, rename, strip_common_suffixes};

/// Helper to create Rust naming convention target
fn rust_target() -> Target {
    Target {
        struct_type: Some(Case::Pascal),
        union_type: Some(Case::Pascal),
        enum_type: Some(Case::Pascal),
        interface: Some(Case::Pascal),
        valuetype: Some(Case::Pascal),
        alias: Some(Case::Pascal),
        bitmask: Some(Case::Pascal),
        bitset: Some(Case::Pascal),
        exception: Some(Case::Pascal),
        annotation: Some(Case::Pascal),
        member: Some(Case::Snake),
        variant: Some(Case::Pascal),
        enumerator: Some(Case::Pascal),
        bit_flag: Some(Case::Snake),
        bitset_field: Some(Case::Snake),
        constant: Some(Case::Snake),
        module: Some(Case::Snake),
        operation: Some(Case::Snake),
        attribute: Some(Case::Snake),
        parameter: Some(Case::Snake),
        annotation_param: Some(Case::Snake),
        name_preprocessor: Some(strip_common_suffixes),
    }
}

/// Helper to create Python naming convention target
fn python_target() -> Target {
    Target {
        struct_type: Some(Case::Pascal),
        union_type: Some(Case::Pascal),
        enum_type: Some(Case::Pascal),
        interface: Some(Case::Pascal),
        valuetype: Some(Case::Pascal),
        alias: Some(Case::Pascal),
        bitmask: Some(Case::Pascal),
        bitset: Some(Case::Pascal),
        exception: Some(Case::Pascal),
        annotation: Some(Case::Pascal),
        member: Some(Case::Snake),
        variant: Some(Case::Snake),
        enumerator: Some(Case::Snake),
        bit_flag: Some(Case::Snake),
        bitset_field: Some(Case::Snake),
        constant: Some(Case::Snake),
        module: Some(Case::Snake),
        operation: Some(Case::Snake),
        attribute: Some(Case::Snake),
        parameter: Some(Case::Snake),
        annotation_param: Some(Case::Snake),
        name_preprocessor: Some(strip_common_suffixes),
    }
}

/// Helper visitor to verify renamed identifiers
struct RenameVerifier {
    errors: Vec<String>,
}

impl<'a> Visitor<'a> for RenameVerifier {
    fn context(&self) -> &'a ic_hir::Context {
        unreachable!("not used in tests")
    }

    fn visit_def(&mut self, def: &'a ic_hir::hir::Def) {
        // Visit the definition content
        match &def.kind {
            DefKind::Struct(s) => self.visit_struct(def, s),
            DefKind::Union(u) => self.visit_union(def, u),
            DefKind::Enum(e) => self.visit_enum(def, e),
            DefKind::Interface(i) => self.visit_interface(def, i),
            DefKind::Valuetype(v) => self.visit_valuetype(def, v),
            DefKind::Module(_) => self.visit_module(def),
            DefKind::Const(_) => self.visit_const(def),
            _ => {}
        }
    }
}

impl RenameVerifier {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn assert_case(&mut self, name: &str, expected_case: Case, item_type: &str) {
        if !self.check_case(name, expected_case) {
            self.errors.push(format!(
                "{} '{}' does not follow {:?} convention",
                item_type, name, expected_case
            ));
        }
    }

    fn check_case(&self, name: &str, case: Case) -> bool {
        match case {
            Case::Snake => name
                .chars()
                .all(|c| c.is_lowercase() || c.is_numeric() || c == '_'),
            Case::Pascal => name.chars().next().map_or(false, |c| c.is_uppercase()),
            Case::Camel => name.chars().next().map_or(false, |c| c.is_lowercase()),
            Case::Kebab => name
                .chars()
                .all(|c| c.is_lowercase() || c.is_numeric() || c == '-'),
        }
    }

    fn visit_struct(&mut self, def: &ic_hir::hir::Def, s: &StructTy) {
        self.assert_case(&def.ident.name, Case::Pascal, "Struct");
        for member in &s.members {
            self.assert_case(&member.ident.name, Case::Snake, "Struct member");
        }
    }

    fn visit_union(&mut self, def: &ic_hir::hir::Def, u: &UnionTy) {
        self.assert_case(&def.ident.name, Case::Pascal, "Union");
        for variant in &u.variants {
            self.assert_case(&variant.ident.name, Case::Pascal, "Union variant");
        }
    }

    fn visit_enum(&mut self, def: &ic_hir::hir::Def, _e: &EnumTy) {
        self.assert_case(&def.ident.name, Case::Pascal, "Enum");
        // Note: enum constants are separate definitions, checked in visit_const
    }

    fn visit_interface(&mut self, def: &ic_hir::hir::Def, i: &InterfaceTy) {
        self.assert_case(&def.ident.name, Case::Pascal, "Interface");
        for proto in &i.prototypes {
            self.assert_case(&proto.ident.name, Case::Snake, "Interface operation");
            for param in &proto.params {
                self.assert_case(&param.ident.name, Case::Snake, "Operation parameter");
            }
        }
        for attr in &i.attributes {
            self.assert_case(&attr.ident.name, Case::Snake, "Interface attribute");
        }
    }

    fn visit_valuetype(&mut self, def: &ic_hir::hir::Def, v: &ValueTy) {
        self.assert_case(&def.ident.name, Case::Pascal, "Valuetype");
        for member in &v.members {
            self.assert_case(&member.ident.name, Case::Snake, "Valuetype member");
        }
        for proto in &v.prototypes {
            self.assert_case(&proto.ident.name, Case::Snake, "Valuetype operation");
        }
        for attr in &v.attributes {
            self.assert_case(&attr.ident.name, Case::Snake, "Valuetype attribute");
        }
    }

    fn visit_module(&mut self, def: &ic_hir::hir::Def) {
        self.assert_case(&def.ident.name, Case::Snake, "Module");
    }

    fn visit_const(&mut self, def: &ic_hir::hir::Def) {
        // Check if this is an enum constant by looking at the parent
        if let Some(_parent_id) = def.parent {
            // For enum constants in Rust style, we use Pascal case
            self.assert_case(&def.ident.name, Case::Pascal, "Enum constant");
        } else {
            self.assert_case(&def.ident.name, Case::Snake, "Constant");
        }
    }
}

#[test]
fn test_rust_naming_conventions() {
    let idl = r#"
        module test_module {
            struct MyStruct {
                long myField;
                string anotherField;
            };
            
            enum MyEnum {
                firstValue,
                secondValue,
                THIRD_VALUE
            };
            
            union MyUnion switch(long) {
                case 1: long intValue;
                case 2: string stringValue;
                default: boolean defaultValue;
            };
            
            interface MyInterface {
                void doSomething(in long inputParam, out string outputParam);
                attribute long myAttribute;
            };
            
            const long MY_CONSTANT = 42;
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    // Apply Rust naming conventions
    let renamed = rename::transform(hir, rust_target());

    // Verify the renaming
    let mut verifier = RenameVerifier::new();
    for def in renamed.iter() {
        verifier.visit_def(def);
    }

    if !verifier.errors.is_empty() {
        panic!(
            "Naming convention violations:\n{}",
            verifier.errors.join("\n")
        );
    }
}

#[test]
fn test_python_naming_conventions() {
    let idl = r#"
        struct myStruct {
            long MyField;
            string AnotherField;
        };
        
        enum myEnum {
            FirstValue,
            SecondValue
        };
        
        union myUnion switch(long) {
            case 1: long IntValue;
            case 2: string StringValue;
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    // Apply Python naming conventions
    let renamed = rename::transform(hir, python_target());

    // Verify Python conventions
    for def in renamed.iter() {
        match &def.kind {
            DefKind::Struct(_) | DefKind::Enum(_) | DefKind::Union(_) => {
                // Classes should be PascalCase
                assert!(def.ident.name.chars().next().unwrap().is_uppercase());
            }
            DefKind::Const(_) => {
                // Constants should be snake_case in Python (not UPPER_SNAKE)
                assert!(
                    def.ident
                        .name
                        .chars()
                        .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test_custom_naming_target() {
    let idl = r#"
        struct test_struct {
            long test_field;
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    // Create custom target - all kebab-case
    let target = Target {
        struct_type: Some(Case::Kebab),
        member: Some(Case::Kebab),
        ..Default::default()
    };

    let renamed = rename::transform(hir, target);

    // Verify kebab-case
    for def in renamed.iter() {
        if let DefKind::Struct(s) = &def.kind {
            assert_eq!(def.ident.name, "test-struct");
            assert_eq!(s.members[0].ident.name, "test-field");
        }
    }
}

#[test]
fn test_preserve_unchanged() {
    let idl = r#"
        struct AlreadyPascal {
            long already_snake;
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    let renamed = rename::transform(hir, rust_target());

    // Verify names are preserved when already correct
    for def in renamed.iter() {
        if let DefKind::Struct(s) = &def.kind {
            assert_eq!(def.ident.name, "AlreadyPascal");
            assert_eq!(s.members[0].ident.name, "already_snake");
        }
    }
}

#[test]
fn test_interface_members() {
    let idl = r#"
        interface testInterface {
            void DoOperation(in long InputParam);
            readonly attribute long SomeAttribute;
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    let renamed = rename::transform(hir, rust_target());

    for def in renamed.iter() {
        if let DefKind::Interface(i) = &def.kind {
            assert_eq!(def.ident.name, "TestInterface");
            assert_eq!(i.prototypes[0].ident.name, "do_operation");
            assert_eq!(i.prototypes[0].params[0].ident.name, "input_param");
            assert_eq!(i.attributes[0].ident.name, "some_attribute");
        }
    }
}

#[test]
fn test_no_suffix_stripping() {
    let idl = r#"
        struct property_t {};
        enum my_enum_e {
            value_1,
            value_2
        };
    "#;

    let hir = common::parse_and_resolve(idl);

    // Create target with no preprocessing (default)
    let target = Target {
        struct_type: Some(Case::Pascal),
        enum_type: Some(Case::Pascal),
        enumerator: Some(Case::Pascal),
        ..Default::default()
    };

    let renamed = rename::transform(hir, target);

    for def in renamed.iter() {
        match &def.kind {
            DefKind::Struct(_) => {
                // Without preprocessing, _t suffix is preserved in PascalCase
                assert_eq!(def.ident.name, "PropertyT");
            }
            DefKind::Enum(_) => {
                // Without preprocessing, _e suffix is preserved in PascalCase
                assert_eq!(def.ident.name, "MyEnumE");
            }
            _ => {}
        }
    }
}

#[test]
fn test_custom_preprocessor() {
    let idl = r#"
        struct foo_bar_baz {};
    "#;

    let hir = common::parse_and_resolve(idl);

    // Custom preprocessor that removes "foo_" prefix
    fn remove_foo_prefix(name: &str) -> String {
        if name.starts_with("foo_") {
            name[4..].to_string()
        } else {
            name.to_string()
        }
    }

    let target = Target {
        struct_type: Some(Case::Pascal),
        name_preprocessor: Some(remove_foo_prefix),
        ..Default::default()
    };

    let renamed = rename::transform(hir, target);

    for def in renamed.iter() {
        if let DefKind::Struct(_) = &def.kind {
            // Custom preprocessor removes foo_ prefix, then converts to PascalCase
            assert_eq!(def.ident.name, "BarBaz");
        }
    }
}
