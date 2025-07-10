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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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
use std::rc::Rc;

use ic_vfs::{Include, SourceMap};

#[test]
fn test_embed_virtual_file() {
    let mut map = SourceMap::default();

    let content = "virtual file content";
    let id = map.embed(content);

    assert_eq!(map.source_str(id), content);
    assert!(map.name(id).to_str().unwrap().starts_with("<builtin-"));
}

#[test]
fn test_embed_with_custom_name() {
    let mut map = SourceMap::default();

    let content = "custom virtual file";
    let name = "<my-custom-file>";
    let id = map.embed_with_name(name, content);

    assert_eq!(map.source_str(id), content);
    assert_eq!(map.name(id), Path::new(name));
    assert_eq!(map.included_as(id), Path::new(name));
}

#[test]
fn test_multiple_embeds() {
    let mut map = SourceMap::default();

    let id1 = map.embed("first");
    let id2 = map.embed("second");
    let id3 = map.embed("third");

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    assert_eq!(map.source_str(id1), "first");
    assert_eq!(map.source_str(id2), "second");
    assert_eq!(map.source_str(id3), "third");
}

#[test]
fn test_source_returns_rc_str() {
    let mut map = SourceMap::default();

    let content = "test content";
    let id = map.embed(content);

    let rc1 = map.source(id);
    let rc2 = map.source(id);

    // Both should point to the same allocation
    assert!(Rc::ptr_eq(&rc1, &rc2));
    assert_eq!(&*rc1, content);
}

#[test]
fn test_file_info_metadata() {
    let mut map = SourceMap::default();

    let id = map.embed_with_name("test.idl", "content");
    let info = map.file_info(id);

    assert_eq!(info.path, Path::new("test.idl"));
    assert_eq!(info.included_as, Path::new("test.idl"));
    assert_eq!(&*info.source, "content");
    assert_eq!(info.kind, Include::Static);
}

#[test]
fn test_embed_with_rc_str() {
    let mut map = SourceMap::default();

    let content: Rc<str> = Rc::from("shared content");
    let id = map.embed_with_name("shared.idl", content.clone());

    let retrieved = map.source(id);
    assert!(Rc::ptr_eq(&content, &retrieved));
}

#[test]
fn test_embed_increments_builtin_count() {
    let mut map = SourceMap::default();

    let id1 = map.embed("first");
    let id2 = map.embed("second");

    let name1 = map.name(id1).to_str().unwrap();
    let name2 = map.name(id2).to_str().unwrap();

    assert!(name1.starts_with("<builtin-"));
    assert!(name2.starts_with("<builtin-"));
    assert_ne!(name1, name2);
}
