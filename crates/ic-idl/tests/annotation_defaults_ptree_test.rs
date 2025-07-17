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

    let input_file = format!("{}/test.idl", temp_dir);
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
        Ok(compiled) => {
            assert!(
                compiled.diagnostics.errors.is_empty(),
                "Compilation had errors"
            );
        }
        Err(e) => panic!("Compilation failed: {:?}", e),
    }

    // Now generate IDL output
    let compiled = result.unwrap();
    let hir = ic_idl::ast_to_hir(
        compiled.items,
        compiler.source_map(),
        &compiler.options().warn.to_lint_config(),
    )
    .unwrap();

    let ptree = ic_idl::hir_to_ptree(&hir, compiler.source_map());
    let files = ic_codegen_idl::codegen_idl(&ptree);

    // Find the generated content
    assert!(!files.is_empty(), "No files generated");
    let generated = match &files[0] {
        ic_emit::File::Generated { source, .. } => source,
        _ => panic!("Expected generated file"),
    };

    // Check that the annotation definition includes default values
    assert!(generated.contains("@annotation sample"));
    assert!(generated.contains("boolean enabled default true")); // IDL uses lowercase
    assert!(generated.contains("string  description default \"Default text\""));
    assert!(generated.contains("int32   priority default 42")); // long becomes int32

    // Clean up
    fs::remove_dir_all(&temp_dir).ok();
}
