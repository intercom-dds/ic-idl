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

// Test constant references to enumerators

use ic_vfs::SourceMap;

#[test]
fn test_const_enum_reference() {
    let input = r"
        enum MyEnum {
            ZERO,
            ONE,
            TWO = 5,
            THREE
        };
        
        const MyEnum MY_CONST = MyEnum::ZERO;
        const int32 INT_CONST = MyEnum::TWO;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );

    // Verify the constant values were resolved correctly
    let mut found_my_const = false;
    let mut found_int_const = false;

    for (_, def) in &result.context.definitions {
        if def.ident.name == "MY_CONST" {
            if let ic_hir::hir::DefKind::Const(const_ty) = &def.kind {
                if let ic_hir::hir::Numeric::Int32(val) = const_ty.value {
                    assert_eq!(val, 0, "MY_CONST should have value 0 (ZERO)");
                    found_my_const = true;
                }
            }
        } else if def.ident.name == "INT_CONST" {
            if let ic_hir::hir::DefKind::Const(const_ty) = &def.kind {
                if let ic_hir::hir::Numeric::Int32(val) = const_ty.value {
                    assert_eq!(val, 5, "INT_CONST should have value 5 (TWO)");
                    found_int_const = true;
                }
            }
        }
    }

    assert!(found_my_const, "MY_CONST not found or has wrong type");
    assert!(found_int_const, "INT_CONST not found or has wrong type");
}

#[test]
fn test_const_ref_to_const() {
    let input = r"
        const int32 BASE = 100;
        const int32 DERIVED = BASE + 50;
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have no errors
    assert!(
        result.errors.is_empty(),
        "Unexpected errors: {:?}",
        result.errors
    );

    // Verify DERIVED has the correct value
    for (_, def) in &result.context.definitions {
        if def.ident.name == "DERIVED" {
            if let ic_hir::hir::DefKind::Const(const_ty) = &def.kind {
                if let ic_hir::hir::Numeric::Int32(val) = const_ty.value {
                    assert_eq!(val, 150, "DERIVED should have value 150 (BASE + 50)");
                    return;
                }
            }
        }
    }

    panic!("DERIVED constant not found or has wrong type");
}

#[test]
fn test_undefined_enum_field() {
    let mut source_map = SourceMap::default();
    let input = r"
        enum MyEnum {
            ZERO,
            ONE
        };
        
        const int32 BAD = MyEnum::UNDEFINED;
    ";

    let file = source_map.embed_with_name("test.idl", input);
    let parsed = ic_parse::from_file(file, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have an error about undefined enum field
    assert!(
        !result.errors.is_empty(),
        "Expected error for undefined enum field"
    );

    // Snapshot test the error message
    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_undefined_variable() {
    let mut source_map = SourceMap::default();
    let input = r"
        const int32 BAD = UNDEFINED_VAR;
    ";

    let file = source_map.embed_with_name("test.idl", input);
    let parsed = ic_parse::from_file(file, ic_preproc::ProcArgs::default(), &mut source_map);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);

    // Should have an error about undefined variable
    assert!(
        !result.errors.is_empty(),
        "Expected error for undefined variable"
    );

    // Snapshot test the error message
    let mut output = String::new();
    for error in &result.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}
