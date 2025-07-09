use ic_vfs::{Include, SourceMap};
use std::path::Path;
use std::rc::Rc;

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