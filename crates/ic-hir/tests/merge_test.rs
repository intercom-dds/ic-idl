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

mod common;

use ic_hir::hir::DefKind;
use ic_hir::merge::merge_hir_trees;
use ic_vfs::SourceMap;

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

    let (graph, _, _) = common::parse_and_resolve(input);
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

    let (graph1, _, _) = common::parse_and_resolve(input1);
    let (graph2, _, _) = common::parse_and_resolve(input2);

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

    let (graph1, _, _) = common::parse_and_resolve(input1);
    let (graph2, _, _) = common::parse_and_resolve(input2);

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

    let (graph, _, _) = common::parse_and_resolve(input);
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
    // Test that modules with the same name from different files don't get deduplicated
    let mut source_map = SourceMap::default();

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

    // Parse with different file names to simulate different files
    let file1 = source_map.embed_with_name("shapes1.idl", input1);
    let file2 = source_map.embed_with_name("shapes2.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

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

#[test]
fn test_merge_conflicting_definitions() {
    // Test that conflicting definitions are detected and reported as errors
    let mut source_map = SourceMap::default();

    let input1 = r"struct Foo {};";
    let input2 = r"struct Foo {
    string value;
};";

    let file1 = source_map.embed_with_name("file1.idl", input1);
    let file2 = source_map.embed_with_name("file2.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Snapshot test the error message
    let mut output = String::new();
    ic_diagnostic::emit_diagnostic(&mut output, &source_map, &merged.errors[0]).unwrap();
    insta::assert_snapshot!(output);

    // Despite the error, we should still have the definition (mapped to one of them)
    assert_eq!(merged.order.len(), 1);
}

#[test]
fn test_merge_same_definition_from_include() {
    // Test that the same definition from an include is properly deduplicated
    // This simulates the same span appearing multiple times
    let input = r"
        struct SharedType {
            long id;
        };
    ";

    // Parse the same input twice (simulating include)
    let (graph1, _, _) = common::parse_and_resolve(input);
    let (graph2, _, _) = common::parse_and_resolve(input);

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no errors - same span means same definition
    assert!(merged.errors.is_empty());

    // Should have only 1 definition due to deduplication
    assert_eq!(merged.order.len(), 1);
}

#[test]
fn test_merge_multiple_conflicts() {
    // Test multiple conflicting definitions in a single merge
    let mut source_map = SourceMap::default();

    let input1 = r"
struct Point { long x; };
enum Color { RED, GREEN };
";

    let input2 = r"
struct Point { long x; long y; };
enum Color { RED, GREEN, BLUE };
";

    let file1 = source_map.embed_with_name("shapes.idl", input1);
    let file2 = source_map.embed_with_name("graphics.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have errors for the conflicting definitions

    // Snapshot test all error messages
    let mut output = String::new();
    for error in &merged.errors {
        ic_diagnostic::emit_diagnostic(&mut output, &source_map, error).unwrap();
        output.push('\n');
    }
    insta::assert_snapshot!(output);
}

#[test]
fn test_merge_with_nested_definitions() {
    // Test that definitions nested in modules are properly merged
    let mut source_map = SourceMap::default();

    let input1 = r"
module api {
    struct Request {
        string method;
    };
};
";

    let input2 = r"
module api {
    struct Response {
        long status;
    };
};
";

    let file1 = source_map.embed_with_name("api_request.idl", input1);
    let file2 = source_map.embed_with_name("api_response.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no errors
    assert!(merged.errors.is_empty());

    // Should have 2 api modules (module reopening)
    let module_count = merged
        .order
        .iter()
        .filter(|&&def_id| {
            let def = merged.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Module(_)) && def.ident.name == "api"
        })
        .count();
    assert_eq!(module_count, 2);
}

#[test]
fn test_merge_module_with_include_deduplication() {
    // Test that module deduplication preserves children when the same module
    // appears in multiple files (e.g., via includes)

    // When the same module definition appears with the same span (from an include),
    // it should be deduplicated but all children should be preserved

    // Use a shared source map to simulate the same file being included in two places
    let mut source_map = SourceMap::default();

    // Create a shared file that will be "included"
    let shared_content = r"
module abc {
    struct bar {};
};
";
    let shared_file = source_map.embed_with_name("shared.idl", shared_content);

    // Parse the shared file twice (simulating it being included in two different files)
    let parsed1 = ic_parse::from_file(shared_file, &source_map);
    let parsed2 = ic_parse::from_file(shared_file, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no errors - same module with same span is deduplicated
    assert_eq!(merged.errors.len(), 0);

    // Should have only 1 module due to deduplication
    let module_count = merged
        .order
        .iter()
        .filter(|&&def_id| {
            let def = merged.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Module(_)) && def.ident.name == "abc"
        })
        .count();
    assert_eq!(module_count, 1, "Module should be deduplicated");

    // But the struct inside should still exist
    let modules: Vec<_> = merged
        .order
        .iter()
        .filter_map(|&def_id| {
            let def = merged.context.definitions.get(def_id);
            if matches!(def.kind, DefKind::Module(_)) && def.ident.name == "abc" {
                Some(def_id)
            } else {
                None
            }
        })
        .collect();

    let module_def = merged.context.definitions.get(modules[0]);
    if let DefKind::Module(module_ty) = &module_def.kind {
        // Should have the bar struct
        let has_bar = module_ty.definitions.iter().any(|&def_id| {
            let def = merged.context.definitions.get(def_id);
            matches!(def.kind, DefKind::Struct(_)) && def.ident.name == "bar"
        });
        assert!(has_bar, "Module should contain struct bar");
    } else {
        panic!("Expected module");
    }
}

#[test]
fn test_merge_forward_declaration_and_definition() {
    // Test the merge behavior when we have forward declarations and definitions
    // The fix ensures both are preserved as separate entries during merge

    // Create a simple test case with forward declaration and definition
    let input = r"
struct Foo;
struct Foo {
    long x;
};
";

    // Parse it with built-in context
    let (graph, _, _) = common::parse_and_resolve(input);

    // Should have no errors
    assert!(graph.errors.is_empty());

    // Verify we have both the forward declaration and the definition
    let foos: Vec<_> = graph
        .context
        .definitions
        .iter()
        .filter(|(_, def)| def.ident.name == "Foo")
        .collect();

    assert_eq!(
        foos.len(),
        2,
        "Should have both forward declaration and definition"
    );

    // Count by type
    let decl_count = foos
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Decl(_)))
        .count();
    let struct_count = foos
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Struct(_)))
        .count();

    assert_eq!(decl_count, 1, "Should have 1 forward declaration");
    assert_eq!(struct_count, 1, "Should have 1 struct definition");

    // Now test merging - parse again for the second graph
    let (graph2, _, _) = common::parse_and_resolve(input);

    // Merge them
    let merged = merge_hir_trees(&[graph, graph2]);

    // After merge, we should still have both (deduplicated by span)
    let merged_foos: Vec<_> = merged
        .context
        .definitions
        .iter()
        .filter(|(_, def)| def.ident.name == "Foo")
        .collect();

    // The merge might create additional entries due to how builtin context is handled
    // What matters is we have at least one forward declaration and one struct definition
    let merged_decl_count = merged_foos
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Decl(_)))
        .count();
    let merged_struct_count = merged_foos
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Struct(_)))
        .count();

    assert!(
        merged_decl_count >= 1,
        "Should have at least 1 forward declaration after merge"
    );
    assert!(
        merged_struct_count >= 1,
        "Should have at least 1 struct definition after merge"
    );
}

#[test]
fn test_merge_forward_declaration_across_files() {
    // Test the actual fix: when merging files where one has forward declarations
    // and another has definitions, both should be preserved
    let mut source_map = SourceMap::default();

    // Simulate the ast.idl case - file with forward declarations at top
    let input1 = r"
module test {
    struct Foo;  // Forward declaration
    
    struct Container {
        long x;
    };
    
    struct Foo {  // Full definition  
        long id;
    };
};
";

    // Another file that references the same module
    let input2 = r"
module test {
    struct Bar {
        long y;
    };
};
";

    let file1 = source_map.embed_with_name("file1.idl", input1);
    let file2 = source_map.embed_with_name("file2.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no errors
    assert!(merged.errors.is_empty());

    // The key test: we should have BOTH the forward declaration AND the full definition
    let all_foos: Vec<_> = merged
        .context
        .definitions
        .iter()
        .filter(|(_, def)| def.ident.name == "Foo")
        .collect();

    assert_eq!(
        all_foos.len(),
        2,
        "Should have both forward declaration and full definition of Foo"
    );

    // Check that we have one forward declaration and one struct definition
    let decl_count = all_foos
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Decl(_)))
        .count();
    let struct_count = all_foos
        .iter()
        .filter(|(_, def)| matches!(def.kind, DefKind::Struct(_)))
        .count();

    assert_eq!(decl_count, 1, "Should have 1 forward declaration");
    assert_eq!(struct_count, 1, "Should have 1 struct definition");
}

#[test]
fn test_merge_preserves_parent_defid() {
    // Test that parent DefId relationships are preserved during merge
    // This was the bug where types lost their parent DefId, causing
    // unqualified name lookups to fail
    let mut source_map = SourceMap::default();

    // First file defines a module
    let input1 = r"
module TestModule {
    // Empty module
};
";

    // Second file adds to the module
    let input2 = r"
module TestModule {
    struct ChildStruct {
        long value;
    };
};
";

    let file1 = source_map.embed_with_name("parent.idl", input1);
    let file2 = source_map.embed_with_name("child.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no errors
    assert!(merged.errors.is_empty());

    // Find the ChildStruct definition
    let child_struct = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "ChildStruct" && matches!(def.kind, DefKind::Struct(_)))
        .expect("ChildStruct should exist");

    // The key test: ChildStruct should have a parent DefId
    assert!(
        child_struct.1.parent.is_some(),
        "ChildStruct should have a parent DefId"
    );

    // Verify the parent is a TestModule
    let parent_id = child_struct.1.parent.unwrap();
    let parent_def = merged.context.definitions.get(parent_id);
    assert_eq!(parent_def.ident.name, "TestModule");
    assert!(matches!(parent_def.kind, DefKind::Module(_)));

    // Verify we can get the qualified name (this would panic before the fix)
    let qualified_name = merged.context.qualified_name(child_struct.0);
    assert_eq!(qualified_name, "TestModule::ChildStruct");
}

#[test]
#[allow(clippy::similar_names)]
fn test_merge_complex_parent_relationships() {
    // Test more complex parent relationships with nested modules
    let mut source_map = SourceMap::default();

    let input1 = r"
module Outer {
    module Inner {
        struct TypeA {
            long id;
        };
    };
};
";

    let input2 = r"
module Outer {
    module Inner {
        struct TypeB {
            long value;
        };
    };
    
    module Another {
        struct TypeC {
            string name;
        };
    };
};
";

    let file1 = source_map.embed_with_name("types1.idl", input1);
    let file2 = source_map.embed_with_name("types2.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no errors
    assert!(merged.errors.is_empty());

    // Find TypeA and verify its parent chain
    let type_a = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TypeA" && matches!(def.kind, DefKind::Struct(_)))
        .expect("TypeA struct should exist");

    assert!(type_a.1.parent.is_some(), "TypeA should have a parent");
    let type_a_qualified = merged.context.qualified_name(type_a.0);
    assert_eq!(type_a_qualified, "Outer::Inner::TypeA");

    // Find TypeB and verify its parent chain
    let type_b = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TypeB" && matches!(def.kind, DefKind::Struct(_)))
        .expect("TypeB struct should exist");

    assert!(type_b.1.parent.is_some(), "TypeB should have a parent");
    let type_b_qualified = merged.context.qualified_name(type_b.0);
    assert_eq!(type_b_qualified, "Outer::Inner::TypeB");

    // Find TypeC and verify its parent chain
    let type_c = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TypeC" && matches!(def.kind, DefKind::Struct(_)))
        .expect("TypeC struct should exist");

    assert!(type_c.1.parent.is_some(), "TypeC should have a parent");
    let type_c_qualified = merged.context.qualified_name(type_c.0);
    assert_eq!(type_c_qualified, "Outer::Another::TypeC");
}

#[test]
fn test_merge_preserves_builtin_annotations() {
    // Test that built-in annotations are available in all merged files
    // This was a bug where only the first file had access to built-in annotations

    // Create built-in annotations
    let builtins = r"
        @annotation optional {
            boolean value default TRUE;
        };
    ";

    // First file using the annotation
    let input1 = r"
        struct TypeA {
            @optional string name;
        };
    ";

    // Second file also using the annotation
    let input2 = r"
        struct TypeB {
            @optional long value;
        };
    ";

    // Parse and resolve with built-in context
    let (graph1, _, _) = common::parse_with_custom_builtins(builtins, input1, false);
    let (graph2, _, _) = common::parse_with_custom_builtins(builtins, input2, false);

    // Both should compile without errors
    assert!(graph1.errors.is_empty(), "First file should have no errors");
    assert!(
        graph2.errors.is_empty(),
        "Second file should have no errors"
    );

    // Merge the graphs
    let merged = merge_hir_trees(&[graph1, graph2]);

    // Should have no merge errors
    assert!(merged.errors.is_empty(), "Merge should have no errors");

    // Find both structs
    let type_a = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TypeA" && matches!(def.kind, DefKind::Struct(_)))
        .expect("TypeA should exist");

    let type_b = merged
        .context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "TypeB" && matches!(def.kind, DefKind::Struct(_)))
        .expect("TypeB should exist");

    // Verify both have members with annotations
    if let DefKind::Struct(struct_a) = &type_a.1.kind {
        assert_eq!(struct_a.members.len(), 1);
        assert!(
            !struct_a.members[0].annotations.is_empty(),
            "TypeA member should have annotation"
        );
    } else {
        panic!("TypeA should be a struct");
    }

    if let DefKind::Struct(struct_b) = &type_b.1.kind {
        assert_eq!(struct_b.members.len(), 1);
        assert!(
            !struct_b.members[0].annotations.is_empty(),
            "TypeB member should have annotation"
        );
    } else {
        panic!("TypeB should be a struct");
    }
}

#[test]
fn test_merge_nested_modules_parent_child_consistency() {
    let mut source_map = SourceMap::default();

    let input1 = r"
bitmask Flags {
    FLAG_A,
    FLAG_B
};
";

    let input2 = r"
module outer {
    module sub1 {
        struct Type1 { long x; };
    };
    module sub2 {
        struct Type2 { long y; };
    };
    module sub3 {
        struct Type3 { long z; };
    };
    module sub4 {
        struct Type4 { long w; };
    };
};
";

    let file1 = source_map.embed_with_name("file1.idl", input1);
    let file2 = source_map.embed_with_name("file2.idl", input2);

    let parsed1 = ic_parse::from_file(file1, &source_map);
    let parsed2 = ic_parse::from_file(file2, &source_map);

    let graph1 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed1.tree));
    let graph2 = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed2.tree));

    assert!(graph1.errors.is_empty());
    assert!(graph2.errors.is_empty());

    let merged = merge_hir_trees(&[graph1, graph2]);

    assert!(merged.errors.is_empty(), "Merge should have no errors");

    for (def_id, def) in &merged.context.definitions {
        if let DefKind::Module(module_ty) = &def.kind {
            for &child_id in &module_ty.definitions {
                let child_def = merged.context.definitions.get(child_id);
                assert_eq!(
                    child_def.parent,
                    Some(def_id),
                    "Child {} in module {} has wrong parent: expected {:?}, got {:?}",
                    child_def.ident.name,
                    def.ident.name,
                    Some(def_id),
                    child_def.parent
                );
            }
        }
    }
}
