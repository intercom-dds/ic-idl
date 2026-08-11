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

use std::collections::HashMap;
use std::hash::Hash;

use crate::index::IndexMap;

#[must_use]
#[derive(Debug)]
pub struct DiGraph<T, E = ()> {
    nodes: Vec<T>,
    index: HashMap<T, u32>,
    edges: Vec<IndexMap<u32, E>>,
}

struct Csr {
    offsets: Vec<u32>,
    targets: Vec<u32>,
}

impl<T, E> DiGraph<T, E>
where
    T: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn add_node(&mut self, node: T) {
        if self.index.contains_key(&node) {
            return;
        }

        self.index.insert(node.clone(), self.nodes.len() as u32);
        self.nodes.push(node);
        self.edges.push(IndexMap::new());
    }

    pub fn add_edge(&mut self, from: &T, to: &T, weight: E) -> bool {
        let (Some(u), Some(v)) = (self.id_of(from), self.id_of(to)) else {
            return false;
        };

        self.edges[u as usize].insert(v, weight);
        true
    }

    pub fn edge_mut(&mut self, from: &T, to: &T) -> Option<&mut E> {
        let u = self.id_of(from)?;
        let v = self.id_of(to)?;
        self.edges[u as usize].get_mut(&v)
    }

    pub fn edge_or_default(&mut self, from: &T, to: &T) -> Option<&mut E>
    where
        E: Default,
    {
        let u = self.id_of(from)?;
        let v = self.id_of(to)?;

        let adjacency = &mut self.edges[u as usize];
        if !adjacency.contains_key(&v) {
            adjacency.insert(v, E::default());
        }

        adjacency.get_mut(&v)
    }

    fn id_of(&self, node: &T) -> Option<u32> {
        self.index.get(node).copied()
    }

    pub fn edge(&self, from: &T, to: &T) -> Option<&E> {
        let u = self.id_of(from)?;
        let v = self.id_of(to)?;
        self.edges[u as usize].get(&v)
    }

    pub fn has_edge(&self, from: &T, to: &T) -> bool {
        self.edge(from, to).is_some()
    }

    pub fn contains(&self, node: &T) -> bool {
        self.index.contains_key(node)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter()
    }

    pub fn neighbors(&self, of: &T) -> impl Iterator<Item = (&T, &E)> {
        self.index
            .get(of)
            .map(|u| self.edges[*u as usize].iter())
            .into_iter()
            .flatten()
            .map(|(v, weight)| (&self.nodes[*v as usize], weight))
    }

    #[must_use]
    pub fn scc_tarjan(&self) -> Vec<Vec<T>> {
        let keep = vec![true; self.nodes.len()];
        self.resolve(&tarjan(&self.csr(&keep), &keep))
    }

    #[must_use]
    pub fn scc_kosaraju(&self) -> Vec<Vec<T>> {
        let keep = vec![true; self.nodes.len()];
        self.resolve(&kosaraju(&self.csr(&keep), &self.reverse_csr(&keep), &keep))
    }

    #[must_use]
    pub fn cyclic_scc(&self) -> Vec<Vec<T>> {
        let keep = vec![true; self.nodes.len()];
        let csr = self.csr(&keep);

        let cyclic: Vec<_> = tarjan(&csr, &keep)
            .into_iter()
            .filter(|component| is_cyclic(&csr, component))
            .collect();

        self.resolve(&cyclic)
    }

    pub fn cyclic_scc_where(&self, keep: impl Fn(&T) -> bool) -> Vec<Vec<T>> {
        let keep: Vec<_> = self.nodes.iter().map(keep).collect();
        let csr = self.csr(&keep);

        let cyclic: Vec<_> = tarjan(&csr, &keep)
            .into_iter()
            .filter(|component| is_cyclic(&csr, component))
            .collect();

        self.resolve(&cyclic)
    }

    fn resolve(&self, components: &[Vec<u32>]) -> Vec<Vec<T>> {
        components
            .iter()
            .map(|component| {
                component
                    .iter()
                    .map(|id| self.nodes[*id as usize].clone())
                    .collect()
            })
            .collect()
    }

    fn csr(&self, keep: &[bool]) -> Csr {
        let mut offsets = Vec::with_capacity(self.nodes.len() + 1);
        let mut targets = vec![];

        for (u, adjacency) in self.edges.iter().enumerate() {
            offsets.push(u32::try_from(targets.len()).expect("edge count exceeded u32::MAX"));

            if !keep[u] {
                continue;
            }

            for (v, _) in adjacency {
                if keep[*v as usize] {
                    targets.push(*v);
                }
            }
        }

        offsets.push(u32::try_from(targets.len()).expect("edge count exceeded u32::MAX"));
        Csr { offsets, targets }
    }

    fn reverse_csr(&self, keep: &[bool]) -> Csr {
        let mut degrees = vec![0u32; self.nodes.len()];
        for (u, adjacency) in self.edges.iter().enumerate() {
            if !keep[u] {
                continue;
            }
            for (v, _) in adjacency {
                if keep[*v as usize] {
                    degrees[*v as usize] += 1;
                }
            }
        }

        let mut offsets = Vec::with_capacity(self.nodes.len() + 1);
        let mut total = 0u32;
        for degree in &degrees {
            offsets.push(total);
            total += degree;
        }
        offsets.push(total);

        let mut cursors = offsets.clone();
        let mut targets = vec![0u32; total as usize];
        for (u, adjacency) in self.edges.iter().enumerate() {
            if !keep[u] {
                continue;
            }
            for (v, _) in adjacency {
                if keep[*v as usize] {
                    let slot = &mut cursors[*v as usize];
                    targets[*slot as usize] = u32::try_from(u).expect("node id exceeded u32::MAX");
                    *slot += 1;
                }
            }
        }

        Csr { offsets, targets }
    }
}

impl Csr {
    fn neighbors(&self, v: u32) -> &[u32] {
        let start = self.offsets[v as usize] as usize;
        let end = self.offsets[v as usize + 1] as usize;
        &self.targets[start..end]
    }
}

fn is_cyclic(csr: &Csr, component: &[u32]) -> bool {
    if component.len() > 1 {
        return true;
    }

    let v = component[0];
    csr.neighbors(v).contains(&v)
}

fn tarjan(csr: &Csr, keep: &[bool]) -> Vec<Vec<u32>> {
    const UNVISITED: u32 = u32::MAX;

    let count = keep.len();
    let mut index = vec![UNVISITED; count];
    let mut low = vec![0u32; count];
    let mut on_stack = vec![false; count];

    let mut stack = vec![];
    let mut frames = vec![];
    let mut next = 0u32;
    let mut components = vec![];

    for root in 0..count {
        if !keep[root] || index[root] != UNVISITED {
            continue;
        }

        let root = u32::try_from(root).expect("node id exceeded u32::MAX");
        visit(
            root,
            &mut index,
            &mut low,
            &mut on_stack,
            &mut stack,
            &mut next,
        );
        frames.push((root, 0));

        while let Some((v, cursor)) = frames.last_mut() {
            let v = *v;
            let neighbors = csr.neighbors(v);

            if *cursor < neighbors.len() {
                let w = neighbors[*cursor];
                *cursor += 1;

                if index[w as usize] == UNVISITED {
                    visit(
                        w,
                        &mut index,
                        &mut low,
                        &mut on_stack,
                        &mut stack,
                        &mut next,
                    );
                    frames.push((w, 0));
                } else if on_stack[w as usize] {
                    low[v as usize] = low[v as usize].min(index[w as usize]);
                }

                continue;
            }

            frames.pop();

            if let Some((parent, _)) = frames.last() {
                let parent = *parent as usize;
                low[parent] = low[parent].min(low[v as usize]);
            }

            if low[v as usize] == index[v as usize] {
                let mut component = vec![];
                loop {
                    let w = stack.pop().expect("tarjan stack should not be empty");
                    on_stack[w as usize] = false;
                    component.push(w);
                    if w == v {
                        break;
                    }
                }
                components.push(component);
            }
        }
    }

    components
}

fn visit(
    v: u32,
    index: &mut [u32],
    low: &mut [u32],
    on_stack: &mut [bool],
    stack: &mut Vec<u32>,
    next: &mut u32,
) {
    index[v as usize] = *next;
    low[v as usize] = *next;
    *next += 1;
    stack.push(v);
    on_stack[v as usize] = true;
}

fn kosaraju(csr: &Csr, reverse: &Csr, keep: &[bool]) -> Vec<Vec<u32>> {
    let count = keep.len();

    let mut visited = vec![false; count];
    let mut order = vec![];
    let mut frames = vec![];

    for root in 0..count {
        if !keep[root] || visited[root] {
            continue;
        }

        let root = u32::try_from(root).expect("node id exceeded u32::MAX");
        visited[root as usize] = true;
        frames.push((root, 0));

        while let Some((v, cursor)) = frames.last_mut() {
            let v = *v;
            let neighbors = csr.neighbors(v);

            if *cursor < neighbors.len() {
                let w = neighbors[*cursor];
                *cursor += 1;

                if !visited[w as usize] {
                    visited[w as usize] = true;
                    frames.push((w, 0));
                }

                continue;
            }

            frames.pop();
            order.push(v);
        }
    }

    let mut assigned = vec![false; count];
    let mut components = vec![];

    for root in order.iter().rev().copied() {
        if assigned[root as usize] {
            continue;
        }

        let mut component = vec![];
        let mut pending = vec![root];
        assigned[root as usize] = true;

        while let Some(v) = pending.pop() {
            component.push(v);

            for w in reverse.neighbors(v).iter().copied() {
                if !assigned[w as usize] {
                    assigned[w as usize] = true;
                    pending.push(w);
                }
            }
        }

        components.push(component);
    }

    components
}

impl<T, E> Default for DiGraph<T, E> {
    fn default() -> Self {
        Self {
            nodes: vec![],
            index: HashMap::new(),
            edges: vec![],
        }
    }
}

pub fn post_order<T, E>(_graph: &DiGraph<T, E>) {}

pub fn topological_sort<T, E>(_graph: &DiGraph<T, E>) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_of(edges: &[(i32, i32)], isolated: &[i32]) -> DiGraph<i32> {
        let mut graph = DiGraph::new();

        for (from, to) in edges {
            graph.add_node(*from);
            graph.add_node(*to);
        }

        for node in isolated {
            graph.add_node(*node);
        }

        for (from, to) in edges {
            assert!(graph.add_edge(from, to, ()));
        }

        graph
    }

    fn normalize<T: Ord>(components: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let mut components: Vec<_> = components
            .into_iter()
            .map(|mut component| {
                component.sort();
                component
            })
            .collect();

        components.sort();
        components
    }

    fn sizes<T>(components: &[Vec<T>]) -> Vec<usize> {
        let mut sizes: Vec<_> = components.iter().map(Vec::len).collect();
        sizes.sort_unstable();
        sizes
    }

    #[test]
    fn new_graph_is_empty() {
        let graph: DiGraph<i32> = DiGraph::new();
        assert_eq!(graph.len(), 0);
        assert!(graph.is_empty());
    }

    #[test]
    fn add_node_is_idempotent() {
        let mut graph: DiGraph<&str> = DiGraph::new();

        graph.add_node("A");
        graph.add_node("A");

        assert_eq!(graph.len(), 1);
        assert!(graph.contains(&"A"));
    }

    #[test]
    fn add_edge_records_direction_only() {
        let graph = graph_of(&[(1, 2)], &[]);

        assert!(graph.has_edge(&1, &2));
        assert!(!graph.has_edge(&2, &1));
    }

    #[test]
    fn add_edge_rejects_unknown_endpoints() {
        let mut graph: DiGraph<&str> = DiGraph::new();
        graph.add_node("A");

        assert!(!graph.add_edge(&"A", &"missing", ()));
        assert!(!graph.add_edge(&"missing", &"A", ()));

        assert_eq!(graph.len(), 1);
        assert!(!graph.contains(&"missing"));
    }

    #[test]
    fn add_edge_replaces_the_weight() {
        let mut graph: DiGraph<&str, u32> = DiGraph::new();
        graph.add_node("A");
        graph.add_node("B");

        graph.add_edge(&"A", &"B", 1);
        graph.add_edge(&"A", &"B", 2);

        assert_eq!(graph.edge(&"A", &"B"), Some(&2));
    }

    #[test]
    fn edge_or_default_accumulates() {
        let mut graph: DiGraph<&str, Vec<u32>> = DiGraph::new();
        graph.add_node("A");
        graph.add_node("B");

        graph.edge_or_default(&"A", &"B").unwrap().push(1);
        graph.edge_or_default(&"A", &"B").unwrap().push(2);

        assert_eq!(graph.edge(&"A", &"B"), Some(&vec![1, 2]));
    }

    #[test]
    fn edge_or_default_rejects_unknown_endpoints() {
        let mut graph: DiGraph<&str, Vec<u32>> = DiGraph::new();
        graph.add_node("A");

        assert!(graph.edge_or_default(&"A", &"missing").is_none());
        assert!(!graph.contains(&"missing"));
    }

    #[test]
    fn edge_mut_does_not_insert() {
        let mut graph: DiGraph<&str, u32> = DiGraph::new();
        graph.add_node("A");
        graph.add_node("B");

        assert!(graph.edge_mut(&"A", &"B").is_none());
        assert!(!graph.has_edge(&"A", &"B"));
    }

    #[test]
    fn neighbors_yields_nodes_and_weights() {
        let mut graph: DiGraph<&str, u32> = DiGraph::new();
        for node in ["A", "B", "C"] {
            graph.add_node(node);
        }
        graph.add_edge(&"A", &"B", 7);
        graph.add_edge(&"A", &"C", 9);

        let found: Vec<_> = graph.neighbors(&"A").map(|(n, w)| (*n, *w)).collect();
        assert_eq!(found, vec![("B", 7), ("C", 9)]);
    }

    #[test]
    fn chain_has_no_cyclic_components() {
        let graph = graph_of(&[(1, 2), (2, 3), (3, 4)], &[]);

        assert_eq!(graph.scc_tarjan().len(), 4);
        assert!(graph.cyclic_scc().is_empty());
    }

    #[test]
    fn simple_cycle_is_one_component() {
        let graph = graph_of(&[(1, 2), (2, 3), (3, 1)], &[]);

        assert_eq!(sizes(&graph.scc_tarjan()), vec![3]);
        assert_eq!(sizes(&graph.cyclic_scc()), vec![3]);
    }

    #[test]
    fn self_loop_is_cyclic_but_isolated_node_is_not() {
        let graph = graph_of(&[(1, 1)], &[2]);

        assert_eq!(graph.scc_tarjan().len(), 2);
        assert_eq!(graph.cyclic_scc(), vec![vec![1]]);
    }

    #[test]
    fn multiple_components_are_separated() {
        let graph = graph_of(&[(1, 2), (2, 1), (3, 4), (4, 3), (5, 6), (6, 5)], &[]);

        assert_eq!(sizes(&graph.scc_tarjan()), vec![2, 2, 2]);
    }

    #[test]
    fn two_cycles_joined_by_a_bridge() {
        let graph = graph_of(
            &[(1, 2), (2, 3), (3, 1), (3, 4), (4, 5), (5, 6), (6, 4)],
            &[],
        );

        assert_eq!(sizes(&graph.scc_tarjan()), vec![3, 3]);
    }

    #[test]
    fn isolated_node_forms_its_own_component() {
        let graph = graph_of(&[(1, 2), (2, 1)], &[3]);

        assert_eq!(sizes(&graph.scc_tarjan()), vec![1, 2]);
    }

    #[test]
    fn tarjan_and_kosaraju_agree() {
        let graph = graph_of(
            &[
                (1, 2),
                (2, 3),
                (3, 1),
                (3, 4),
                (4, 5),
                (5, 4),
                (5, 6),
                (7, 7),
            ],
            &[8],
        );

        assert_eq!(
            normalize(graph.scc_tarjan()),
            normalize(graph.scc_kosaraju())
        );
    }

    #[test]
    fn cyclic_scc_where_uses_the_induced_subgraph() {
        let graph = graph_of(&[(1, 2), (2, 1), (2, 3), (3, 2)], &[]);

        assert_eq!(sizes(&graph.cyclic_scc()), vec![3]);

        // Dropping 2 removes the only path between 1 and 3, so nothing cycles.
        assert!(graph.cyclic_scc_where(|n| *n != 2).is_empty());
    }

    #[test]
    fn deep_chain_does_not_overflow_the_stack() {
        let mut graph: DiGraph<u32> = DiGraph::new();
        for i in 0..=500_000 {
            graph.add_node(i);
        }
        for i in 0..500_000 {
            graph.add_edge(&i, &(i + 1), ());
        }

        assert_eq!(graph.scc_tarjan().len(), 500_001);
        assert_eq!(graph.scc_kosaraju().len(), 500_001);
        assert!(graph.cyclic_scc().is_empty());
    }
}
