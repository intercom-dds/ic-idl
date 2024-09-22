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

    pub fn iter(&self) -> IndexIter<K, V> {
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
            if let Some((key, index)) = self.inner.keys.iter().find(|(_, &v)| v == self.index) {
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
}

impl<T> Default for IndexSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
