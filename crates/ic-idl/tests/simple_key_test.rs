use ic_idl::ast_to_hir;
use ic_lint::LintConfig;
use ic_parse::SourceMap;

#[test]
fn test_simple_key_annotation() {
    let input = r#"
        struct S {
            @key string value;
        };
    "#;

    let mut source_map = SourceMap::default();
    let file_id = source_map.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, Default::default(), &mut source_map);

    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let hir_result = ast_to_hir(parsed.tree, &source_map, &LintConfig::default());

    match hir_result {
        Ok(_) => println!("SUCCESS: HIR conversion succeeded"),
        Err(e) => panic!("HIR conversion failed: {}", e),
    }
}
