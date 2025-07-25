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
fn test_annotation_scope_enum_resolution() {
    let idl = r"
        @annotation FooBar {
            enum MyEnum { ZERO, ONE, TWO };
            MyEnum value;
        };
        
        @FooBar(ONE)
        struct Asd {};
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the struct definition
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Asd")
        .expect("Asd not found");

    // Check it has the FooBar annotation
    assert_eq!(struct_def.1.annotations.len(), 1);
    let ann = &struct_def.1.annotations[0];
    assert_eq!(ann.ident.name, "FooBar");

    // Check the annotation argument
    assert_eq!(ann.args.len(), 1);
    let arg = &ann.args[0];
    assert_eq!(arg.ident.name, "value");

    // The value should be a reference to the ONE constant from the annotation's scope
    match &arg.value {
        Numeric::Const(def_id) => {
            let const_def = result.context.definitions.get(*def_id);
            assert_eq!(const_def.ident.name, "ONE");
            if let DefKind::Const(const_ty) = &const_def.kind {
                assert_eq!(const_ty.value, Numeric::Int32(1));
            } else {
                panic!("Expected const definition");
            }
        }
        other => panic!("Expected Numeric::Const, got {other:?}"),
    }
}

#[test]
fn test_annotation_scope_precedence() {
    let idl = r"
        enum OuterEnum { FIRST, SECOND };
        
        @annotation MyAnn {
            enum InnerEnum { FIRST, THIRD };
            InnerEnum value;
        };
        
        @MyAnn(FIRST)
        struct Test {};
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the struct
    let struct_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Test")
        .expect("Test not found");

    let ann = &struct_def.1.annotations[0];
    let arg = &ann.args[0];

    // Should resolve to InnerEnum::FIRST (from annotation scope), not OuterEnum::FIRST
    match &arg.value {
        Numeric::Const(def_id) => {
            let const_def = result.context.definitions.get(*def_id);
            assert_eq!(const_def.ident.name, "FIRST");

            // Check it's from the inner enum by verifying its parent
            if let Some(parent_id) = const_def.parent {
                let parent_def = result.context.definitions.get(parent_id);
                assert_eq!(parent_def.ident.name, "InnerEnum");
            } else {
                panic!("Expected const to have parent enum");
            }
        }
        other => panic!("Expected Numeric::Const, got {other:?}"),
    }
}

#[test]
fn test_annotation_scope_fallback() {
    let idl = r"
        enum GlobalEnum { VALUE1, VALUE2 };
        
        @annotation SimpleAnn {
            GlobalEnum field;
        };
        
        @SimpleAnn(VALUE2)
        interface Test {};
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());

    // Find the interface
    let interface_def = result
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Test")
        .expect("Test not found");

    let ann = &interface_def.1.annotations[0];
    let arg = &ann.args[0];

    // Should resolve to GlobalEnum::VALUE2 (fallback to outer scope)
    match &arg.value {
        Numeric::Const(def_id) => {
            let const_def = result.context.definitions.get(*def_id);
            assert_eq!(const_def.ident.name, "VALUE2");
            if let DefKind::Const(const_ty) = &const_def.kind {
                assert_eq!(const_ty.value, Numeric::Int32(1));
            } else {
                panic!("Expected const definition");
            }
        }
        other => panic!("Expected Numeric::Const, got {other:?}"),
    }
}
