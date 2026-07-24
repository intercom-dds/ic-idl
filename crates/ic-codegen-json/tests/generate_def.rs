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

use std::path::PathBuf;

use ic_codegen_json::generate_def;
use ic_idl::{Compiler, CompilerOptions};
use intercom_cts::json::Value;

fn compile(input: &str) -> ic_idl::hir::ResolvedGraph {
    let mut compiler = Compiler::new(CompilerOptions::default());
    let name = "/virtual/generate-def-test.idl";
    _ = compiler.source_map_mut().embed_with_name(name, input);
    compiler.add_file(PathBuf::from(name));

    let (graph, diagnostics) = compiler.compile().expect("compilation failed");
    assert!(!diagnostics.has_errors(), "unexpected compile errors");
    graph
}

#[test]
fn generate_def_marks_root_and_includes_dependencies() {
    let graph = compile(
        r"
        module Farm {
            struct Animal { long id; };
            struct Pen { Animal occupant; };
        };
        ",
    );

    let root = graph
        .context
        .definitions
        .iter()
        .find_map(|(id, _)| (graph.context.qualified_name(id) == "Farm::Pen").then_some(id))
        .expect("Farm::Pen not found");

    let json = generate_def(&graph, root);
    let value: Value = intercom_cts::json::from_str(&json).expect("valid json");

    let Value::Object(top) = &value else {
        panic!("expected object at top level");
    };

    assert_eq!(
        top.get("root_type"),
        Some(&Value::String("Farm::Pen".to_string()))
    );

    let Some(Value::Object(definitions)) = top.get("definitions") else {
        panic!("expected definitions object");
    };
    let Some(Value::Object(farm)) = definitions.get("Farm") else {
        panic!("expected Farm module in definitions");
    };

    assert!(farm.contains_key("Pen"), "definitions missing Pen");
    assert!(
        farm.contains_key("Animal"),
        "definitions missing dependency Animal"
    );
}

fn definitions_of(json: &str) -> std::collections::BTreeMap<String, Value> {
    let value: Value = intercom_cts::json::from_str(json).expect("valid json");
    let Value::Object(top) = value else {
        panic!("expected object at top level");
    };
    let Some(Value::Object(definitions)) = top.get("definitions") else {
        panic!("expected definitions object");
    };
    definitions.clone()
}

#[test]
fn extensibility_is_hoisted_out_of_annotations() {
    let graph = compile(
        r"
        @final struct F { long a; };
        @mutable struct M { long a; };
        @appendable struct A { long a; };
        struct Plain { long a; };
        ",
    );

    let root = graph
        .context
        .definitions
        .iter()
        .find_map(|(id, _)| (graph.context.qualified_name(id) == "F").then_some(id))
        .expect("F not found");

    let defs = definitions_of(&generate_def(&graph, root));

    let extensibility = |name: &str| {
        let Some(Value::Object(def)) = defs.get(name) else {
            panic!("definition {name} missing");
        };
        assert!(
            !def.contains_key("annotations"),
            "{name} should not carry an annotations object"
        );
        def.get("extensibilityKind").cloned()
    };

    assert_eq!(extensibility("F"), Some(Value::String("final".to_string())));
    assert_eq!(
        extensibility("M"),
        Some(Value::String("mutable".to_string()))
    );
    assert_eq!(
        extensibility("A"),
        Some(Value::String("appendable".to_string()))
    );
    assert_eq!(extensibility("Plain"), None);
}

#[test]
fn base_type_is_a_type_name_string() {
    let graph = compile(
        r"
        struct Base { long a; };
        struct Derived : Base { long b; };
        ",
    );

    let root = graph
        .context
        .definitions
        .iter()
        .find_map(|(id, _)| (graph.context.qualified_name(id) == "Derived").then_some(id))
        .expect("Derived not found");

    let defs = definitions_of(&generate_def(&graph, root));

    let Some(Value::Object(derived)) = defs.get("Derived") else {
        panic!("Derived definition missing");
    };

    assert_eq!(
        derived.get("base_type"),
        Some(&Value::String("Base".to_string()))
    );
}
