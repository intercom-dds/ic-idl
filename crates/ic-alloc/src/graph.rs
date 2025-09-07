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
            set.insert(v);
            self.edges.insert(u, set);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let graph: DiGraph<i32> = DiGraph::new();
        assert_eq!(graph.len(), 0);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_default() {
        let graph: DiGraph<i32> = DiGraph::default();
        assert_eq!(graph.len(), 0);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_add_vertex() {
        let mut graph = DiGraph::new();

        let v1 = graph.add_vertex(10);
        assert_eq!(graph.len(), 1);
        assert!(!graph.is_empty());

        let v2 = graph.add_vertex(20);
        assert_eq!(graph.len(), 2);

        let v3 = graph.add_vertex(30);
        assert_eq!(graph.len(), 3);

        // Verify IDs are sequential
        assert_eq!(v1, VertexId(0));
        assert_eq!(v2, VertexId(1));
        assert_eq!(v3, VertexId(2));
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DiGraph::new();
        let v1 = graph.add_vertex("A");
        let v2 = graph.add_vertex("B");

        // Add edge from v1 to v2
        graph.add_edge(v1, v2);

        // v1 should be the key and v2 should be in the set
        assert!(graph.edges.contains_key(&v1));
        assert!(graph.edges.get(&v1).unwrap().contains(&v2));
        assert!(!graph.edges.contains_key(&v2));
    }

    #[test]
    fn test_vertex_id_traits() {
        use std::collections::HashSet;

        let id1 = VertexId(1);
        let id2 = VertexId(2);
        let id1_copy = VertexId(1);

        // Test Eq and PartialEq
        assert_eq!(id1, id1_copy);
        assert_ne!(id1, id2);

        // Test Clone
        let cloned = id1;
        assert_eq!(id1, cloned);

        // Test Copy
        let copied = id1;
        assert_eq!(id1, copied);

        // Test Debug
        let debug_str = format!("{id1:?}");
        assert!(debug_str.contains("VertexId"));
        assert!(debug_str.contains('1'));

        // Test Hash
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id1_copy);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_multiple_vertices() {
        let mut graph = DiGraph::new();

        let vertices: Vec<_> = (0..10).map(|i| graph.add_vertex(i * 10)).collect();

        assert_eq!(graph.len(), 10);

        for (i, &vid) in vertices.iter().enumerate() {
            assert_eq!(vid, VertexId(i));
        }
    }

    #[test]
    fn test_add_multiple_edges() {
        let mut graph = DiGraph::new();
        let v1 = graph.add_vertex("A");
        let v2 = graph.add_vertex("B");
        let v3 = graph.add_vertex("C");

        // Add edges
        graph.add_edge(v1, v2);
        graph.add_edge(v1, v3);
        graph.add_edge(v2, v3);

        // Check edges exist
        assert_eq!(graph.edges.len(), 2); // v1 and v2 as keys
        assert!(graph.edges.get(&v1).unwrap().contains(&v2));
        assert!(graph.edges.get(&v1).unwrap().contains(&v3));
        assert!(graph.edges.get(&v2).unwrap().contains(&v3));
    }

    #[test]
    fn test_self_loop() {
        let mut graph = DiGraph::new();
        let v1 = graph.add_vertex("A");

        // Add self-loop
        graph.add_edge(v1, v1);

        // v1 is both key and in the set
        assert!(graph.edges.contains_key(&v1));
        assert!(graph.edges.get(&v1).unwrap().contains(&v1));
    }

    #[test]
    fn test_empty_graph_functions() {
        let graph: DiGraph<i32> = DiGraph::new();

        // These should not panic on empty graph
        post_order(&graph);
        topological_sort(&graph);
    }

    #[test]
    fn test_graph_with_edges_functions() {
        let mut graph = DiGraph::new();
        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        graph.add_edge(v1, v2);

        // These should not panic
        post_order(&graph);
        topological_sort(&graph);
    }
}
