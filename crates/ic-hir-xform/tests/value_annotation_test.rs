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

use ic_hir::hir::{DefKind, Numeric};
use ic_preproc::ProcArgs;
use ic_vfs::SourceMap;

#[test]
fn test_value_annotation_transform_is_noop() {
    // The value annotation transform is now a no-op since enum values
    // are handled during HIR lowering with the new constant-based enum structure.
    // This test verifies that the transform doesn't break anything.
    
    let input = r"
        enum Status {
            OK = 200,
            NOT_FOUND = 404,
            ERROR = 500
        };
    ";

    // Parse the input
    let mut source_map = SourceMap::default();
    let file_id = source_map.embed(input);
    let parsed = ic_parse::from_file(file_id, ProcArgs::default(), &mut source_map);

    // Lower to HIR
    let hir = ic_hir::from_ast(ic_hir::AstInput::User(parsed.tree));

    // Apply the transformation (which should be a no-op)
    let transformed = ic_hir_xform::value_annotation::transform(hir);

    // Find the Status enum
    let status_enum = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Status")
        .expect("Status enum not found");

    if let DefKind::Enum(enum_ty) = &status_enum.1.kind {
        assert_eq!(enum_ty.fields.len(), 3);
        
        // Check that explicit values are preserved
        let field0 = transformed.context.definitions.get(enum_ty.fields[0]);
        assert_eq!(field0.ident.name, "OK");
        if let DefKind::Const(const_ty) = &field0.kind {
            assert_eq!(const_ty.value, Numeric::Int32(200));
        }
        
        let field1 = transformed.context.definitions.get(enum_ty.fields[1]);
        assert_eq!(field1.ident.name, "NOT_FOUND");
        if let DefKind::Const(const_ty) = &field1.kind {
            assert_eq!(const_ty.value, Numeric::Int32(404));
        }
        
        let field2 = transformed.context.definitions.get(enum_ty.fields[2]);
        assert_eq!(field2.ident.name, "ERROR");
        if let DefKind::Const(const_ty) = &field2.kind {
            assert_eq!(const_ty.value, Numeric::Int32(500));
        }
    } else {
        panic!("Status should be an enum");
    }
}

#[test]
fn test_enum_auto_increment() {
    // Test that enum values auto-increment correctly
    let input = r"
        enum Numbers {
            ZERO,
            ONE,
            TEN = 10,
            ELEVEN,
            TWENTY = 20
        };
    ";

    // Parse the input
    let mut source_map = SourceMap::default();
    let file_id = source_map.embed(input);
    let parsed = ic_parse::from_file(file_id, ProcArgs::default(), &mut source_map);

    // Lower to HIR
    let hir = ic_hir::from_ast(ic_hir::AstInput::User(parsed.tree));

    // Apply the transformation (which should be a no-op)
    let transformed = ic_hir_xform::value_annotation::transform(hir);

    // Find the Numbers enum
    let numbers_enum = transformed
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Numbers")
        .expect("Numbers enum not found");

    if let DefKind::Enum(enum_ty) = &numbers_enum.1.kind {
        assert_eq!(enum_ty.fields.len(), 5);
        
        let field0 = transformed.context.definitions.get(enum_ty.fields[0]);
        if let DefKind::Const(const_ty) = &field0.kind {
            assert_eq!(const_ty.value, Numeric::Int32(0)); // ZERO
        }
        
        let field1 = transformed.context.definitions.get(enum_ty.fields[1]);
        if let DefKind::Const(const_ty) = &field1.kind {
            assert_eq!(const_ty.value, Numeric::Int32(1)); // ONE
        }
        
        let field2 = transformed.context.definitions.get(enum_ty.fields[2]);
        if let DefKind::Const(const_ty) = &field2.kind {
            assert_eq!(const_ty.value, Numeric::Int32(10)); // TEN
        }
        
        let field3 = transformed.context.definitions.get(enum_ty.fields[3]);
        if let DefKind::Const(const_ty) = &field3.kind {
            assert_eq!(const_ty.value, Numeric::Int32(11)); // ELEVEN
        }
        
        let field4 = transformed.context.definitions.get(enum_ty.fields[4]);
        if let DefKind::Const(const_ty) = &field4.kind {
            assert_eq!(const_ty.value, Numeric::Int32(20)); // TWENTY
        }
    } else {
        panic!("Numbers should be an enum");
    }
}