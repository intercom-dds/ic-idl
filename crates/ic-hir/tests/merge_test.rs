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

use ic_hir::merge::merge_hir_trees;
use ic_hir::hir::DefKind;

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
    
    // Should have one definition (Point struct)
    assert_eq!(merged.order.len(), 1);
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
    
    // With deduplication, we should have only 1 Point definition
    assert_eq!(merged.order.len(), 1);
    
    // Verify it's the Point struct
    let def = merged.context.definitions.get(merged.order[0]);
    assert_eq!(def.ident.name, "Point");
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
    
    // Should have 2 module definitions (A and B)
    assert_eq!(merged.order.len(), 2);
    
    // Verify we can access the definitions
    let mut module_names = Vec::new();
    for &def_id in &merged.order {
        let def = merged.context.definitions.get(def_id);
        if let DefKind::Module(_) = &def.kind {
            module_names.push(def.ident.name.clone());
        }
    }
    module_names.sort();
    assert_eq!(module_names, vec!["A", "B"]);
}

#[test]
fn test_merge_with_references() {
    // Test that references between types are properly updated
    let input = r"
        struct Point {
            long x;
            long y;
        };
        
        struct Line {
            Point start;
            Point end;
        };
    ";
    
    let parsed = ic_parse::from_str(input);
    assert!(parsed.errors.is_empty());
    
    let graph = ic_hir::from_ast(parsed.tree);
    assert!(graph.errors.is_empty());
    
    let merged = merge_hir_trees(&[graph]);
    
    // Should have 2 definitions (Point and Line)
    assert_eq!(merged.order.len(), 2);
    
    // TODO: Verify that Line's references to Point are correctly updated
}