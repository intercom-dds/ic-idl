use intercom_build::Codegen;

fn main() {
    let idl_files: Vec<_> = std::fs::read_dir("../corpus")
        .expect("corpus dir")
        .map(|res| res.map(|e| e.path()).expect("corpus IDL file path"))
        .collect();

    // Rebuild if corpus or ic-idl binary changes
    println!("cargo::rerun-if-changed=../corpus/");
    println!("cargo::rerun-if-changed=../../target/debug/ic-idl");

    Codegen::new("corpus")
        .executable("../../target/debug/ic-idl")
        .include("../corpus")
        .input(&idl_files)
        .generate()
        .expect("Generated corpus IDL");
}
