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

use std::ops::Range;

use ic_vfs::{Location, SourceMap, Span};

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
#[cfg(debug_assertions)]
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

    assert!(span1 < span2);
    assert_eq!(span1.cmp(&span3), std::cmp::Ordering::Less);
}

#[test]
fn test_span_empty() {
    let mut map = SourceMap::default();
    let id = map.embed("test");

    let span = Span {
        start: Location::new(5, id),
        end: Location::new(5, id),
    };

    let range = span.range();
    assert_eq!(range, 5..5);
    assert!(range.is_empty());
}
