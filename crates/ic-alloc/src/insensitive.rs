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

//! Case-insensitive strings and maps.

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A copy-on-write string type that always performs case-insensitive
/// comparisons and hashing.
#[derive(Clone, Debug, Default, Eq)]
pub struct CaseString<'a>(Cow<'a, str>);

impl AsRef<str> for CaseString<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for CaseString<'_> {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

impl<S> PartialEq<S> for CaseString<'_>
where
    S: AsRef<str>,
{
    #[inline]
    fn eq(&self, other: &S) -> bool {
        self.0.eq_ignore_ascii_case(other.as_ref())
    }
}

impl Hash for CaseString<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0
            .bytes()
            .map(|v| v.to_ascii_lowercase())
            .for_each(|b| hasher.write_u8(b));
    }
}

/// A case-insensitive map that stores the key in its initial form, but
/// performs case-insensitive hashing and lookups.
#[must_use]
#[derive(Debug)]
pub struct CaseMap<'a, T>(HashMap<CaseString<'a>, T>);

impl<'a, T> CaseMap<'a, T>
where
    T: Hash + Eq,
{
    pub fn insert<K>(&mut self, key: K, value: T) -> Option<T>
    where
        K: Into<Cow<'a, str>>,
    {
        self.0.insert(CaseString(key.into()), value)
    }

    pub fn remove(&mut self, key: &'a str) -> Option<T> {
        self.0.remove(&CaseString(Cow::Borrowed(key)))
    }

    #[must_use]
    pub fn get(&self, key: &'a str) -> Option<&T> {
        self.0.get(&CaseString(Cow::Borrowed(key)))
    }

    #[must_use]
    pub fn get_mut(&mut self, key: &'a str) -> Option<&mut T> {
        self.0.get_mut(&CaseString(Cow::Borrowed(key)))
    }

    pub fn entry<K>(&mut self, key: K) -> Entry<'_, CaseString<'a>, T>
    where
        K: Into<Cow<'a, str>>,
    {
        self.0.entry(CaseString(key.into()))
    }
}

impl<T> Default for CaseMap<'_, T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insensitive_key() {
        let mut map = CaseMap::default();
        map.insert("foo", 1);
        assert!(map.get("foo").is_some());
        assert!(map.get("FOo").is_some());
    }

    #[test]
    fn insensitive_entry() {
        let mut map = CaseMap::default();
        map.insert("foo", 1);

        match map.entry("FoO") {
            Entry::Occupied(v) => {
                assert_eq!(v.key().as_ref(), "foo");
                assert_eq!(*v.get(), 1);
            }
            Entry::Vacant(_) => unreachable!(),
        }
    }
}
