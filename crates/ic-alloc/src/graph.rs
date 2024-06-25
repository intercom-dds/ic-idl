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

use std::hash::Hash;

use crate::index::{IndexMap, IndexSet};

#[must_use]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct VertexId(usize);

/// A directed graph implementation.
#[must_use]
#[derive(Debug)]
pub struct DiGraph<T> {
    vertices: Vec<T>,
    edges: IndexMap<VertexId, IndexSet<VertexId>>,
}

impl<T> DiGraph<T>
where
    T: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_edge(&mut self, u: VertexId, v: VertexId) {
        if let Some(entry) = self.edges.get_mut(&u) {
            entry.insert(v);
        } else {
            let mut set = IndexSet::new();
            set.insert(u);
            self.edges.insert(v, set);
        }
    }

    /// Adds a vertex to the graph.
    pub fn add_vertex(&mut self, vertex: T) -> VertexId {
        let idx = self.vertices.len();
        self.vertices.push(vertex);
        VertexId(idx)
    }

    /// Returns the number of vertices in the graph.
    #[must_use]
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl<T> Default for DiGraph<T> {
    fn default() -> Self {
        Self {
            vertices: vec![],
            edges: IndexMap::new(),
        }
    }
}

pub fn post_order<T>(_graph: &DiGraph<T>) {}

pub fn topological_sort<T>(_graph: &DiGraph<T>) {}
