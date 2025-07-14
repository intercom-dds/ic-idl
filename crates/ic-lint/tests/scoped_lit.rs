// Copyright 2025 KONGSBERG
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
use common::test_lint;

#[test]
fn test_scoped_enum_literal() {
    let source = r"
        enum Color {
            RED,
            GREEN,
            BLUE
        };
        
        const Color DEFAULT_COLOR = Color::RED;
        const Color SECONDARY = Color::GREEN;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_scoped_bitmask_literal() {
    let source = r"
        bitmask<unsigned long> Permissions {
            READ = 0x01,
            WRITE = 0x02,
            EXECUTE = 0x04
        };
        
        const Permissions DEFAULT_PERMS = Permissions::READ | Permissions::WRITE;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_unscoped_literals() {
    let source = r"
        enum Status {
            OK,
            ERROR,
            PENDING
        };
        
        const Status GOOD = OK;
        const Status BAD = ERROR;
    ";

    let output = test_lint(source);
    assert!(output.is_empty(), "Should not warn for unscoped literals");
}

#[test]
fn test_mixed_scoped_unscoped() {
    let source = r"
        enum Mode {
            NORMAL,
            FAST,
            SLOW
        };
        
        const Mode MODE1 = NORMAL;
        const Mode MODE2 = Mode::FAST;
        const Mode MODE3 = SLOW;
        const Mode MODE4 = Mode::SLOW;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_nested_scoped_access() {
    let source = r"
        module Types {
            enum Result {
                SUCCESS,
                FAILURE
            };
        };
        
        const Types::Result GOOD = Types::Result::SUCCESS;
    ";

    assert_snapshot!(test_lint(source));
}

#[test]
fn test_scoped_in_expressions() {
    let source = r"
        enum Level {
            LOW = 1,
            MEDIUM = 5,
            HIGH = 10
        };
        
        const boolean IS_HIGH = (Level::HIGH > Level::MEDIUM);
        const Level NEXT = (Level::LOW + 1);
    ";

    assert_snapshot!(test_lint(source));
}
