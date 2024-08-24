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

use std::ffi::{CStr, NulError};
use std::path::Path;

use ic_syntax::Item;

mod lower;
mod sys;

#[must_use]
#[derive(Debug)]
pub struct ParseResult {
    inner: *mut sys::ic_parse_result_t,
}

impl ParseResult {
    #[must_use]
    pub fn error_count(&self) -> usize {
        unsafe { sys::ic_error_count(self.inner) as usize }
    }

    #[must_use]
    pub fn diagnostics(&self) -> Option<String> {
        let c_str = unsafe { CStr::from_ptr(sys::ic_parse_error(self.inner)) };
        let owned = c_str.to_str().map(ToString::to_string).ok()?;
        if owned.is_empty() { None } else { Some(owned) }
    }
}

impl Drop for ParseResult {
    fn drop(&mut self) {
        unsafe {
            sys::ic_parse_free(self.inner);
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

/// Takes a set of individual parse trees and merges them into one. Once
/// merged, any duplicate types will be removed, and pointers throughout the
/// tree will be updated to point to the same types.
pub fn merge_trees(input: &[ParseResult]) -> ParseResult {
    let mut trees: Vec<_> = input.iter().map(|v| v.inner.cast_const()).collect();
    trees.push(std::ptr::null_mut());

    let inner = unsafe { sys::ic_ptree_merge(trees.as_mut_ptr()) };
    debug_assert!(!inner.is_null());
    ParseResult { inner }
}

#[allow(clippy::ptr_as_ptr)]
pub fn lower_ast(mut ast: &[Item]) -> ParseResult {
    let ptr = std::ptr::addr_of_mut!(ast) as _;
    let inner = unsafe { sys::ic_parse_w_state(Some(lower::callback), ptr) };
    ParseResult { inner }
}

/// Dumps the ptree to `stdout` in a tree-like format.
pub fn ptree_dump(result: &ParseResult) {
    unsafe {
        sys::ic_ptree_dump(result.inner);
    }
}

macro_rules! define_backend {
    ($fn_name:tt, $ffi_name:tt) => {
        #[must_use]
        pub fn $fn_name(result: &ParseResult, directory: &Path) -> Vec<String> {
            let dir = std::ffi::CString::new(directory.to_string_lossy().as_bytes()).unwrap();
            unsafe {
                sys::$ffi_name(result.inner, dir.as_ptr());
            }
            vec![]
        }
    };
}

define_backend!(codegen_cpp, ic_codegen_cpp);
define_backend!(codegen_idl, ic_codegen_idl);
define_backend!(codegen_json, ic_codegen_json);
define_backend!(codegen_proto, ic_codegen_proto);
define_backend!(codegen_python, ic_codegen_python);
define_backend!(codegen_rust, ic_codegen_cpp);
