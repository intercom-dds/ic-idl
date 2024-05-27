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
use std::slice;

#[must_use]
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Id<T> {
    id: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    fn new(id: usize) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

#[must_use]
#[derive(Debug)]
pub struct Arena<T> {
    elements: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn with_capacity(len: usize) -> Self {
        Self {
            elements: Vec::with_capacity(len),
        }
    }

    pub fn alloc(&mut self, value: T) -> Id<T> {
        let index = Id::new(self.elements.len());
        self.elements.push(value);
        index
    }

    pub fn get<Q>(&self, id: Q) -> Option<&T>
    where
        Q: Borrow<Id<T>>,
    {
        self.elements.get(id.borrow().id)
    }

    pub fn get_mut<Q>(&mut self, id: Q) -> Option<&mut T>
    where
        Q: Borrow<Id<T>>,
    {
        self.elements.get_mut(id.borrow().id)
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
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            iter: self.elements.iter_mut().enumerate(),
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
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Id<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(i, v)| (Id::new(i), v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[must_use]
#[derive(Debug)]
pub struct IterMut<'a, T> {
    iter: Enumerate<slice::IterMut<'a, T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Id<T>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(i, v)| (Id::new(i), v))
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
