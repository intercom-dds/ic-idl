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

#[path = "../src/common.rs"]
mod common;

use ic_syntax::{Ident, ParamKind, Path};
use ic_vfs::Span;

#[test]
fn test_create_ident() {
    let ident = common::create_ident("test_name");
    assert_eq!(ident.to_str().unwrap(), "test_name");

    let empty = common::create_ident("");
    assert_eq!(empty.to_str().unwrap(), "");

    let special = common::create_ident("name_with_$pecial");
    assert_eq!(special.to_str().unwrap(), "name_with_$pecial");
}

#[test]
fn test_path_str() {
    let span = Span::default();

    // Simple path
    let path = Path {
        leading_colons: None,
        segments: vec![Ident {
            name: "foo".to_string(),
            span,
        }],
    };
    assert_eq!(common::path_str(&path), "foo");

    // Path with multiple segments
    let path = Path {
        leading_colons: None,
        segments: vec![
            Ident {
                name: "foo".to_string(),
                span,
            },
            Ident {
                name: "bar".to_string(),
                span,
            },
            Ident {
                name: "baz".to_string(),
                span,
            },
        ],
    };
    assert_eq!(common::path_str(&path), "foo::bar::baz");

    // Absolute path (with leading ::)
    let path = Path {
        leading_colons: Some(span),
        segments: vec![
            Ident {
                name: "foo".to_string(),
                span,
            },
            Ident {
                name: "bar".to_string(),
                span,
            },
        ],
    };
    assert_eq!(common::path_str(&path), "::foo::bar");

    // Empty segments (edge case)
    let path = Path {
        leading_colons: None,
        segments: vec![],
    };
    assert_eq!(common::path_str(&path), "");
}

#[test]
fn test_param_kind() {
    use ic_ptree::sys;

    assert_eq!(common::param_kind(ParamKind::In) as u32, sys::OPT_IN);
    assert_eq!(common::param_kind(ParamKind::Out) as u32, sys::OPT_OUT);
    assert_eq!(common::param_kind(ParamKind::Inout) as u32, sys::OPT_INOUT);
}

#[test]
fn test_parse_builtin() {
    let result = common::parse_builtin();

    // Should successfully parse the built-in annotations
    assert!(result.errors.is_empty());
    assert!(!result.tree.is_empty());

    // The built-in annotations should include common ones like @unit, @range, etc.
    // We can't test the exact content without parsing details, but we can verify
    // it parses without errors
}

#[test]
fn test_collect_with() {
    use ic_ptree::sys;

    unsafe {
        // This test is tricky because it involves FFI
        // We'll create a mock scenario to test the logic

        extern "C" fn mock_appender(
            _state: *mut sys::parser_state,
            list: *mut sys::ptree,
            node: *mut sys::ptree,
        ) -> *mut sys::ptree {
            // In a real scenario, this would append node to list
            // For testing, we just return the node
            if list.is_null() { node } else { list }
        }

        let state = std::ptr::null_mut();
        let items = vec![1, 2, 3];

        let result = common::collect_with(state, mock_appender, items, |_item| {
            // Return a mock pointer
            1 as *mut sys::ptree
        });

        // The result should be non-null (our mock returns the first node)
        assert!(!result.is_null());
    }
}

#[test]
fn test_num_undef() {
    unsafe {
        // NUM_UNDEF should point to sys::num_undef
        assert_eq!(
            common::NUM_UNDEF as *const _ as usize,
            std::ptr::addr_of!(ic_ptree::sys::num_undef) as *const _ as usize
        );
    }
}
