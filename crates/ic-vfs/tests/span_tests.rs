use ic_vfs::{Location, Span, SourceMap};
use std::ops::Range;

#[test]
fn test_location_creation() {
    let mut map = SourceMap::default();
    let id = map.embed("test content");
    
    let loc = Location::new(42, id);
    assert_eq!(loc.offset, 42);
    assert_eq!(loc.file_id, id);
}

#[test]
fn test_location_ordering() {
    let mut map = SourceMap::default();
    let id1 = map.embed("file1");
    let id2 = map.embed("file2");
    
    let loc1 = Location::new(10, id1);
    let loc2 = Location::new(20, id1);
    let loc3 = Location::new(10, id2);
    
    // Same file, different offsets
    assert!(loc1 < loc2);
    assert!(loc2 > loc1);
    
    // Different files
    if id1 < id2 {
        assert!(loc1 < loc3);
    } else {
        assert!(loc1 > loc3);
    }
}

#[test]
fn test_span_creation() {
    let mut map = SourceMap::default();
    let id = map.embed("test content");
    
    let start = Location::new(5, id);
    let end = Location::new(10, id);
    
    let span = Span { start, end };
    assert_eq!(span.start.offset, 5);
    assert_eq!(span.end.offset, 10);
    assert_eq!(span.start.file_id, id);
    assert_eq!(span.end.file_id, id);
}

#[test]
fn test_span_range() {
    let mut map = SourceMap::default();
    let id = map.embed("test content");
    
    let span = Span {
        start: Location::new(5, id),
        end: Location::new(15, id),
    };
    
    let range = span.range();
    assert_eq!(range, 5..15);
    assert_eq!(range.start, 5);
    assert_eq!(range.end, 15);
    assert_eq!(range.len(), 10);
}

#[test]
fn test_span_into_range() {
    let mut map = SourceMap::default();
    let id = map.embed("test content");
    
    let span = Span {
        start: Location::new(0, id),
        end: Location::new(100, id),
    };
    
    let range: Range<usize> = span.into();
    assert_eq!(range, 0..100);
}

#[test]
#[should_panic(expected = "start.file_id != end.file_id")]
fn test_span_range_different_files() {
    let mut map = SourceMap::default();
    let id1 = map.embed("file1");
    let id2 = map.embed("file2");
    
    let span = Span {
        start: Location::new(5, id1),
        end: Location::new(10, id2),
    };
    
    // This should panic in debug mode
    let _ = span.range();
}

#[test]
fn test_span_ordering() {
    let mut map = SourceMap::default();
    let id = map.embed("test content");
    
    let span1 = Span {
        start: Location::new(0, id),
        end: Location::new(10, id),
    };
    
    let span2 = Span {
        start: Location::new(5, id),
        end: Location::new(15, id),
    };
    
    let span3 = Span {
        start: Location::new(0, id),
        end: Location::new(20, id),
    };
    
    // Spans are ordered by start position first
    assert!(span1 < span2); // starts at 0 vs 5
    assert_eq!(span1.cmp(&span3), std::cmp::Ordering::Less); // same start, different end
}

#[test]
fn test_span_chumsky_trait() {
    use chumsky::Span as ChumskySpan;
    
    let mut map = SourceMap::default();
    let id = map.embed("test content");
    
    let start = Location::new(10, id);
    let end = Location::new(20, id);
    
    // Test chumsky::Span trait implementation
    let span = Span::new((), start..end);
    assert_eq!(span.start(), start);
    assert_eq!(span.end(), end);
    assert_eq!(span.context(), ());
}

#[test]
fn test_location_default() {
    // Default location should have offset 0
    let loc = Location::default();
    assert_eq!(loc.offset, 0);
    // Note: We can't test the file_id because it uses _do_not_use()
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start.offset, 0);
    assert_eq!(span.end.offset, 0);
}

#[test]
fn test_span_empty() {
    let mut map = SourceMap::default();
    let id = map.embed("test");
    
    // Empty span at position 5
    let span = Span {
        start: Location::new(5, id),
        end: Location::new(5, id),
    };
    
    let range = span.range();
    assert_eq!(range, 5..5);
    assert!(range.is_empty());
}