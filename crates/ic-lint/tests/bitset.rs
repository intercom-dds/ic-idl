// Copyright 2026 KONGSBERG
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

use insta::assert_snapshot;

mod common;
use common::test_lint_hir;

use crate::common::lint_hir;

#[test]
fn bitset_basic() {
    assert_snapshot!(test_lint_hir(
        r"
bitset Flags {
    bitfield<4> low;
    bitfield<4> high;
};
"
    ));
}

#[test]
fn bitset_with_base() {
    assert_snapshot!(test_lint_hir(
        r"
bitset Base {
    bitfield<8> field;
};

bitset Extended : Base {
    bitfield<8> extra;
};
"
    ));
}

#[test]
fn bitset_multiple() {
    assert_snapshot!(test_lint_hir(
        r"
bitset FlagsA {
    bitfield<1> a;
};

bitset FlagsB {
    bitfield<1> b;
};

bitset FlagsC {
    bitfield<1> c;
};
"
    ));
}

#[test]
fn no_bitset() {
    let output = lint_hir(
        r"
bitmask Permissions {
    READ,
    WRITE,
    EXECUTE
};
",
    );

    assert!(output.warnings.is_empty());
    assert!(output.errors.is_empty());
}
