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

use std::ffi::CString;

use ic_cli::Command;
use ic_emit::File;

#[derive(Command, Debug, Default, Clone)]
pub struct CppOptions {
    /// Generate scoped enums
    #[option(long)]
    pub scoped_enums: bool,

    /// Use access functions instead of direct member access
    #[option(long)]
    pub access_functions: bool,

    /// Do not generate ostream operators for serialization
    #[option(long)]
    pub no_stream_op: bool,

    /// Generate formatting specializations for fmtlib
    #[option(long)]
    pub use_fmt: bool,

    /// Use <sym> as dllexport symbol
    #[option(long, arg = "sym")]
    pub dll_export: Option<String>,

    /// Use <ext> as file extension for C++ headers
    #[option(long, arg = "ext")]
    pub header_ext: Option<String>,

    /// Store header files inside a subfolder
    #[option(long, arg = "dir")]
    pub header_subfolder: Option<String>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
struct cpp_options_t {
    pub header_postfix: *const ::std::os::raw::c_char,
    pub header_subfolder: *const ::std::os::raw::c_char,
    pub header_ext: *const ::std::os::raw::c_char,
    pub dll_export: *const ::std::os::raw::c_char,
    pub scoped_enums: u8,
    pub access_functions: u8,
    pub no_stream_op: u8,
    pub use_fmt: u8,
}

unsafe extern "C" {
    fn ic_codegen_cpp(
        result: *const ic_ptree::sys::parse_result,
        options: cpp_options_t,
        list: *mut ic_ptree::sys::ic_list_t,
    );
}

/// # Panics
///
/// May panic if some of the passed string parameters contain a NUL byte.
#[must_use]
#[allow(clippy::undocumented_unsafe_blocks, clippy::needless_pass_by_value)]
pub fn codegen_cpp(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: CppOptions,
) -> Vec<File> {
    let result = ic_ptree_lower::from_hir(hir, source_map);
    let header_subfolder = options
        .header_subfolder
        .as_ref()
        .map(|s| CString::new(s.as_str()).expect("Invalid header_subfolder"));

    let header_ext = options
        .header_ext
        .as_ref()
        .map(|s| CString::new(s.as_str()).expect("Invalid header_ext"));

    let dll_export = options
        .dll_export
        .as_ref()
        .map(|s| CString::new(s.as_str()).expect("Invalid dll_export"));

    let ffi_options = cpp_options_t {
        header_postfix: std::ptr::null(),
        header_subfolder: header_subfolder
            .as_ref()
            .map_or(std::ptr::null(), |s| s.as_ptr()),
        header_ext: header_ext.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
        dll_export: dll_export.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
        scoped_enums: u8::from(options.scoped_enums),
        access_functions: u8::from(options.access_functions),
        no_stream_op: u8::from(options.no_stream_op),
        use_fmt: u8::from(options.use_fmt),
    };

    let mut generated = vec![];
    unsafe {
        ic_codegen_cpp(
            result.as_raw(),
            ffi_options,
            std::ptr::addr_of_mut!(generated).cast::<_>(),
        );
    }
    generated
}
