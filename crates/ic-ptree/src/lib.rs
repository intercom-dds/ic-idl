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

use std::ffi::{CStr, CString, NulError};
use std::path::Path;

mod ffi;

#[must_use]
#[derive(Debug)]
pub struct ParseResult {
    inner: *mut ffi::parse_result,
}

impl ParseResult {
    #[must_use]
    pub fn warning_count(&self) -> usize {
        unsafe { ffi::ic_warning_count(self.inner) as usize }
    }

    #[must_use]
    pub fn error_count(&self) -> usize {
        unsafe { ffi::ic_error_count(self.inner) as usize }
    }

    #[must_use]
    pub fn diagnostics(&self) -> Option<String> {
        let c_str = unsafe { CStr::from_ptr(ffi::ic_parse_error(self.inner)) };
        let owned = c_str.to_str().map(ToString::to_string).ok()?;
        if owned.is_empty() { None } else { Some(owned) }
    }
}

impl Drop for ParseResult {
    fn drop(&mut self) {
        unsafe {
            ffi::ic_parse_free(self.inner);
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    Syntax(String),
    NulError(NulError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Syntax(v) => v.fmt(f),
            Error::NulError(v) => v.fmt(f),
        }
    }
}

/// Parses the given IDL. The parser assumes the input has already been
/// preprocessed; no preprocessor directives will be expanded or evaluated.
///
/// # Errors
///
/// This function may fail if the input IDL contains a nul byte, or if the
/// input contained invalid IDL.
pub fn parse_idl(input: &str) -> Result<ParseResult, Error> {
    let c_str = CString::new(input).map_err(Error::NulError)?;
    let inner = unsafe { ffi::ic_parse_idl(c_str.as_ptr()) };
    debug_assert!(!inner.is_null());

    let result = ParseResult { inner };
    if result.error_count() > 0 {
        let msg = result.diagnostics().unwrap_or_default();
        Err(Error::Syntax(msg))
    } else {
        Ok(result)
    }
}

/// Takes a set of individual parse trees and merges them into one. Once
/// merged, any duplicate types will be removed, and pointers throughout the
/// tree will be updated to point to the same types.
pub fn merge_trees(input: &[ParseResult]) -> ParseResult {
    let mut trees: Vec<_> = input.iter().map(|v| v.inner).collect();
    trees.push(std::ptr::null_mut());

    let inner = unsafe { ffi::ic_ptree_merge(trees.as_mut_ptr()) };
    debug_assert!(!inner.is_null());
    ParseResult { inner }
}

/// Dumps the ptree to `stdout` in a tree-like format.
pub fn ast_dump(result: &ParseResult) {
    unsafe {
        ffi::ic_ast_dump(result.inner);
    }
}

macro_rules! define_backend {
    ($fn_name:tt, $ffi_name:tt) => {
        pub fn $fn_name(result: &ParseResult, directory: &Path) {
            let dir = std::ffi::CString::new(directory.to_string_lossy().as_bytes()).unwrap();
            unsafe {
                ffi::$ffi_name(result.inner, dir.as_ptr());
            }
        }
    };
}

define_backend!(codegen_proto, ic_codegen_proto);
define_backend!(codegen_java, ic_codegen_java);
define_backend!(codegen_csharp, ic_codegen_csharp);
define_backend!(codegen_cpp, ic_codegen_cpp);
define_backend!(codegen_python, ic_codegen_python);
