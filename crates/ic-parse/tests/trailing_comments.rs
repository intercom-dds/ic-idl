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

use ic_parse::from_str;
use ic_syntax::Item;

#[test]
fn test_struct_field_trailing_comments() {
    let result = from_str(
        r"
struct Example {
    long field1; /// This is a trailing comment for field1
    string field2; /// This is a trailing comment for field2
};",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 1);

    // Check that the struct has the expected fields with annotations
    if let Item::Struct(s) = &result.tree[0] {
        assert_eq!(s.fields.len(), 2);

        // Check field1 has a trailing comment annotation
        let field1 = &s.fields[0];
        assert_eq!(field1.meta.annotations.len(), 1);
        assert_eq!(field1.meta.annotations[0].path.segments[0].name, "doc");

        // Check field2 has a trailing comment annotation
        let field2 = &s.fields[1];
        assert_eq!(field2.meta.annotations.len(), 1);
        assert_eq!(field2.meta.annotations[0].path.segments[0].name, "doc");
    } else {
        panic!("Expected a struct item");
    }
}

#[test]
fn test_leading_vs_trailing_struct_comments() {
    let result = from_str(
        r"
struct MixedComments {
    /// Leading comment for field1
    long field1;
    long field2; /// Trailing comment for field2
    /// Leading comment for field3
    /// Another line of leading comment
    string field3;
};",
    );
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );
    assert_eq!(result.tree.len(), 1);

    // Check that the struct has the expected fields with annotations
    if let Item::Struct(s) = &result.tree[0] {
        assert_eq!(s.fields.len(), 3);

        // field1 should have 1 leading comment annotation
        assert_eq!(s.fields[0].meta.annotations.len(), 1);
        assert_eq!(s.fields[0].meta.annotations[0].path.segments[0].name, "doc");

        // field2 should have 1 trailing comment annotation
        assert_eq!(s.fields[1].meta.annotations.len(), 1);
        assert_eq!(s.fields[1].meta.annotations[0].path.segments[0].name, "doc");

        // field3 should have 2 leading comment annotations
        assert_eq!(s.fields[2].meta.annotations.len(), 2);
        assert_eq!(s.fields[2].meta.annotations[0].path.segments[0].name, "doc");
        assert_eq!(s.fields[2].meta.annotations[1].path.segments[0].name, "doc");
    } else {
        panic!("Expected a struct item");
    }
}
