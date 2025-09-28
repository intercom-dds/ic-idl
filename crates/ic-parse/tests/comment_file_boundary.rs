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

use std::path::Path;

use ic_parse::from_path;
use ic_preproc::ProcArgs;
use ic_syntax::{AnnotationAppl, Expr, Item, Literal, LiteralValue};
use ic_vfs::SourceMap;

fn get_doc_strings(annotations: &[AnnotationAppl]) -> Vec<String> {
    annotations
        .iter()
        .filter(|a| a.ident.segments[0].name == "doc")
        .map(|a| {
            if let Some(arg) = a.args.first() {
                if let Expr::Literal(Literal {
                    value: LiteralValue::String(s),
                    ..
                }) = &arg.value
                {
                    s.clone()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        })
        .collect()
}

#[test]
fn test_fileid_bug_with_includes() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_includes");
    let main_path = test_dir.join("main.idl");

    // Parse with includes
    let mut vfs = SourceMap::default();
    let args = ProcArgs::default().include(&test_dir);
    let result = from_path(&main_path, args, &mut vfs).unwrap();

    // Verify parsing succeeded
    assert!(result.errors.is_empty());
    assert_eq!(result.tree.len(), 1);

    if let Item::StructValue(s) = &result.tree[0] {
        // Check struct has correct comments
        let struct_docs = get_doc_strings(&s.annotations);
        assert!(struct_docs.contains(&"Main struct comment at position 0".to_string()));
        assert!(struct_docs.contains(&"Struct trailing comment".to_string()));

        // Should have 3 fields
        assert_eq!(s.members.len(), 3);

        // First field should have its comment
        let first_docs = get_doc_strings(&s.members[0].annotations);
        assert!(first_docs.contains(&"First field comment".to_string()));

        // Field from included file should have both its comments
        let included_docs = get_doc_strings(&s.members[1].annotations);
        assert_eq!(
            included_docs.len(),
            2,
            "Included field should have 2 comments, got: {included_docs:?}"
        );
        assert!(
            included_docs
                .iter()
                .any(|doc| doc.contains("Comment at position"))
        );
        assert!(
            included_docs
                .iter()
                .any(|doc| doc.contains("Trailing at position"))
        );

        // Last field should have its comment
        let last_docs = get_doc_strings(&s.members[2].annotations);
        assert!(last_docs.contains(&"Last field comment".to_string()));
    } else {
        panic!("Expected a struct");
    }
}

#[test]
fn test_fileid_collision_with_same_offsets() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test_includes");
    let main_path = test_dir.join("main2.idl");

    let mut vfs = SourceMap::default();
    let args = ProcArgs::default().include(&test_dir);
    let result = from_path(&main_path, args, &mut vfs).unwrap();

    assert!(result.errors.is_empty());

    if let Item::StructValue(s) = &result.tree[0] {
        assert_eq!(s.members.len(), 2);

        // Each field should have only its own file's comment
        let field1_docs = get_doc_strings(&s.members[0].annotations);
        assert_eq!(field1_docs.len(), 1);
        assert!(field1_docs[0].contains("file1"));
        assert!(!field1_docs[0].contains("file2"));

        let field2_docs = get_doc_strings(&s.members[1].annotations);
        assert_eq!(field2_docs.len(), 1);
        assert!(field2_docs[0].contains("file2"));
        assert!(!field2_docs[0].contains("file1"));
    } else {
        panic!("Expected a struct");
    }
}
