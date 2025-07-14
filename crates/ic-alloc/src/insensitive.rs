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

//! Case-insensitive strings and maps.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// A wrapper type for case-insensitive string comparison.
/// Used internally for lookups without allocating.
#[derive(Debug)]
struct CaseInsensitiveStr<'a>(&'a str);

impl Hash for CaseInsensitiveStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.0.bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl PartialEq for CaseInsensitiveStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for CaseInsensitiveStr<'_> {}

/// A string that preserves its original casing but compares case-insensitively.
#[derive(Clone, Debug)]
pub struct CaseString {
    original: String,
}

impl CaseString {
    /// Creates a new `CaseString` from the given string.
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self { original: s.into() }
    }

    /// Returns the string with its original casing.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.original
    }
}

impl AsRef<str> for CaseString {
    fn as_ref(&self) -> &str {
        &self.original
    }
}

impl std::fmt::Display for CaseString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.original)
    }
}

impl Hash for CaseString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        CaseInsensitiveStr(&self.original).hash(state);
    }
}

impl PartialEq for CaseString {
    fn eq(&self, other: &Self) -> bool {
        self.original.eq_ignore_ascii_case(&other.original)
    }
}

impl Eq for CaseString {}

/// A case-insensitive map that stores keys in their original form but
/// performs case-insensitive lookups.
#[derive(Debug, Default)]
pub struct CaseMap<T> {
    inner: HashMap<CaseString, T>,
}

impl<T> CaseMap<T> {
    /// Creates a new empty `CaseMap`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Inserts a key-value pair into the map.
    /// Returns the previous value if the key was already present.
    pub fn insert<K: Into<String>>(&mut self, key: K, value: T) -> Option<T> {
        let key_str = key.into();
        // First check if key already exists (case-insensitive)
        if let Some((existing_key, _)) = self
            .inner
            .iter()
            .find(|(k, _)| k.as_str().eq_ignore_ascii_case(&key_str))
        {
            let existing_key = existing_key.clone();
            self.inner.insert(existing_key, value)
        } else {
            let key = CaseString::new(key_str);
            self.inner.insert(key, value)
        }
    }

    /// Gets a reference to the value for the given key.
    pub fn get<Q: AsRef<str>>(&self, key: Q) -> Option<&T> {
        self.inner
            .iter()
            .find(|(k, _)| k.as_str().eq_ignore_ascii_case(key.as_ref()))
            .map(|(_, v)| v)
    }

    /// Gets a mutable reference to the value for the given key.
    pub fn get_mut<Q: AsRef<str>>(&mut self, key: Q) -> Option<&mut T> {
        self.inner
            .iter_mut()
            .find(|(k, _)| k.as_str().eq_ignore_ascii_case(key.as_ref()))
            .map(|(_, v)| v)
    }

    /// Gets the original casing of a key if it exists in the map.
    pub fn get_key<Q: AsRef<str>>(&self, key: Q) -> Option<&str> {
        self.inner
            .keys()
            .find(|k| k.as_str().eq_ignore_ascii_case(key.as_ref()))
            .map(CaseString::as_str)
    }

    /// Removes a key from the map, returning the value if it was present.
    pub fn remove<Q: AsRef<str>>(&mut self, key: Q) -> Option<T> {
        let key_to_remove = self
            .inner
            .keys()
            .find(|k| k.as_str().eq_ignore_ascii_case(key.as_ref()))
            .cloned()?;
        self.inner.remove(&key_to_remove)
    }

    /// Checks if the map contains the given key.
    pub fn contains_key<Q: AsRef<str>>(&self, key: Q) -> bool {
        self.inner
            .keys()
            .any(|k| k.as_str().eq_ignore_ascii_case(key.as_ref()))
    }

    /// Returns the number of entries in the map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the map, removing all entries.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an iterator over the entries in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Returns a mutable iterator over the entries in the map.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut T)> {
        self.inner.iter_mut().map(|(k, v)| (k.as_str(), v))
    }

    /// Returns an iterator over the values in the map.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.inner.values()
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry<K: Into<String>>(&mut self, key: K) -> CaseMapEntry<'_, T> {
        let key_str = key.into();

        // Check if key already exists (case-insensitive)
        if let Some(existing_key) = self
            .inner
            .keys()
            .find(|k| k.as_str().eq_ignore_ascii_case(&key_str))
            .cloned()
        {
            CaseMapEntry {
                inner: self.inner.entry(existing_key),
            }
        } else {
            let case_key = CaseString::new(key_str);
            CaseMapEntry {
                inner: self.inner.entry(case_key),
            }
        }
    }
}

/// An entry in a `CaseMap`.
pub struct CaseMapEntry<'a, T> {
    inner: Entry<'a, CaseString, T>,
}

impl<'a, T> CaseMapEntry<'a, T> {
    /// Returns a reference to this entry's key with original casing.
    #[must_use]
    pub fn key(&self) -> &str {
        match &self.inner {
            Entry::Occupied(e) => e.key().as_str(),
            Entry::Vacant(e) => e.key().as_str(),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty.
    pub fn or_insert(self, default: T) -> &'a mut T {
        self.inner.or_insert(default)
    }

    /// Ensures a value is in the entry by inserting the result of the function if empty.
    pub fn or_insert_with<F: FnOnce() -> T>(self, f: F) -> &'a mut T {
        self.inner.or_insert_with(f)
    }
}

/// A case-insensitive set that stores strings in their original form.
#[derive(Debug, Default)]
pub struct CaseSet {
    inner: HashSet<CaseString>,
}

impl CaseSet {
    /// Creates a new empty `CaseSet`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    /// Inserts a string into the set.
    /// Returns true if the value was newly inserted.
    pub fn insert<K: Into<String>>(&mut self, key: K) -> bool {
        let key_str = key.into();
        // Check if key already exists (case-insensitive)
        if self
            .inner
            .iter()
            .any(|k| k.as_str().eq_ignore_ascii_case(&key_str))
        {
            false
        } else {
            let key = CaseString::new(key_str);
            self.inner.insert(key)
        }
    }

    /// Checks if the set contains the given string.
    pub fn contains<Q: AsRef<str>>(&self, key: Q) -> bool {
        self.inner
            .iter()
            .any(|k| k.as_str().eq_ignore_ascii_case(key.as_ref()))
    }

    /// Gets the original casing of a string if it exists in the set.
    pub fn get<Q: AsRef<str>>(&self, key: Q) -> Option<&str> {
        self.inner
            .iter()
            .find(|k| k.as_str().eq_ignore_ascii_case(key.as_ref()))
            .map(CaseString::as_str)
    }

    /// Removes a string from the set.
    /// Returns true if the value was present.
    pub fn remove<Q: AsRef<str>>(&mut self, key: Q) -> bool {
        if let Some(key_to_remove) = self
            .inner
            .iter()
            .find(|k| k.as_str().eq_ignore_ascii_case(key.as_ref()))
            .cloned()
        {
            self.inner.remove(&key_to_remove)
        } else {
            false
        }
    }

    /// Returns the number of strings in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the set, removing all strings.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an iterator over the strings in the set.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(CaseString::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_insensitive_map_basic() {
        let mut map = CaseMap::new();

        // Insert with original casing
        map.insert("FooBar", 42);

        // Lookup with different casings
        assert_eq!(map.get("foobar"), Some(&42));
        assert_eq!(map.get("FOOBAR"), Some(&42));
        assert_eq!(map.get("FooBar"), Some(&42));
        assert_eq!(map.get("fOoBaR"), Some(&42));

        // Get original casing
        assert_eq!(map.get_key("foobar"), Some("FooBar"));
        assert_eq!(map.get_key("FOOBAR"), Some("FooBar"));
    }

    #[test]
    fn case_insensitive_map_overwrite() {
        let mut map = CaseMap::new();

        // Insert with one casing
        map.insert("test", 1);
        assert_eq!(map.get_key("TEST"), Some("test"));

        // Insert with different casing overwrites
        map.insert("TEST", 2);
        assert_eq!(map.get("test"), Some(&2));
        // Original casing is preserved from first insert
        assert_eq!(map.get_key("test"), Some("test"));
    }

    #[test]
    fn case_insensitive_map_entry() {
        let mut map = CaseMap::new();

        // Insert via entry API
        map.entry("Hello").or_insert(1);
        assert_eq!(map.get("hello"), Some(&1));
        assert_eq!(map.get_key("HELLO"), Some("Hello"));

        // Access existing entry with different casing
        *map.entry("HELLO").or_insert(999) = 2;
        assert_eq!(map.get("hello"), Some(&2));
        // Original casing preserved
        assert_eq!(map.get_key("hello"), Some("Hello"));
    }

    #[test]
    fn case_insensitive_set_basic() {
        let mut set = CaseSet::new();

        // Insert with original casing
        assert!(set.insert("FooBar"));

        // Contains checks with different casings
        assert!(set.contains("foobar"));
        assert!(set.contains("FOOBAR"));
        assert!(set.contains("FooBar"));

        // Get original casing
        assert_eq!(set.get("foobar"), Some("FooBar"));
        assert_eq!(set.get("FOOBAR"), Some("FooBar"));

        // Duplicate insert returns false
        assert!(!set.insert("foobar"));
        assert!(!set.insert("FOOBAR"));
    }

    #[test]
    fn case_insensitive_map_remove() {
        let mut map = CaseMap::new();
        map.insert("Test", 123);

        // Remove with different casing
        assert_eq!(map.remove("TEST"), Some(123));
        assert!(map.is_empty());
    }

    #[test]
    fn case_insensitive_set_remove() {
        let mut set = CaseSet::new();
        set.insert("Test");

        // Remove with different casing
        assert!(set.remove("TEST"));
        assert!(set.is_empty());
    }

    #[test]
    fn case_insensitive_map_iter() {
        let mut map = CaseMap::new();
        map.insert("Alpha", 1);
        map.insert("BETA", 2);
        map.insert("gamma", 3);

        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|&(_, &v)| v);

        assert_eq!(entries[0].0, "Alpha");
        assert_eq!(entries[1].0, "BETA");
        assert_eq!(entries[2].0, "gamma");
    }
}
