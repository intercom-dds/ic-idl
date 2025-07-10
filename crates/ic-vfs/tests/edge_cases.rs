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

use ic_vfs::SourceMap;

#[test]
fn test_empty_content() {
    let mut map = SourceMap::default();

    let id = map.embed("");
    assert_eq!(map.source_str(id), "");
    assert_eq!(map.source(id).len(), 0);
    assert!(map.source(id).is_empty());
}

#[test]
fn test_unicode_content() {
    let mut map = SourceMap::default();

    let unicode_samples = vec![
        "Hello, 世界!",
        "Γεια σου κόσμε",
        "مرحبا بالعالم",
        "🦀 Rust 🦀",
        "Mixed: 中文, English, العربية, 🎉",
    ];

    for (i, content) in unicode_samples.iter().enumerate() {
        let id = map.embed(content);
        assert_eq!(map.source_str(id), *content);

        // Check byte length vs char length
        let byte_len = content.len();
        let char_len = content.chars().count();
        assert_eq!(map.source(id).len(), byte_len);
        assert_eq!(map.source(id).chars().count(), char_len);

        // For emoji and non-ASCII, byte length > char length
        if i >= 3 {
            assert!(byte_len > char_len);
        }
    }
}

#[test]
fn test_null_bytes_in_content() {
    let mut map = SourceMap::default();

    let content_with_nulls = "before\0middle\0after";
    let id = map.embed(content_with_nulls);

    assert_eq!(map.source_str(id), content_with_nulls);
    assert_eq!(map.source(id).len(), 19); // includes null bytes

    // Verify null bytes are preserved
    let bytes: Vec<u8> = map.source(id).bytes().collect();
    assert_eq!(bytes[6], 0);
    assert_eq!(bytes[13], 0);
}

#[test]
fn test_very_long_names() {
    let mut map = SourceMap::default();

    // Test with extremely long file names
    let long_name = "a".repeat(1000) + ".idl";
    let id = map.embed_with_name(&long_name, "content");

    assert_eq!(map.name(id).to_str().unwrap().len(), 1004);
    assert_eq!(map.included_as(id).to_str().unwrap().len(), 1004);
}

#[test]
fn test_special_characters_in_names() {
    let mut map = SourceMap::default();

    let special_names = vec![
        "file with spaces.idl",
        "file-with-dashes.idl",
        "file_with_underscores.idl",
        "file.multiple.dots.idl",
        "path/to/file.idl",
        "../relative/path.idl",
        "C:\\Windows\\style\\path.idl",
    ];

    for name in special_names {
        let id = map.embed_with_name(name, "content");
        assert_eq!(map.name(id).to_str().unwrap(), name);
        assert_eq!(map.included_as(id).to_str().unwrap(), name);
    }
}

#[test]
fn test_line_endings_preserved() {
    let mut map = SourceMap::default();

    // Different line ending styles
    let unix_style = "line1\nline2\nline3";
    let windows_style = "line1\r\nline2\r\nline3";
    let old_mac_style = "line1\rline2\rline3";
    let mixed_style = "line1\nline2\r\nline3\rline4";

    let id_unix = map.embed(unix_style);
    let id_windows = map.embed(windows_style);
    let id_mac = map.embed(old_mac_style);
    let id_mixed = map.embed(mixed_style);

    // Content should be preserved exactly
    assert_eq!(map.source_str(id_unix), unix_style);
    assert_eq!(map.source_str(id_windows), windows_style);
    assert_eq!(map.source_str(id_mac), old_mac_style);
    assert_eq!(map.source_str(id_mixed), mixed_style);

    // Verify specific line endings
    assert!(map.source(id_unix).contains('\n'));
    assert!(!map.source(id_unix).contains('\r'));

    assert!(map.source(id_windows).contains("\r\n"));

    assert!(map.source(id_mac).contains('\r'));
    assert!(!map.source(id_mac).contains('\n'));
}

#[test]
fn test_embed_string_vs_str() {
    let mut map = SourceMap::default();

    // Test that embed works with both &str and String
    let str_content = "string literal";
    let string_content = String::from("owned string");

    let id1 = map.embed(str_content);
    let id2 = map.embed(&string_content);

    assert_eq!(map.source_str(id1), str_content);
    assert_eq!(map.source_str(id2), string_content);
}

#[test]
#[should_panic]
fn test_line_span_panics() {
    let mut map = SourceMap::default();
    let id = map.embed("test");

    // This should panic with todo!()
    let _ = map.line_span(id);
}
