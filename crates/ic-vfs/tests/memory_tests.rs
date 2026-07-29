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

use std::rc::Rc;

use ic_vfs::{Include, SourceMap};

#[test]
fn test_rc_sharing() {
    let mut map = SourceMap::default();
    let large_content = "x".repeat(10000);
    let id = map.embed(&large_content);

    let rc1 = map.source(id);
    let rc2 = map.source(id);
    let rc3 = map.source(id);
    assert!(Rc::ptr_eq(&rc1, &rc2));
    assert!(Rc::ptr_eq(&rc2, &rc3));
    assert!(Rc::strong_count(&rc1) >= 4);
}

#[test]
fn test_source_str_borrowing() {
    let mut map = SourceMap::default();

    let content = "borrowed content";
    let id = map.embed(content);

    // source_str should return a borrowed reference
    let borrowed = map.source_str(id);
    assert_eq!(borrowed, content);

    // We can have multiple borrows simultaneously
    let borrow1 = map.source_str(id);
    let borrow2 = map.source_str(id);
    assert_eq!(borrow1.as_ptr(), borrow2.as_ptr());
}

#[test]
fn test_embed_preserves_rc() {
    let mut map = SourceMap::default();

    let external_rc: Rc<str> = Rc::from("shared string");
    let original_ptr = Rc::as_ptr(&external_rc);
    let original_count = Rc::strong_count(&external_rc);
    let id = map.embed_with_name("shared.idl", external_rc.clone());
    assert_eq!(Rc::strong_count(&external_rc), original_count + 1);

    let retrieved = map.source(id);
    assert_eq!(Rc::as_ptr(&retrieved), original_ptr);
}

#[test]
fn test_different_ids_for_different_content() {
    let mut map = SourceMap::default();

    let id1 = map.embed("content1");
    let id2 = map.embed("content2");
    let id3 = map.embed("content3");
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_embed_counter_increments() {
    let mut map = SourceMap::default();

    let ids: Vec<_> = (0..5).map(|_| map.embed("content")).collect();
    let names: Vec<_> = ids
        .iter()
        .map(|&id| map.name(id).to_str().unwrap().to_string())
        .collect();

    // All names should be unique
    for i in 0..names.len() {
        for j in i + 1..names.len() {
            assert_ne!(names[i], names[j]);
        }
    }

    // All should start with "<builtin-"
    for name in &names {
        assert!(name.starts_with("<builtin-"));
    }
}

#[test]
fn test_included_as_vs_path() {
    let mut map = SourceMap::default();
    let id = map.embed_with_name("virtual.idl", "content");
    assert_eq!(map.path(id), map.included_as(id));

    assert_eq!(map.path(id).to_str().unwrap(), "virtual.idl");
    assert_eq!(map.included_as(id).to_str().unwrap(), "virtual.idl");
}

#[test]
fn test_file_info_access() {
    let mut map = SourceMap::default();
    let content = "test content";
    let name = "test.idl";
    let id = map.embed_with_name(name, content);
    let info = map.file_info(id);
    assert_eq!(info.path.to_str().unwrap(), name);
    assert_eq!(info.included_as.to_str().unwrap(), name);
    assert_eq!(&*info.source, content);
    assert_eq!(info.kind, Include::Static);
}

#[test]
fn test_static_embed_kind() {
    let mut map = SourceMap::default();
    let id1 = map.embed("content");
    let id2 = map.embed_with_name("named.idl", "content");
    assert_eq!(map.file_info(id1).kind, Include::Static);
    assert_eq!(map.file_info(id2).kind, Include::Static);
}
