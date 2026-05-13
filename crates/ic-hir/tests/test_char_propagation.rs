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

mod common;

use ic_hir::hir::{DefKind, Numeric};

#[test]
fn test_character_literal_propagation() {
    let idl = r"
        const char C1 = 'A';
        const char C2 = '\n';
        const char C3 = '\x41';
        const char C4 = '\0';
        const char C5 = '\\';
    ";

    let (hir, _, _) = common::parse_and_resolve(idl);
    assert!(hir.errors.is_empty(), "HIR errors: {:?}", hir.errors);

    // Check the character constants
    let mut found_chars = std::collections::HashMap::new();

    for def in &hir {
        if let DefKind::Const(c) = &def.kind
            && let Numeric::Char(ch) | Numeric::WChar(ch) = &c.value
        {
            found_chars.insert(def.ident.name.clone(), *ch);
        }
    }

    // Verify expected values
    assert_eq!(found_chars.get("C1"), Some(&'A'));
    assert_eq!(found_chars.get("C2"), Some(&'\n'));
    assert_eq!(found_chars.get("C3"), Some(&'A')); // \x41 is 'A'
    assert_eq!(found_chars.get("C4"), Some(&'\0'));
    assert_eq!(found_chars.get("C5"), Some(&'\\'));
}
