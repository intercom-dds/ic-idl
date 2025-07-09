use ic_vfs::{Include, SourceMap};
use std::rc::Rc;

#[test]
fn test_rc_sharing() {
    let mut map = SourceMap::default();
    
    // Create a large string to make reference counting benefits clear
    let large_content = "x".repeat(10000);
    let id = map.embed(&large_content);
    
    // Get multiple references
    let rc1 = map.source(id);
    let rc2 = map.source(id);
    let rc3 = map.source(id);
    
    // All should point to the same allocation
    assert!(Rc::ptr_eq(&rc1, &rc2));
    assert!(Rc::ptr_eq(&rc2, &rc3));
    
    // Reference count should be at least 4 (original + 3 clones)
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
    
    // Create an Rc<str> externally
    let external_rc: Rc<str> = Rc::from("shared string");
    let original_ptr = Rc::as_ptr(&external_rc);
    let original_count = Rc::strong_count(&external_rc);
    
    // Embed it
    let id = map.embed_with_name("shared.idl", external_rc.clone());
    
    // Should have increased reference count
    assert_eq!(Rc::strong_count(&external_rc), original_count + 1);
    
    // Retrieved Rc should point to same allocation
    let retrieved = map.source(id);
    assert_eq!(Rc::as_ptr(&retrieved), original_ptr);
}

#[test]
fn test_different_ids_for_different_content() {
    let mut map = SourceMap::default();
    
    let id1 = map.embed("content1");
    let id2 = map.embed("content2");
    let id3 = map.embed("content3");
    
    // All IDs should be different
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_embed_counter_increments() {
    let mut map = SourceMap::default();
    
    // Embed multiple files and check names are unique
    let ids: Vec<_> = (0..5).map(|_| map.embed("content")).collect();
    
    let names: Vec<_> = ids.iter()
        .map(|&id| map.name(id).to_str().unwrap().to_string())
        .collect();
    
    // All names should be unique
    for i in 0..names.len() {
        for j in i+1..names.len() {
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
    
    // For embedded files, path and included_as should be the same
    let id = map.embed_with_name("virtual.idl", "content");
    assert_eq!(map.path(id), map.included_as(id));
    
    // Both should equal the name we provided
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
    
    // All embedded files should have Static kind
    let id1 = map.embed("content");
    let id2 = map.embed_with_name("named.idl", "content");
    
    assert_eq!(map.file_info(id1).kind, Include::Static);
    assert_eq!(map.file_info(id2).kind, Include::Static);
}