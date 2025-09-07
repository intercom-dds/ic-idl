// Copyright 2025 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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

use ic_parse::from_file;
use ic_preproc::ProcArgs;
use ic_syntax::{Expr, Item, OpKind};
use ic_vfs::SourceMap;

#[test]
fn test_shift_operators_in_constants() {
    let mut vfs = SourceMap::default();
    let file = vfs.embed(
        r"
        const long LEFT_SHIFT = 1 << 4;
        const long RIGHT_SHIFT = 64 >> 2;
        const long COMPLEX = (1 << 8) + (256 >> 4);
    ",
    );

    let result = from_file(file, ProcArgs::default(), &mut vfs);
    assert!(result.errors.is_empty());

    // Verify the constants were parsed with shift operators
    let items = result.tree;
    assert_eq!(items.len(), 3);

    // Check that shift operators are present in the AST
    for item in &items {
        if let Item::ConstValue(c) = item {
            if let Expr::Binary(b) = &c.value {
                // Get the constant name from the declarator
                if let ic_syntax::Declarator::Simple(ident) = &c.decl {
                    match &ident.name[..] {
                        "LEFT_SHIFT" => assert_eq!(b.op.kind, OpKind::Lshift),
                        "RIGHT_SHIFT" => assert_eq!(b.op.kind, OpKind::Rshift),
                        _ => {}
                    }
                }
            }
        }
    }
}

#[test]
fn test_nested_templates() {
    let mut vfs = SourceMap::default();
    let file = vfs.embed(
        r"
        typedef sequence<sequence<string>> StringMatrix;
        typedef map<string, sequence<long>> StringToSeq;
        typedef sequence<map<long, sequence<octet>>> ComplexType;
    ",
    );

    let result = from_file(file, ProcArgs::default(), &mut vfs);
    assert!(result.errors.is_empty());

    // Verify all typedefs were parsed successfully
    assert_eq!(result.tree.len(), 3);

    for item in &result.tree {
        assert!(matches!(item, Item::AliasValue(_)));
    }
}

#[test]
fn test_shift_in_template_bounds() {
    let mut vfs = SourceMap::default();
    let file = vfs.embed(
        r"
        typedef sequence<long, 1 << 10> KB_Array;
        typedef sequence<octet, 256 >> 2> SixtyFour_Array;
    ",
    );

    let result = from_file(file, ProcArgs::default(), &mut vfs);
    assert!(result.errors.is_empty());
    assert_eq!(result.tree.len(), 2);
}

#[test]
fn test_ambiguous_cases() {
    let mut vfs = SourceMap::default();

    // Test case where >> could be ambiguous
    let file = vfs.embed(
        r"
        // In expression context, >> is shift
        const long SHIFT = 1024 >> 2;
        
        // In template context, >> is two separate >
        typedef sequence<sequence<long>> Matrix;
        
        // Mixed: shift in template parameter
        typedef sequence<octet, 1024 >> 2> Array;
    ",
    );

    let result = from_file(file, ProcArgs::default(), &mut vfs);
    assert!(result.errors.is_empty());
    assert_eq!(result.tree.len(), 3);
}
