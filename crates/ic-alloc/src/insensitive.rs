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

    #[test]
    fn case_string_new() {
        let cs = CaseString::new("Hello");
        assert_eq!(cs.as_str(), "Hello");
        assert_eq!(cs.as_ref(), "Hello");

        let cs = CaseString::new(String::from("World"));
        assert_eq!(cs.as_str(), "World");
    }

    #[test]
    fn case_string_display() {
        let cs = CaseString::new("FooBar");
        assert_eq!(format!("{cs}"), "FooBar");
    }

    #[test]
    fn case_string_hash_eq() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(CaseString::new("Hello"));
        set.insert(CaseString::new("HELLO"));
        set.insert(CaseString::new("world"));

        // Only 2 unique strings (case-insensitive)
        assert_eq!(set.len(), 2);

        // Check equality
        let cs1 = CaseString::new("Test");
        let cs2 = CaseString::new("TEST");
        let cs3 = CaseString::new("test");

        assert_eq!(cs1, cs2);
        assert_eq!(cs2, cs3);
        assert_eq!(cs1, cs3);
    }

    #[test]
    fn case_string_clone() {
        let cs1 = CaseString::new("Original");
        let cs2 = cs1.clone();

        assert_eq!(cs1, cs2);
        assert_eq!(cs1.as_str(), cs2.as_str());
    }

    #[test]
    fn case_string_debug() {
        let cs = CaseString::new("Test");
        let debug_str = format!("{cs:?}");
        assert!(debug_str.contains("CaseString"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn case_map_default() {
        let map: CaseMap<i32> = CaseMap::default();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn case_map_get_mut() {
        let mut map = CaseMap::new();
        map.insert("key", 10);

        if let Some(value) = map.get_mut("KEY") {
            *value = 20;
        }

        assert_eq!(map.get("key"), Some(&20));
    }

    #[test]
    fn case_map_contains_key() {
        let mut map = CaseMap::new();
        map.insert("Present", 42);

        assert!(map.contains_key("present"));
        assert!(map.contains_key("PRESENT"));
        assert!(map.contains_key("Present"));
        assert!(!map.contains_key("absent"));
    }

    #[test]
    fn case_map_clear() {
        let mut map = CaseMap::new();
        map.insert("one", 1);
        map.insert("two", 2);
        map.insert("three", 3);

        assert_eq!(map.len(), 3);

        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn case_map_iter_mut() {
        let mut map = CaseMap::new();
        map.insert("a", 1);
        map.insert("B", 2);
        map.insert("C", 3);

        for (_, value) in map.iter_mut() {
            *value *= 2;
        }

        assert_eq!(map.get("a"), Some(&2));
        assert_eq!(map.get("b"), Some(&4));
        assert_eq!(map.get("c"), Some(&6));
    }

    #[test]
    fn case_map_values() {
        let mut map = CaseMap::new();
        map.insert("one", 1);
        map.insert("two", 2);
        map.insert("three", 3);

        let mut values: Vec<_> = map.values().copied().collect();
        values.sort_unstable();

        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn case_map_entry_key() {
        let mut map: CaseMap<i32> = CaseMap::new();

        let entry = map.entry("TestKey");
        assert_eq!(entry.key(), "TestKey");
        entry.or_insert(42);

        let entry = map.entry("TESTKEY");
        assert_eq!(entry.key(), "TestKey"); // Original casing preserved
    }

    #[test]
    fn case_map_entry_or_insert_with() {
        let mut map = CaseMap::new();
        let mut counter = 0;

        map.entry("key").or_insert_with(|| {
            counter += 1;
            100
        });

        assert_eq!(counter, 1);
        assert_eq!(map.get("KEY"), Some(&100));

        // Second call shouldn't execute the closure
        map.entry("KEY").or_insert_with(|| {
            counter += 1;
            200
        });

        assert_eq!(counter, 1); // Counter unchanged
        assert_eq!(map.get("key"), Some(&100));
    }

    #[test]
    fn case_map_empty_string() {
        let mut map = CaseMap::new();
        map.insert("", 42);

        assert_eq!(map.get(""), Some(&42));
        assert!(map.contains_key(""));
        assert_eq!(map.get_key(""), Some(""));
    }

    #[test]
    fn case_map_unicode() {
        let mut map = CaseMap::new();
        // Note: Only ASCII case is handled
        map.insert("café", 1);
        map.insert("CAFÉ", 2);

        // These are different because é doesn't have ASCII case conversion
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn case_set_default() {
        let set: CaseSet = CaseSet::default();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn case_set_clear() {
        let mut set = CaseSet::new();
        set.insert("one");
        set.insert("two");
        set.insert("three");

        assert_eq!(set.len(), 3);

        set.clear();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn case_set_iter() {
        let mut set = CaseSet::new();
        set.insert("Alpha");
        set.insert("BETA");
        set.insert("gamma");

        let mut items: Vec<_> = set.iter().collect();
        items.sort_unstable();

        assert_eq!(items.len(), 3);
        // Original casing preserved
        assert!(items.contains(&"Alpha"));
        assert!(items.contains(&"BETA"));
        assert!(items.contains(&"gamma"));
    }

    #[test]
    fn case_set_duplicate_different_case() {
        let mut set = CaseSet::new();

        assert!(set.insert("Test"));
        assert!(!set.insert("test"));
        assert!(!set.insert("TEST"));
        assert!(!set.insert("TeSt"));

        assert_eq!(set.len(), 1);
        assert_eq!(set.get("test"), Some("Test")); // Original casing
    }

    #[test]
    fn case_set_empty_string() {
        let mut set = CaseSet::new();

        assert!(set.insert(""));
        assert!(set.contains(""));
        assert_eq!(set.get(""), Some(""));

        assert!(set.remove(""));
        assert!(!set.contains(""));
    }

    #[test]
    fn case_set_debug() {
        let mut set = CaseSet::new();
        set.insert("Test");

        let debug_str = format!("{set:?}");
        assert!(debug_str.contains("CaseSet"));
    }

    #[test]
    fn case_insensitive_str_hash_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let s1 = CaseInsensitiveStr("Hello");
        let s2 = CaseInsensitiveStr("HELLO");
        let s3 = CaseInsensitiveStr("hello");
        let s4 = CaseInsensitiveStr("world");

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
        assert_eq!(s1, s3);
        assert_ne!(s1, s4);

        // Hash should be the same for case-insensitive equal strings

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        s1.hash(&mut hasher1);
        s2.hash(&mut hasher2);
        s3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_eq!(hasher2.finish(), hasher3.finish());
    }

    #[test]
    fn case_insensitive_str_debug() {
        let s = CaseInsensitiveStr("Test");
        let debug_str = format!("{s:?}");
        assert!(debug_str.contains("CaseInsensitiveStr"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn case_map_insert_returns_old_value() {
        let mut map = CaseMap::new();

        assert_eq!(map.insert("key", 10), None);
        assert_eq!(map.insert("KEY", 20), Some(10));
        assert_eq!(map.insert("Key", 30), Some(20));
    }

    #[test]
    fn case_map_string_keys() {
        let mut map = CaseMap::new();

        let key1 = String::from("Dynamic");
        let key2 = "Static";

        map.insert(key1, 1);
        map.insert(key2, 2);

        assert_eq!(map.get("dynamic"), Some(&1));
        assert_eq!(map.get("static"), Some(&2));
    }

    #[test]
    fn case_set_string_values() {
        let mut set = CaseSet::new();

        let val1 = String::from("Dynamic");
        let val2 = "Static";

        assert!(set.insert(val1));
        assert!(set.insert(val2));

        assert!(set.contains("dynamic"));
        assert!(set.contains("static"));
    }
}
