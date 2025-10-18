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
use std::iter::Enumerate;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::sync::atomic::{AtomicU16, Ordering};
use std::{panic, slice};

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

impl<T> From<Id<T>> for usize {
    fn from(value: Id<T>) -> Self {
        value.id
    }
}

// Bad idea, but necessary at the moment
impl<T> From<usize> for Id<T> {
    fn from(value: usize) -> Self {
        Self::new(value, ArenaId(u16::MAX))
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

impl<T> Deref for Arena<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        &self.elements
    }
}

impl<T> DerefMut for Arena<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.elements
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_new() {
        let arena: Arena<i32> = Arena::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_arena_with_capacity() {
        let arena: Arena<i32> = Arena::with_capacity(100);
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_alloc() {
        let mut arena = Arena::new();
        let id1 = arena.alloc(42);
        let id2 = arena.alloc(100);

        assert_eq!(arena.len(), 2);
        assert!(!arena.is_empty());
        assert_eq!(*arena.get(id1), 42);
        assert_eq!(*arena.get(id2), 100);
    }

    #[test]
    fn test_alloc_with_id() {
        let mut arena = Arena::new();
        let id = arena.alloc_with_id(|id| format!("Item {}", usize::from(id)));

        assert_eq!(*arena.get(id), "Item 0");
    }

    #[test]
    fn test_get_mut() {
        let mut arena = Arena::new();
        let id = arena.alloc(42);

        *arena.get_mut(id) = 100;
        assert_eq!(*arena.get(id), 100);
    }

    #[test]
    fn test_fold() {
        let mut arena = Arena::new();
        let id = arena.alloc(10);

        arena.fold(id, |x| x * 2);
        assert_eq!(*arena.get(id), 20);

        arena.fold(&id, |x| x + 5);
        assert_eq!(*arena.get(id), 25);
    }

    #[test]
    fn test_index_operators() {
        let mut arena = Arena::new();
        let id = arena.alloc(42);

        assert_eq!(arena[id], 42);
        arena[id] = 100;
        assert_eq!(arena[id], 100);
    }

    #[test]
    fn test_iter() {
        let mut arena = Arena::new();
        let id1 = arena.alloc(1);
        let id2 = arena.alloc(2);
        let id3 = arena.alloc(3);

        let items: Vec<_> = arena.iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], (id1, &1));
        assert_eq!(items[1], (id2, &2));
        assert_eq!(items[2], (id3, &3));
    }

    #[test]
    fn test_iter_mut() {
        let mut arena = Arena::new();
        let _ = arena.alloc(1);
        let _ = arena.alloc(2);
        let _ = arena.alloc(3);

        for (_, value) in &mut arena {
            *value *= 2;
        }

        let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec![2, 4, 6]);
    }

    #[test]
    fn test_deref() {
        let mut arena = Arena::new();
        let _ = arena.alloc(1);
        let _ = arena.alloc(2);
        let _ = arena.alloc(3);

        let slice: &[i32] = &arena;
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_deref_mut() {
        let mut arena = Arena::new();
        let _ = arena.alloc(1);
        let _ = arena.alloc(2);
        let _ = arena.alloc(3);

        let slice: &mut [i32] = &mut arena;
        slice[1] = 20;
        assert_eq!(slice, &[1, 20, 3]);
    }

    #[test]
    fn test_id_equality() {
        let mut arena = Arena::new();
        let id1 = arena.alloc(42);
        let id2 = arena.alloc(42);
        let id1_copy = id1;

        assert_eq!(id1, id1_copy);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_ordering() {
        let mut arena = Arena::new();
        let id1 = arena.alloc(1);
        let id2 = arena.alloc(2);
        let id3 = arena.alloc(3);

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
    }

    #[test]
    fn test_id_hash() {
        use std::collections::HashSet;

        let mut arena = Arena::new();
        let id1 = arena.alloc(1);
        let id2 = arena.alloc(2);

        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id1); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }

    #[test]
    fn test_id_debug() {
        let mut arena = Arena::new();
        let id = arena.alloc(42);

        let debug_str = format!("{id:?}");
        assert!(debug_str.contains('0')); // First allocation has id 0
    }

    #[test]
    fn test_id_from_usize() {
        let id: Id<i32> = 42usize.into();
        assert_eq!(usize::from(id), 42);
    }

    #[test]
    fn test_id_do_not_use() {
        let id: Id<i32> = Id::_do_not_use();
        assert_eq!(usize::from(id), usize::MAX);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_arena_id_mismatch() {
        let mut arena1 = Arena::new();
        let arena2 = Arena::new();

        let id = arena1.alloc(42);
        arena2.get(id); // Should panic due to arena ID mismatch
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_arena_id_mismatch_mut() {
        let mut arena1 = Arena::new();
        let mut arena2 = Arena::new();

        let id = arena1.alloc(42);
        arena2.get_mut(id); // Should panic due to arena ID mismatch
    }

    #[test]
    fn test_into_iter_ref() {
        let mut arena = Arena::new();
        let id1 = arena.alloc(1);
        let id2 = arena.alloc(2);

        let items: Vec<_> = (&arena).into_iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], (id1, &1));
        assert_eq!(items[1], (id2, &2));
    }

    #[test]
    fn test_into_iter_mut() {
        let mut arena = Arena::new();
        let _ = arena.alloc(1);
        let _ = arena.alloc(2);

        for (_, value) in &mut arena {
            *value *= 2;
        }

        let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec![2, 4]);
    }

    #[test]
    fn test_iter_size_hint() {
        let mut arena = Arena::new();
        let _ = arena.alloc(1);
        let _ = arena.alloc(2);
        let _ = arena.alloc(3);

        let iter = arena.iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_default() {
        let arena: Arena<i32> = Arena::default();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_clone() {
        let mut arena = Arena::new();
        let _ = arena.alloc(1);
        let _ = arena.alloc(2);
        let _ = arena.alloc(3);

        let cloned = arena.clone();
        assert_eq!(cloned.len(), 3);
        assert_eq!(cloned[0.into()], 1);
        assert_eq!(cloned[1.into()], 2);
        assert_eq!(cloned[2.into()], 3);
    }

    #[test]
    fn test_multiple_arenas() {
        let mut arena1 = Arena::new();
        let mut arena2 = Arena::new();

        let id1 = arena1.alloc(10);
        let id2 = arena2.alloc(20);

        assert_eq!(*arena1.get(id1), 10);
        assert_eq!(*arena2.get(id2), 20);
    }
}
