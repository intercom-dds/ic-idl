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

use std::collections::hash_map::Entry;
use std::collections::HashMap;
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
    cache: HashMap<u64, SymbolId>,
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
    pub fn get(&self, id: SymbolId) -> Option<&str> {
        self.arena.get(id).map(|v| v.string.as_ref())
    }

    pub fn insert<I>(&mut self, str: I) -> SymbolId
    where
        I: Into<Box<str>>,
    {
        let cached = CachedStr::new(str.into());

        match self.cache.entry(cached.hash) {
            Entry::Occupied(v) => *v.get(),
            Entry::Vacant(v) => {
                let id = self.arena.alloc(cached);
                v.insert(id);
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

        assert_eq!(interner.get(id).unwrap(), str);
    }
}
