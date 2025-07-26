// Copyright 2025 KONGSBERG
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

// Test that annotation default values are properly lowered through HIR to ptree

use std::fs;

use ic_idl::{Compiler, CompilerOptions};

#[test]
fn test_annotation_defaults_in_generated_idl() {
    let test_content = r#"
        @annotation sample {
            boolean enabled default TRUE;
            string description default "Default text";
            long priority default 42;
        };

        @sample(enabled = FALSE)
        struct TestStruct {
            long value;
        };
    "#;

    // Create a temporary directory
    let temp_dir = format!("/tmp/ic_idl_test_{}", std::process::id());
    fs::create_dir_all(&temp_dir).unwrap();

    let input_file = format!("{temp_dir}/test.idl");
    fs::write(&input_file, test_content).unwrap();

    // Set up compiler options
    let mut options = CompilerOptions::default();
    options.files.push(input_file.into());
    options.codegen.idl_out = Some(temp_dir.clone().into());

    // Compile
    let mut compiler = Compiler::new(options);
    let result = compiler.compile();

    // Check compilation succeeded
    match &result {
        Ok(compilation_result) => {
            assert!(
                compilation_result.diagnostics.errors.is_empty(),
                "Compilation had errors"
            );
        }
        Err(e) => panic!("Compilation failed: {e:?}"),
    }

    // Now generate IDL output
    let compilation_output = result.unwrap();
    let hir = ic_idl::ast_to_hir(
        compilation_output.items,
        compiler.source_map(),
        &compiler.options().warn.to_lint_config(),
    )
    .unwrap();

    let ptree = ic_idl::hir_to_ptree(&hir, compiler.source_map());
    let files = ic_codegen_idl::codegen_idl(&ptree);

    // Find the generated content
    assert!(!files.is_empty(), "No files generated");
    let ic_emit::File::Generated {
        source: generated, ..
    } = &files[0]
    else {
        panic!("Expected generated file")
    };

    // Check that the annotation definition includes default values
    assert!(generated.contains("@annotation sample"));
    assert!(generated.contains("boolean enabled default true")); // IDL uses lowercase
    assert!(generated.contains("string  description default \"Default text\""));
    assert!(generated.contains("int32   priority default 42")); // long becomes int32

    // Clean up
    fs::remove_dir_all(&temp_dir).ok();
}
