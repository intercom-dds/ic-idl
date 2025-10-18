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

use std::ffi::CString;

use ic_cli::Command;
use ic_emit::File;

#[derive(Command, Debug, Default, Clone)]
pub struct PythonOptions {
    /// Rename all types to conform to PEP-8
    #[option(long)]
    pub use_pep8: bool,

    /// Postfix to use for global modules
    #[option(long)]
    pub global_postfix: Option<String>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
struct python_options_t {
    pub use_pep8: u8,
    pub global_postfix: *const ::std::os::raw::c_char,
}

unsafe extern "C" {
    fn ic_codegen_python(
        result: *const ic_ptree::sys::parse_result,
        options: python_options_t,
        list: *mut ic_ptree::sys::ic_list_t,
    );
}

/// # Panics
///
/// May panic if some of the passed string parameters contain a NUL byte.
#[must_use]
#[allow(clippy::undocumented_unsafe_blocks, clippy::needless_pass_by_value)]
pub fn codegen_python(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: PythonOptions,
) -> Vec<File> {
    let result = ic_ptree_lower::from_hir(hir, source_map);
    let global_postfix = options
        .global_postfix
        .as_ref()
        .map(|s| CString::new(s.as_str()).expect("Invalid global_postfix"));

    let ffi_options = python_options_t {
        use_pep8: u8::from(options.use_pep8),
        global_postfix: global_postfix
            .as_ref()
            .map_or(std::ptr::null(), |s| s.as_ptr()),
    };

    let mut generated = vec![];
    unsafe {
        ic_codegen_python(
            result.as_raw(),
            ffi_options,
            std::ptr::addr_of_mut!(generated).cast::<_>(),
        );
    }
    generated
}
