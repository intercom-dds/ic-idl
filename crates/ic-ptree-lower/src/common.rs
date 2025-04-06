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

use std::ffi::{self, CString};

use ic_ptree::sys;
use ic_syntax::{ParamKind, Path};

const BUILTIN_ANNOTATIONS: &str = include_str!("../idl/annotations.idl");

#[allow(unused_unsafe)]
pub static mut NUM_UNDEF: *const sys::numeric = unsafe { std::ptr::addr_of!(sys::num_undef) };

#[must_use]
pub fn create_ident(name: &str) -> CString {
    CString::new(name).unwrap()
}

#[must_use]
pub fn path_str(path: &Path) -> String {
    let str = path
        .segments
        .iter()
        .map(|v| v.name.as_str())
        .collect::<Vec<_>>()
        .join("::");

    if path.leading_colons.is_some() {
        format!("::{str}")
    } else {
        str
    }
}

#[allow(clippy::cast_possible_wrap)]
pub fn param_kind(kind: ParamKind) -> ffi::c_int {
    let c = match kind {
        ParamKind::In => sys::OPT_IN,
        ParamKind::Out => sys::OPT_OUT,
        ParamKind::Inout => sys::OPT_INOUT,
    };
    c as ffi::c_int
}

type Appender = unsafe extern "C" fn(
    *mut sys::parser_state,
    *mut sys::ptree,
    *mut sys::ptree,
) -> *mut sys::ptree;

#[must_use]
pub unsafe fn collect_with<I, C, T>(
    state: *mut sys::parser_state,
    appender: Appender,
    iter: I,
    mut cb: C,
) -> *mut sys::ptree
where
    I: IntoIterator<Item = T>,
    C: FnMut(T) -> *mut sys::ptree,
{
    let mut list = std::ptr::null_mut();
    unsafe {
        for elem in iter {
            let node = cb(elem);
            list = appender(state, list, node);
        }
    }
    list
}

pub fn parse_builtin() -> ic_parse::ParseResult {
    let (builtin, errors) = ic_parse::from_str(BUILTIN_ANNOTATIONS);
    assert!(errors.is_empty(), "failed to parse built-in annotations: {errors:?}");
    builtin
}
