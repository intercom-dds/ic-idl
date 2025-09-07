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

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn test_annotation_with_enum_argument() {
    let idl = r"
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

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the struct definition
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TestStruct")
        .expect("TestStruct not found");

    // Check it has the FooBar annotation
    assert_eq!(struct_def.1.annotations.len(), 1);
    let ann = &struct_def.1.annotations[0];
    assert_eq!(ann.ident.name, "FooBar");

    // Check the annotation argument
    assert_eq!(ann.args.len(), 1);
    let arg = &ann.args[0];
    assert_eq!(arg.ident.name, ""); // Positional argument has empty name

    // The value should be a reference to the ONE constant
    match &arg.value {
        Numeric::Const(def_id) => {
            let const_def = result.context.definitions.get(*def_id);
            assert_eq!(const_def.ident.name, "ONE");
            if let DefKind::Const(const_ty) = &const_def.kind {
                assert_eq!(const_ty.value, Numeric::Int32(0));
            } else {
                panic!("Expected const definition");
            }
        }
        other => panic!("Expected Numeric::Const, got {other:?}"),
    }
}

#[test]
fn test_annotation_with_scoped_enum_argument() {
    let idl = r"
        module foo {
            enum Status {
                OK = 200,
                NOT_FOUND = 404,
                ERROR = 500
            };
        };
        
        @annotation StatusAnnotation {
            foo::Status code;
        };
        
        @StatusAnnotation(foo::Status::NOT_FOUND)
        exception NotFoundException {
            string message;
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the exception definition
    let exception_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "NotFoundException")
        .expect("NotFoundException not found");

    // Check it has the StatusAnnotation annotation
    assert_eq!(exception_def.1.annotations.len(), 1);
    let ann = &exception_def.1.annotations[0];
    assert_eq!(ann.ident.name, "StatusAnnotation");

    // Check the annotation argument
    assert_eq!(ann.args.len(), 1);
    let arg = &ann.args[0];
    assert_eq!(arg.ident.name, ""); // Positional argument has empty name

    // The value should be a reference to the NOT_FOUND constant
    match &arg.value {
        Numeric::Const(def_id) => {
            let const_def = result.context.definitions.get(*def_id);
            assert_eq!(const_def.ident.name, "NOT_FOUND");
            if let DefKind::Const(const_ty) = &const_def.kind {
                assert_eq!(const_ty.value, Numeric::Int32(404));
            } else {
                panic!("Expected const definition");
            }
        }
        other => panic!("Expected Numeric::Const, got {other:?}"),
    }
}

#[test]
#[ignore = "Positional argument name mapping not yet supported"]
fn test_annotation_with_unscoped_enum_argument() {
    let idl = r"
        enum Color {
            RED,
            GREEN,
            BLUE
        };
        
        @annotation ColorAnnotation {
            Color value;
        };
        
        @ColorAnnotation(GREEN)
        interface ColoredInterface {
            void paint();
        };
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the interface definition
    let interface_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ColoredInterface")
        .expect("ColoredInterface not found");

    // Check it has the ColorAnnotation annotation
    assert_eq!(interface_def.1.annotations.len(), 1);
    let ann = &interface_def.1.annotations[0];
    assert_eq!(ann.ident.name, "ColorAnnotation");

    // Check the annotation argument
    assert_eq!(ann.args.len(), 1);
    let arg = &ann.args[0];
    assert_eq!(arg.ident.name, "value");

    // The value should be a reference to the GREEN constant
    match &arg.value {
        Numeric::Const(def_id) => {
            let const_def = result.context.definitions.get(*def_id);
            assert_eq!(const_def.ident.name, "GREEN");
            if let DefKind::Const(const_ty) = &const_def.kind {
                assert_eq!(const_ty.value, Numeric::Int32(1)); // GREEN is second, so value is 1
            } else {
                panic!("Expected const definition");
            }
        }
        other => panic!("Expected Numeric::Const, got {other:?}"),
    }
}
