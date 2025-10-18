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

use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;

/// A `HashMap` that maintains the insertion order of all entries.
#[must_use]
#[derive(Debug, Clone)]
pub struct IndexMap<K, V> {
    keys: HashMap<K, usize>,
    data: Vec<V>,
}

impl<K, V> IndexMap<K, V> {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            data: vec![],
        }
    }
}

impl<K, V> IndexMap<K, V>
where
    K: Hash + Eq,
{
    pub fn insert(&mut self, key: K, value: V) -> usize {
        match self.keys.entry(key) {
            Entry::Occupied(v) => {
                let id = *v.get();
                self.data[id] = value;
                id
            }
            Entry::Vacant(v) => {
                let id = self.data.len();
                self.data.push(value);
                v.insert(id);
                id
            }
        }
    }

    pub fn insert_multi<I>(&mut self, keys: I, value: V)
    where
        I: IntoIterator<Item = K>,
    {
        self.data.push(value);
        let idx = self.data.len() - 1;
        for k in keys {
            self.keys.insert(k, idx);
        }
    }

    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.keys.get(key).map(|&idx| &self.data[idx])
    }

    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.keys.get(key).map(|&idx| &mut self.data[idx])
    }

    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.keys.contains_key(key)
    }

    #[must_use]
    pub fn values(&self) -> &Vec<V> {
        &self.data
    }

    #[must_use]
    pub fn values_mut(&mut self) -> &mut Vec<V> {
        &mut self.data
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn iter(&self) -> IndexIter<'_, K, V> {
        IndexIter {
            inner: self,
            index: 0,
        }
    }
}

impl<K, V> Default for IndexMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub struct IndexIter<'a, K, V> {
    inner: &'a IndexMap<K, V>,
    index: usize,
}

impl<'a, K, V> Iterator for IndexIter<'a, K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.inner.keys.len() {
            if let Some((key, index)) = self.inner.keys.iter().find(|&(_, v)| *v == self.index) {
                self.index += 1;
                return Some((key, &self.inner.data[*index]));
            }
        }
        None
    }
}

impl<'a, K, V> IntoIterator for &'a IndexMap<K, V>
where
    K: Hash + Eq,
{
    type IntoIter = IndexIter<'a, K, V>;

    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A `HashSet` that maintains the insertion order of all entries.
#[must_use]
#[derive(Debug, Clone)]
pub struct IndexSet<T>(IndexMap<T, ()>);

impl<T> IndexSet<T> {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }
}

impl<T> IndexSet<T>
where
    T: Hash + Eq,
{
    pub fn insert(&mut self, value: T) -> usize {
        self.0.insert(value, ())
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.0.keys.remove(value).is_some()
    }

    #[must_use]
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.0.contains_key(key)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.keys.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.keys.keys()
    }
}

impl<T> Default for IndexSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_map_new() {
        let map: IndexMap<String, i32> = IndexMap::new();
        assert!(map.is_empty());
        assert_eq!(map.values().len(), 0);
    }

    #[test]
    fn test_index_map_default() {
        let map: IndexMap<String, i32> = IndexMap::default();
        assert!(map.is_empty());
    }

    #[test]
    fn test_index_map_insert() {
        let mut map = IndexMap::new();

        let idx1 = map.insert("first", 10);
        let idx2 = map.insert("second", 20);
        let idx3 = map.insert("third", 30);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);

        assert_eq!(map.get("first"), Some(&10));
        assert_eq!(map.get("second"), Some(&20));
        assert_eq!(map.get("third"), Some(&30));
    }

    #[test]
    fn test_index_map_insert_duplicate() {
        let mut map = IndexMap::new();

        let idx1 = map.insert("key", 10);
        let idx2 = map.insert("key", 20); // Should update value

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0); // Same index
        assert_eq!(map.get("key"), Some(&20)); // Updated value
        assert_eq!(map.values().len(), 1); // Still only one entry
    }

    #[test]
    fn test_index_map_get() {
        let mut map = IndexMap::new();
        map.insert("exists", 42);

        assert_eq!(map.get("exists"), Some(&42));
        assert_eq!(map.get("not_exists"), None);

        // Test with borrowed keys
        let key = String::from("exists");
        assert_eq!(map.get(key.as_str()), Some(&42));
    }

    #[test]
    fn test_index_map_get_mut() {
        let mut map = IndexMap::new();
        map.insert("key", 10);

        if let Some(value) = map.get_mut("key") {
            *value = 20;
        }

        assert_eq!(map.get("key"), Some(&20));
    }

    #[test]
    fn test_index_map_contains_key() {
        let mut map = IndexMap::new();
        map.insert("exists", 42);

        assert!(map.contains_key("exists"));
        assert!(!map.contains_key("not_exists"));
    }

    #[test]
    fn test_index_map_values() {
        let mut map = IndexMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        let values = map.values();
        assert_eq!(values, &vec![1, 2, 3]);
    }

    #[test]
    fn test_index_map_values_mut() {
        let mut map = IndexMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        let values = map.values_mut();
        for v in values.iter_mut() {
            *v *= 2;
        }

        assert_eq!(map.get("a"), Some(&2));
        assert_eq!(map.get("b"), Some(&4));
        assert_eq!(map.get("c"), Some(&6));
    }

    #[test]
    fn test_index_map_iter() {
        let mut map = IndexMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        let mut items: Vec<_> = map.iter().collect();
        items.sort_by_key(|(_, v)| **v); // Sort by value for consistent ordering

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].1, &1);
        assert_eq!(items[1].1, &2);
        assert_eq!(items[2].1, &3);
    }

    #[test]
    fn test_index_map_into_iter() {
        let mut map = IndexMap::new();
        map.insert("a", 1);
        map.insert("b", 2);

        let items: Vec<_> = (&map).into_iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_index_map_clone() {
        let mut map = IndexMap::new();
        map.insert("a", 1);
        map.insert("b", 2);

        let cloned = map.clone();
        assert_eq!(cloned.get("a"), Some(&1));
        assert_eq!(cloned.get("b"), Some(&2));
    }

    #[test]
    fn test_index_map_debug() {
        let mut map = IndexMap::new();
        map.insert("key", 42);

        let debug_str = format!("{map:?}");
        assert!(debug_str.contains("IndexMap"));
    }

    #[test]
    fn test_index_set_new() {
        let set: IndexSet<i32> = IndexSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_index_set_default() {
        let set: IndexSet<i32> = IndexSet::default();
        assert!(set.is_empty());
    }

    #[test]
    fn test_index_set_insert() {
        let mut set = IndexSet::new();

        let idx1 = set.insert(10);
        let idx2 = set.insert(20);
        let idx3 = set.insert(30);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);

        assert!(set.contains(&10));
        assert!(set.contains(&20));
        assert!(set.contains(&30));
        assert!(!set.contains(&40));

        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_index_set_insert_duplicate() {
        let mut set = IndexSet::new();

        let idx1 = set.insert(42);
        let idx2 = set.insert(42); // Should return same index

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_index_set_contains() {
        let mut set = IndexSet::new();
        set.insert(10);
        set.insert(20);

        assert!(set.contains(&10));
        assert!(set.contains(&20));
        assert!(!set.contains(&30));
    }

    #[test]
    fn test_index_set_clone() {
        let mut set = IndexSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let cloned = set.clone();
        assert!(cloned.contains(&1));
        assert!(cloned.contains(&2));
        assert!(cloned.contains(&3));
        assert_eq!(cloned.len(), 3);
    }

    #[test]
    fn test_index_set_debug() {
        let mut set = IndexSet::new();
        set.insert(42);

        let debug_str = format!("{set:?}");
        assert!(debug_str.contains("IndexSet"));
    }

    #[test]
    fn test_iter_bug() {
        // The iterator implementation has a potential issue -
        // it searches for keys by their index value on each iteration
        let mut map = IndexMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        // If we update a value, it should still iterate correctly
        map.insert("b", 20);

        let items: Vec<_> = map.iter().map(|(_, v)| *v).collect();
        assert!(items.contains(&1));
        assert!(items.contains(&20));
        assert!(items.contains(&3));
    }
}
