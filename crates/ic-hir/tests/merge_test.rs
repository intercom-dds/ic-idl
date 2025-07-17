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

use ic_hir::hir::DefKind;
use ic_hir::merge::merge_hir_trees;

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

    // Find Point and Line definitions
    let mut point_id = None;
    let mut line_id = None;

    for &def_id in &merged.order {
        let def = merged.context.definitions.get(def_id);
        match def.ident.name.as_str() {
            "Point" => point_id = Some(def_id),
            "Line" => line_id = Some(def_id),
            _ => {}
        }
    }

    assert!(point_id.is_some(), "Point definition not found");
    assert!(line_id.is_some(), "Line definition not found");

    // Verify that Line's members reference the correct Point
    let line_def = merged.context.definitions.get(line_id.unwrap());
    if let DefKind::Struct(line_struct) = &line_def.kind {
        assert_eq!(line_struct.members.len(), 2);

        // Both start and end should reference Point
        for member in &line_struct.members {
            if let ic_hir::hir::TyKind::Adt(ref_id) = &member.ty.kind {
                assert_eq!(
                    *ref_id,
                    point_id.unwrap(),
                    "Member {} should reference Point",
                    member.ident.name
                );
            } else {
                panic!("Line member {} should be an ADT type", member.ident.name);
            }
        }
    } else {
        panic!("Line should be a struct");
    }
}

#[test]
fn test_merge_module_reopening() {
    // Test that modules with the same name from different files get merged
    let input1 = r"
        module Shapes {
            struct Circle {
                float radius;
            };
        };
    ";

    let input2 = r"
        module Shapes {
            struct Rectangle {
                float width;
                float height;
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

    // Should have 2 Shapes modules (module reopening creates separate modules)
    let module_count = merged
        .order
        .iter()
        .filter(|&&def_id| {
            let def = merged.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Module(_)) && def.ident.name == "Shapes"
        })
        .count();
    assert_eq!(module_count, 2, "Should have exactly two Shapes modules");

    // Find both Shapes modules
    let shapes_modules: Vec<_> = merged
        .order
        .iter()
        .filter_map(|&def_id| {
            let def = merged.context.definitions.get(def_id);
            if matches!(def.kind, DefKind::Module(_)) && def.ident.name == "Shapes" {
                Some((def_id, def))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        shapes_modules.len(),
        2,
        "Should find exactly 2 Shapes modules"
    );

    // Check that each module contains its respective struct
    let mut found_circle = false;
    let mut found_rectangle = false;

    for (_, module_def) in &shapes_modules {
        if let DefKind::Module(module_ty) = &module_def.kind {
            for &def_id in &module_ty.definitions {
                let def = merged.context.definitions.get(def_id);
                match def.ident.name.as_str() {
                    "Circle" => found_circle = true,
                    "Rectangle" => found_rectangle = true,
                    _ => {}
                }
            }
        }
    }

    assert!(
        found_circle,
        "Circle struct should be in one of the Shapes modules"
    );
    assert!(
        found_rectangle,
        "Rectangle struct should be in one of the Shapes modules"
    );

    // Verify that structs with same name within modules are deduplicated
    // Count all Circle structs across all modules
    let mut all_circle_ids = std::collections::HashSet::new();
    for (_, module_def) in &shapes_modules {
        if let DefKind::Module(module_ty) = &module_def.kind {
            for &def_id in &module_ty.definitions {
                let def = merged.context.definitions.get(def_id);
                if matches!(def.kind, DefKind::Struct(_)) && def.ident.name == "Circle" {
                    all_circle_ids.insert(def_id);
                }
            }
        }
    }
    assert_eq!(
        all_circle_ids.len(),
        1,
        "Circle struct should be deduplicated across modules"
    );
}
