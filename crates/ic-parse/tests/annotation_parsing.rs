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

//! Tests for annotation parsing, especially disambiguation of qualified
//! annotation names vs qualified type names.

use ic_parse::from_str;
use ic_syntax::Item;

#[test]
fn annotation_qualified_name_no_spaces() {
    // @foo::bar is a single qualified annotation name
    let result = from_str("struct S { @foo::bar long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::StructValue(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    assert_eq!(def.members.len(), 1);
    let ann = &def.members[0].annotations[0];
    assert_eq!(ann.ident.segments.len(), 2);
    assert_eq!(ann.ident.segments[0].name, "foo");
    assert_eq!(ann.ident.segments[1].name, "bar");
}

#[test]
fn annotation_qualified_name_space_after_colons() {
    // @foo:: bar is still a single qualified annotation name (space after :: is ok)
    let result = from_str("struct S { @foo:: bar long x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::StructValue(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    assert_eq!(def.members.len(), 1);
    let ann = &def.members[0].annotations[0];
    assert_eq!(ann.ident.segments.len(), 2);
    assert_eq!(ann.ident.segments[0].name, "foo");
    assert_eq!(ann.ident.segments[1].name, "bar");
}

#[test]
fn annotation_space_before_colons_breaks_name() {
    // @foo ::bar - space before :: means ::bar is the TYPE, not part of annotation
    let result = from_str("struct S { @foo ::bar x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::StructValue(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    assert_eq!(def.members.len(), 1);

    // Annotation should just be @foo (1 segment)
    let ann = &def.members[0].annotations[0];
    assert_eq!(ann.ident.segments.len(), 1);
    assert_eq!(ann.ident.segments[0].name, "foo");

    // Type should be ::bar (qualified with leading ::)
    let ic_syntax::Type::Path(path) = &def.members[0].ty else {
        panic!("expected path type")
    };
    assert!(path.leading_colons.is_some());
    assert_eq!(path.segments[0].name, "bar");
}

#[test]
fn annotation_multi_segment_space_before_last() {
    // @foo::bar ::baz - annotation is @foo::bar, type is ::baz
    let result = from_str("struct S { @foo::bar ::baz x; };");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

    let Item::StructValue(def) = &result.tree[0] else {
        panic!("expected struct")
    };
    assert_eq!(def.members.len(), 1);

    // Annotation should be @foo::bar (2 segments)
    let ann = &def.members[0].annotations[0];
    assert_eq!(ann.ident.segments.len(), 2);
    assert_eq!(ann.ident.segments[0].name, "foo");
    assert_eq!(ann.ident.segments[1].name, "bar");

    // Type should be ::baz
    let ic_syntax::Type::Path(path) = &def.members[0].ty else {
        panic!("expected path type")
    };
    assert!(path.leading_colons.is_some());
    assert_eq!(path.segments[0].name, "baz");
}
