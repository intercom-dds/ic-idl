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

use std::ffi::{CString, NulError};
use std::path::Path;

mod ffi;

pub struct ParseResult {
    inner: *mut ffi::parse_result,
}

impl Drop for ParseResult {
    fn drop(&mut self) {
        unsafe {
            ffi::ic_parse_free(self.inner);
        }
    }
}

pub fn parse_idl(input: &str) -> Result<ParseResult, NulError> {
    let c_str = CString::new(input)?;
    let inner = unsafe { ffi::ic_parse_idl(c_str.as_ptr()) };
    debug_assert!(!inner.is_null());

    Ok(ParseResult { inner })
}

pub fn merge_trees(input: &[ParseResult]) -> ParseResult {
    let mut trees: Vec<_> = input.iter().map(|v| v.inner).collect();
    trees.push(std::ptr::null_mut());

    let inner = unsafe { ffi::ic_ptree_merge(trees.as_mut_ptr()) };
    debug_assert!(!inner.is_null());
    ParseResult { inner }
}

pub fn ast_dump(result: &ParseResult) {
    unsafe {
        ffi::ic_ast_dump(result.inner);
    }
}

macro_rules! c_str {
    ($var: tt) => {{
        std::ffi::CString::new($var.to_string_lossy().as_bytes()).unwrap()
    }};
}

pub fn codegen_proto(result: &ParseResult, directory: &Path) {
    unsafe {
        let path = c_str!(directory);
        ffi::ic_codegen_proto(result.inner, path.as_ptr());
    }
}

pub fn codegen_java(result: &ParseResult, directory: &Path) {
    unsafe {
        let path = c_str!(directory);
        ffi::ic_codegen_java(result.inner, path.as_ptr());
    }
}
