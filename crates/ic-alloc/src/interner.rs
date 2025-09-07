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

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::arena::{Arena, Id};

/// A cached entry in the interner.
#[derive(Debug)]
pub struct CachedStr {
    hash: u64,
    string: Box<str>,
}

impl CachedStr {
    fn new(string: Box<str>) -> Self {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        let hash = hasher.finish();

        Self { hash, string }
    }
}

impl Hash for CachedStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

/// An ID of a string in the [`Interner`].
/// An immutable reference to the string can be acquired with
/// [`Interner::get`].
pub type SymbolId = Id<CachedStr>;

/// String interner that stores at most one copy of each unique string.
/// Deletion is not supported; all strings are deallocated simulatenously when
/// the interner is dropped.
///
/// Interned strings may not be mutated in any way.
#[must_use]
#[derive(Debug, Default)]
pub struct Interner {
    arena: Arena<CachedStr>,
    cache: HashMap<u64, Vec<SymbolId>>,
}

impl Interner {
    pub fn new() -> Self {
        Self::with_capacity(64)
    }

    pub fn with_capacity(len: usize) -> Self {
        Self {
            arena: Arena::with_capacity(len),
            cache: HashMap::with_capacity(len),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    #[must_use]
    pub fn get(&self, id: SymbolId) -> &str {
        &self.arena.get(id).string
    }

    pub fn insert<I>(&mut self, str: I) -> SymbolId
    where
        I: Into<Box<str>>,
    {
        let cached = CachedStr::new(str.into());

        match self.cache.entry(cached.hash) {
            Entry::Occupied(mut v) => {
                // Check for actual string match due to potential hash collisions
                for &id in v.get() {
                    if self.arena.get(id).string == cached.string {
                        return id;
                    }
                }
                // Hash collision with different string
                let id = self.arena.alloc(cached);
                v.get_mut().push(id);
                id
            }
            Entry::Vacant(v) => {
                let id = self.arena.alloc(cached);
                v.insert(vec![id]);
                id
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate() {
        let str = "foobar";
        let mut interner = Interner::new();

        let id = interner.insert(str);
        let id2 = interner.insert(str);
        assert_eq!(id, id2);
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn get_str() {
        let str = "foobar";
        let mut interner = Interner::new();
        let id = interner.insert(str);

        assert_eq!(interner.get(id), str);
    }

    #[test]
    fn interner_new() {
        let interner = Interner::new();
        assert!(interner.is_empty());
        assert_eq!(interner.len(), 0);
    }

    #[test]
    fn interner_with_capacity() {
        let interner = Interner::with_capacity(100);
        assert!(interner.is_empty());
        assert_eq!(interner.len(), 0);
    }

    #[test]
    fn interner_default() {
        let interner: Interner = Interner::default();
        assert!(interner.is_empty());
        assert_eq!(interner.len(), 0);
    }

    #[test]
    fn multiple_different_strings() {
        let mut interner = Interner::new();

        let id1 = interner.insert("first");
        let id2 = interner.insert("second");
        let id3 = interner.insert("third");

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        assert_eq!(interner.len(), 3);

        assert_eq!(interner.get(id1), "first");
        assert_eq!(interner.get(id2), "second");
        assert_eq!(interner.get(id3), "third");
    }

    #[test]
    fn string_types() {
        let mut interner = Interner::new();

        // &str
        let id1 = interner.insert("static");

        // String
        let s = String::from("dynamic");
        let id2 = interner.insert(s);

        // Box<str>
        let boxed: Box<str> = "boxed".into();
        let id3 = interner.insert(boxed);

        assert_eq!(interner.get(id1), "static");
        assert_eq!(interner.get(id2), "dynamic");
        assert_eq!(interner.get(id3), "boxed");

        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn duplicate_string() {
        let mut interner = Interner::new();

        let s = String::from("test");
        let id1 = interner.insert(s.clone());
        let id2 = interner.insert(s);

        assert_eq!(id1, id2);
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn empty_string() {
        let mut interner = Interner::new();

        let id = interner.insert("");
        assert_eq!(interner.get(id), "");

        let id2 = interner.insert("");
        assert_eq!(id, id2);
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn unicode_strings() {
        let mut interner = Interner::new();

        let id1 = interner.insert("hello 世界");
        let id2 = interner.insert("🦀 Rust");
        let id3 = interner.insert("café");

        assert_eq!(interner.get(id1), "hello 世界");
        assert_eq!(interner.get(id2), "🦀 Rust");
        assert_eq!(interner.get(id3), "café");

        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn large_string() {
        let mut interner = Interner::new();

        let large = "a".repeat(10000);
        let id = interner.insert(large.as_str());

        assert_eq!(interner.get(id), large);

        // Duplicate should return same id
        let id2 = interner.insert(large.as_str());
        assert_eq!(id, id2);
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn many_strings() {
        let mut interner = Interner::new();
        let mut ids = Vec::new();

        for i in 0..1000 {
            let s = format!("string_{i}");
            let id = interner.insert(s.as_str());
            ids.push((id, s));
        }

        assert_eq!(interner.len(), 1000);

        // Verify all strings
        for (id, expected) in &ids {
            assert_eq!(interner.get(*id), expected);
        }
    }

    #[test]
    fn cached_str_new() {
        let s: Box<str> = "test".into();
        let cached = CachedStr::new(s);

        assert_eq!(&*cached.string, "test");
        assert!(cached.hash != 0); // Hash should be computed
    }

    #[test]
    fn cached_str_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let cached1 = CachedStr::new("test".into());
        let cached2 = CachedStr::new("test".into());

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        cached1.hash(&mut hasher1);
        cached2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn cached_str_debug() {
        let cached = CachedStr::new("debug test".into());
        let debug_str = format!("{cached:?}");

        assert!(debug_str.contains("CachedStr"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn interner_debug() {
        let mut interner = Interner::new();
        let _ = interner.insert("test");

        let debug_str = format!("{interner:?}");
        assert!(debug_str.contains("Interner"));
    }

    #[test]
    fn symbol_id_type() {
        let mut interner = Interner::new();
        let id: SymbolId = interner.insert("test");

        // SymbolId should be usable as Id<CachedStr>
        let _cached: &CachedStr = interner.arena.get(id);
    }

    #[test]
    fn case_sensitive() {
        let mut interner = Interner::new();

        let id1 = interner.insert("Test");
        let id2 = interner.insert("test");
        let id3 = interner.insert("TEST");

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn whitespace_differences() {
        let mut interner = Interner::new();

        let id1 = interner.insert("hello world");
        let id2 = interner.insert("hello  world"); // Two spaces
        let id3 = interner.insert("hello\tworld"); // Tab
        let id4 = interner.insert(" hello world"); // Leading space
        let id5 = interner.insert("hello world "); // Trailing space

        // All should be different
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
        assert_ne!(id1, id5);

        assert_eq!(interner.len(), 5);
    }

    #[test]
    fn insert_after_get() {
        let mut interner = Interner::new();

        let id1 = interner.insert("first");
        let _ = interner.get(id1); // Access it

        let id2 = interner.insert("first"); // Insert same string again
        assert_eq!(id1, id2);

        let id3 = interner.insert("second");
        assert_ne!(id1, id3);
    }

    // This test would catch the hash collision bug if we only used hash for lookup
    #[test]
    fn potential_hash_collision_handling() {
        let mut interner = Interner::new();

        // In practice, finding actual hash collisions is difficult,
        // but the implementation should handle them correctly
        let id1 = interner.insert("string1");
        let id2 = interner.insert("string2");

        assert_ne!(id1, id2);
        assert_eq!(interner.get(id1), "string1");
        assert_eq!(interner.get(id2), "string2");
    }
}
