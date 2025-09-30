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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use ic_cli::Command;
use ic_emit::File;
use ic_emit::case::Case;
use ic_hir_xform::rename;

#[derive(Copy, Clone, Command, Debug, Default)]
pub struct RustOptions {
    /// Do not rename generated types
    #[option(long)]
    pub no_rename: bool,

    /// Annotate all types with `#[must_use]`
    #[option(long)]
    pub must_use: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
struct rust_options_t {
    pub no_rename: u8,
    pub must_use: u8,
}

unsafe extern "C" {
    fn ic_codegen_rust(
        result: *const ic_ptree::sys::parse_result,
        options: rust_options_t,
        list: *mut ic_ptree::sys::ic_list_t,
    );
}

#[must_use]
#[allow(clippy::undocumented_unsafe_blocks, clippy::needless_pass_by_value)]
pub fn codegen_rust(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: RustOptions,
) -> Vec<File> {
    // Clone HIR for Rust-specific transformations
    let hir = hir.clone();

    // Move nested types into modules. Keep track of the moved nodes to
    // properly escape their names later on to ensure the correct node gets
    // precedence.
    let (hir, moved_defs) = ic_hir_xform::move_nested::transform(hir);

    // Squash reopened modules into single definitions
    let hir = ic_hir_xform::squash_modules::transform(hir);

    // Strip prefixes from enumerators
    let hir = ic_hir_xform::enum_prefix::transform(hir);

    // Mark types with `IS_TRIVIAL` and `TOTAL_ORDER` flags
    let hir = ic_hir_xform::type_flags::transform(hir);

    // Rename `DDS::XTypes` to `DDS::xtypes`
    let hir = ic_hir_xform::rename_xtypes::transform(hir);

    // Rename all nodes to conform to Rust's naming convention
    let hir = ic_hir_xform::rename::transform(
        hir,
        &rename::Target {
            struct_type: Some(Case::Pascal),
            union_type: Some(Case::Pascal),
            enum_type: Some(Case::Pascal),
            interface: Some(Case::Pascal),
            valuetype: Some(Case::Pascal),
            alias: Some(Case::Pascal),
            bitmask: Some(Case::Pascal),
            bitset: Some(Case::Pascal),
            exception: Some(Case::Pascal),
            annotation: Some(Case::Pascal),
            member: Some(Case::Snake),
            variant: Some(Case::Pascal),
            enumerator: Some(Case::Pascal),
            bit_flag: Some(Case::Snake),
            bitset_field: Some(Case::Snake),
            constant: Some(Case::Snake),
            module: Some(Case::Snake),
            operation: Some(Case::Snake),
            attribute: Some(Case::Snake),
            parameter: Some(Case::Snake),
            annotation_param: Some(Case::Snake),
            name_preprocessor: Some(ic_hir_xform::rename::strip_common_suffixes),
            moved_defs,
        },
    );

    // Convert transformed HIR to ptree for C++ backend
    let result = ic_ptree_lower::from_hir(&hir, source_map);

    let ffi_options = rust_options_t {
        no_rename: u8::from(options.no_rename),
        must_use: u8::from(options.must_use),
    };

    let mut generated = vec![];
    unsafe {
        ic_codegen_rust(
            result.as_raw(),
            ffi_options,
            std::ptr::addr_of_mut!(generated).cast::<_>(),
        );
    }
    generated
}
