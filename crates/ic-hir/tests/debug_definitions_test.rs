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

#[test]
fn debug_forward_declaration_collection() {
    let input = r#"
        struct Foo;
        struct Foo {
            long x;
        };
    "#;

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");

    let result = ic_hir::from_ast(parsed.tree);

    // Print all definitions
    println!("All definitions:");
    for (id, def) in result.context.definitions.iter() {
        println!(
            "  {:?}: {} - {:?} (parent: {:?})",
            id, def.ident.name, def.kind, def.parent
        );
    }

    // Print all definitions named "Foo"
    println!("\nDefinitions named 'Foo':");
    let mut foo_defs = Vec::new();
    for (id, def) in result.context.definitions.iter() {
        if def.ident.name == "Foo" {
            println!("  {:?}: {:?} (parent: {:?})", id, def.kind, def.parent);
            foo_defs.push(id);
        }
    }

    // Check the order they would be processed
    println!("\nOrder of processing:");
    for &id in &foo_defs {
        let def = &result.context.definitions[id];
        println!("  Processing {:?}: {:?}", id, def.kind);
    }
}
