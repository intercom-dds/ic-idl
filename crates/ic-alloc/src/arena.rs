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
use std::iter::Enumerate;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicU16, Ordering};
use std::{panic, slice};

use intercom_cts::decode::{Deserializer, FieldDeserializer};
use intercom_cts::encode::{FieldSerializer, Serializer};
use intercom_cts::{Marshal, Unmarshal};

static ARENA_COUNT: AtomicU16 = AtomicU16::new(0);

#[must_use]
pub struct Id<T> {
    id: usize,
    arena_id: ArenaId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    fn new(id: usize, arena_id: ArenaId) -> Self {
        Self {
            id,
            arena_id,
            _marker: PhantomData,
        }
    }

    // Some objects need to be default constructed. This should be avoided
    // where possible, but it's difficult to do so for generated code.
    #[doc(hidden)]
    pub fn _do_not_use() -> Self {
        Self::new(usize::MAX, ArenaId(u16::MAX))
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id);
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.id, f)
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> Marshal for Id<T> {
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.marshal(archive)
    }
}

impl<T> Unmarshal for Id<T> {
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.id.unmarshal_mut(archive)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
struct ArenaId(u16);

#[must_use]
#[derive(Clone, Debug)]
pub struct Arena<T> {
    elements: Vec<T>,
    arena_id: ArenaId,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(len: usize) -> Self {
        Self {
            elements: Vec::with_capacity(len),
            arena_id: ArenaId(ARENA_COUNT.fetch_add(1, Ordering::SeqCst)),
        }
    }

    pub fn alloc(&mut self, value: T) -> Id<T> {
        let index = Id::new(self.elements.len(), self.arena_id);
        self.elements.push(value);
        index
    }

    pub fn alloc_with_id<F>(&mut self, closure: F) -> Id<T>
    where
        F: FnOnce(Id<T>) -> T,
    {
        let index = Id::new(self.elements.len(), self.arena_id);
        let value = closure(index);
        self.elements.push(value);
        index
    }

    /// # Panics
    ///
    /// Panics if the given ID did not come from this arena.
    pub fn get<Q>(&self, id: Q) -> &T
    where
        Q: Borrow<Id<T>>,
    {
        let id = id.borrow();
        assert_eq!(id.arena_id.0, self.arena_id.0);
        &self.elements[id.id]
    }

    /// # Panics
    ///
    /// Panics if the given ID did not come from this arena.
    pub fn get_mut<Q>(&mut self, id: Q) -> &mut T
    where
        Q: Borrow<Id<T>>,
    {
        let id = id.borrow();
        assert_eq!(id.arena_id.0, self.arena_id.0);
        &mut self.elements[id.id]
    }

    /// # Panics
    ///
    /// Panics if the given ID did not come from this arena.
    pub fn fold<Q, F>(&mut self, id: Q, f: F)
    where
        Q: Borrow<Id<T>>,
        F: FnOnce(T) -> T,
    {
        let val = self.get_mut(id);

        // SAFETY: If the given closure panics, the process will be aborted
        // to uphold the variance of T.
        unsafe {
            let old = std::ptr::read(val);
            let Ok(folded) = panic::catch_unwind(panic::AssertUnwindSafe(|| f(old))) else {
                std::process::abort();
            };
            std::ptr::write(val, folded);
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.elements.iter().enumerate(),
            arena_id: self.arena_id,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            iter: self.elements.iter_mut().enumerate(),
            arena_id: self.arena_id,
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Id<T>> for Arena<T> {
    type Output = T;

    fn index(&self, index: Id<T>) -> &Self::Output {
        &self.elements[index.id]
    }
}

impl<T> IndexMut<Id<T>> for Arena<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        &mut self.elements[index.id]
    }
}

#[must_use]
#[derive(Debug)]
pub struct Iter<'a, T> {
    iter: Enumerate<slice::Iter<'a, T>>,
    arena_id: ArenaId,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Id<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(i, v)| (Id::new(i, self.arena_id), v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[must_use]
#[derive(Debug)]
pub struct IterMut<'a, T> {
    iter: Enumerate<slice::IterMut<'a, T>>,
    arena_id: ArenaId,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Id<T>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(i, v)| (Id::new(i, self.arena_id), v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type IntoIter = Iter<'a, T>;

    type Item = (Id<T>, &'a T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type IntoIter = IterMut<'a, T>;

    type Item = (Id<T>, &'a mut T);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: Marshal> Marshal for Arena<T> {
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = archive.encode_struct("Arena<T>")?;
        state.encode_field(0, "elements", &self.elements)?;
        state.end()
    }
}

impl<T: Default + Unmarshal> Unmarshal for Arena<T> {
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        let mut state = archive.decode_struct("Arena<T>")?;
        state.decode_field(0, "elements", &mut self.elements)
    }
}
