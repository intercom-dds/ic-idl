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

use std::rc::Rc;

const INLINE_SIZE: usize = 30;

const _: () = assert!(std::mem::size_of::<InlineStr>() == 32);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InlineStr(Storage);

impl InlineStr {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Storage::Inline { len, .. } => *len == 0,
            Storage::Ref(v) => v.is_empty(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match &self.0 {
            Storage::Inline { len, .. } => usize::from(*len),
            Storage::Ref(v) => v.len(),
        }
    }
}

impl From<String> for InlineStr {
    // If the string is already heap allocated, we can just continue using that
    // instead of trying to inline it.
    fn from(value: String) -> Self {
        Self(Storage::Ref(Rc::from(value)))
    }
}

// We explicitly check the length before casting it to `u8`
#[allow(clippy::cast_possible_truncation)]
impl<'a> From<&'a str> for InlineStr {
    fn from(value: &'a str) -> Self {
        if value.len() <= INLINE_SIZE {
            let mut buffer = [0; INLINE_SIZE];
            buffer[..value.len()].copy_from_slice(value.as_bytes());
            Self(Storage::Inline {
                len: value.len() as u8,
                buffer,
            })
        } else {
            Self(Storage::Ref(Rc::from(value)))
        }
    }
}

#[derive(Clone, Debug)]
enum Storage {
    Inline { len: u8, buffer: [u8; INLINE_SIZE] },
    Ref(Rc<str>),
}

impl PartialEq for Storage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Storage::Inline {
                    len: l1,
                    buffer: b1,
                },
                Storage::Inline {
                    len: l2,
                    buffer: b2,
                },
            ) => l1 == l2 && b1[..usize::from(*l1)] == b2[..usize::from(*l2)],
            (Storage::Ref(s1), Storage::Ref(s2)) => s1 == s2,
            (Storage::Inline { len, buffer }, Storage::Ref(s))
            | (Storage::Ref(s), Storage::Inline { len, buffer }) => {
                // SAFETY: We only store valid UTF-8 in the buffer
                let inline_str =
                    unsafe { std::str::from_utf8_unchecked(&buffer[..usize::from(*len)]) };
                inline_str == &**s
            }
        }
    }
}

impl Eq for Storage {}

impl PartialOrd for Storage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Storage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let s1 = match self {
            Storage::Inline { len, buffer } => {
                // SAFETY: We only store valid UTF-8 in the buffer
                unsafe { std::str::from_utf8_unchecked(&buffer[..usize::from(*len)]) }
            }
            Storage::Ref(s) => s,
        };
        let s2 = match other {
            Storage::Inline { len, buffer } => {
                // SAFETY: We only store valid UTF-8 in the buffer
                unsafe { std::str::from_utf8_unchecked(&buffer[..usize::from(*len)]) }
            }
            Storage::Ref(s) => s,
        };
        s1.cmp(s2)
    }
}

impl std::hash::Hash for Storage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Storage::Inline { len, buffer } => {
                // SAFETY: We only store valid UTF-8 in the buffer
                let s = unsafe { std::str::from_utf8_unchecked(&buffer[..usize::from(*len)]) };
                s.hash(state);
            }
            Storage::Ref(s) => s.hash(state),
        }
    }
}

impl AsRef<str> for InlineStr {
    fn as_ref(&self) -> &str {
        match &self.0 {
            Storage::Inline { len, buffer } => {
                // SAFETY: We only store valid UTF-8 in the buffer
                unsafe { std::str::from_utf8_unchecked(&buffer[..usize::from(*len)]) }
            }
            Storage::Ref(s) => s,
        }
    }
}

impl std::fmt::Display for InlineStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl std::ops::Deref for InlineStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_empty_str() {
        let s = InlineStr::from("");
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.as_ref(), "");
        assert!(matches!(s.0, Storage::Inline { len: 0, .. }));
    }

    #[test]
    fn test_from_small_str() {
        let s = InlineStr::from("hello");
        assert!(!s.is_empty());
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_ref(), "hello");
        assert!(matches!(s.0, Storage::Inline { len: 5, .. }));
    }

    #[test]
    fn test_from_max_inline_str() {
        let text = "a".repeat(INLINE_SIZE);
        let s = InlineStr::from(text.as_str());
        assert_eq!(s.len(), INLINE_SIZE);
        assert_eq!(s.as_ref(), text);
        assert!(matches!(s.0, Storage::Inline { .. }));
    }

    #[test]
    fn test_from_large_str() {
        let text = "a".repeat(INLINE_SIZE + 1);
        let s = InlineStr::from(text.as_str());
        assert_eq!(s.len(), INLINE_SIZE + 1);
        assert_eq!(s.as_ref(), text);
        assert!(matches!(s.0, Storage::Ref(_)));
    }

    #[test]
    fn test_from_string() {
        let text = String::from("hello world");
        let s = InlineStr::from(text.clone());
        assert_eq!(s.len(), 11);
        assert_eq!(s.as_ref(), "hello world");
        // String is always stored as Ref
        assert!(matches!(s.0, Storage::Ref(_)));
    }

    #[test]
    fn test_display() {
        let s = InlineStr::from("hello");
        assert_eq!(format!("{s}"), "hello");

        let s = InlineStr::from("a".repeat(50).as_str());
        assert_eq!(format!("{s}"), "a".repeat(50));
    }

    #[test]
    fn test_deref() {
        let s = InlineStr::from("hello");
        // Test that we can use string methods via deref
        assert_eq!(s.chars().count(), 5);
        assert!(s.starts_with("hel"));
        assert!(s.ends_with("lo"));
    }

    #[test]
    fn test_clone() {
        let s1 = InlineStr::from("hello");
        let s2 = s1.clone();
        assert_eq!(s1, s2);
        assert_eq!(s1.as_ref(), s2.as_ref());
    }

    #[test]
    fn test_debug() {
        let s = InlineStr::from("hello");
        let debug_str = format!("{s:?}");
        assert!(debug_str.contains("InlineStr"));
        assert!(debug_str.contains("Inline"));
    }

    #[test]
    fn test_eq() {
        let s1 = InlineStr::from("hello");
        let s2 = InlineStr::from("hello");
        let s3 = InlineStr::from("world");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_ord() {
        let s1 = InlineStr::from("apple");
        let s2 = InlineStr::from("banana");
        let s3 = InlineStr::from("apple");

        assert!(s1 < s2);
        assert!(s2 > s1);
        assert!(s1 <= s3);
        assert!(s1 >= s3);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(InlineStr::from("hello"));
        set.insert(InlineStr::from("world"));
        set.insert(InlineStr::from("hello")); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&InlineStr::from("hello")));
        assert!(set.contains(&InlineStr::from("world")));
    }

    #[test]
    fn test_size_guarantee() {
        // Verify the size is exactly 32 bytes as the compile-time assert expects
        assert_eq!(std::mem::size_of::<InlineStr>(), 32);
    }

    #[test]
    fn test_utf8_chars() {
        let s = InlineStr::from("hello 世界");
        assert_eq!(s.as_ref(), "hello 世界");
        assert_eq!(s.len(), 12); // "hello " = 6 bytes, "世界" = 6 bytes
        assert!(matches!(s.0, Storage::Inline { .. }));
    }

    #[test]
    fn test_large_utf8() {
        // Test with multibyte chars that would exceed inline capacity
        let text = "世".repeat(11); // 11 * 3 = 33 bytes > 30
        let s = InlineStr::from(text.as_str());
        assert_eq!(s.len(), 33);
        assert!(matches!(s.0, Storage::Ref(_)));
    }

    #[test]
    fn test_inline_ref_equality() {
        // Test that inline and ref versions of the same string are equal
        let inline_str = InlineStr::from("hello");
        let ref_str = InlineStr::from(String::from("hello"));

        assert_eq!(inline_str, ref_str);
        assert_eq!(inline_str.as_ref(), ref_str.as_ref());
    }

    #[test]
    fn test_empty_string() {
        let s1 = InlineStr::from("");
        let s2 = InlineStr::from(String::new());

        assert!(s1.is_empty());
        assert!(s2.is_empty());
        assert_eq!(s1.len(), 0);
        assert_eq!(s2.len(), 0);
    }

    #[test]
    fn test_boundary_cases() {
        // Test strings of exactly 29, 30, and 31 bytes
        let s29 = InlineStr::from("a".repeat(29).as_str());
        let s30 = InlineStr::from("a".repeat(30).as_str());
        let s31 = InlineStr::from("a".repeat(31).as_str());

        assert!(matches!(s29.0, Storage::Inline { .. }));
        assert!(matches!(s30.0, Storage::Inline { .. }));
        assert!(matches!(s31.0, Storage::Ref(_)));
    }
}
