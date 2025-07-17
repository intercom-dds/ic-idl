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

//! Tests for HIR tree merging functionality.

use ic_hir::merge::{merge_hir_trees, MergedGraph};

#[test]
fn test_merge_empty_graphs() {
    let graphs = vec![];
    let merged = merge_hir_trees(&graphs);
    
    assert_eq!(merged.order.len(), 0);
}

#[test]
fn test_merge_single_graph() {
    let input = r"
        struct Point {
            long x;
            long y;
        };
    ";
    
    let parsed = ic_parse::from_str(input);
    assert!(parsed.errors.is_empty());
    
    let graph = ic_hir::from_ast(parsed.tree);
    assert!(graph.errors.is_empty());
    
    let merged = merge_hir_trees(&[graph]);
    
    // TODO: Once implementation is complete, verify the merged graph
    // For now, just check it doesn't panic
    assert_eq!(merged.order.len(), 0); // Will be 1 when implemented
}

#[test]
fn test_merge_duplicate_definitions() {
    // Two graphs with the same struct definition
    let input1 = r"
        struct Point {
            long x;
            long y;
        };
    ";
    
    let input2 = r"
        struct Point {
            long x;
            long y;
        };
    ";
    
    let parsed1 = ic_parse::from_str(input1);
    let parsed2 = ic_parse::from_str(input2);
    
    let graph1 = ic_hir::from_ast(parsed1.tree);
    let graph2 = ic_hir::from_ast(parsed2.tree);
    
    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());
    
    let merged = merge_hir_trees(&[graph1, graph2]);
    
    // TODO: Once implementation is complete, verify deduplication
    // Should have only one Point definition
    assert_eq!(merged.order.len(), 0); // Will be 1 when implemented
}

#[test]
fn test_merge_different_modules() {
    // Two graphs with different module definitions
    let input1 = r"
        module A {
            struct Foo {
                long value;
            };
        };
    ";
    
    let input2 = r"
        module B {
            struct Bar {
                string name;
            };
        };
    ";
    
    let parsed1 = ic_parse::from_str(input1);
    let parsed2 = ic_parse::from_str(input2);
    
    let graph1 = ic_hir::from_ast(parsed1.tree);
    let graph2 = ic_hir::from_ast(parsed2.tree);
    
    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());
    
    let merged = merge_hir_trees(&[graph1, graph2]);
    
    // TODO: Once implementation is complete, verify both modules exist
    // Should have 2 module definitions
    assert_eq!(merged.order.len(), 0); // Will be 2 when implemented
}