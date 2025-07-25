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

#[test]
fn valuetype_forward_declared_then_defined_then_inherited() {
    let idl = r"
        valuetype ForwardDeclared1;
        
        valuetype ForwardDeclared1 {
            public string data;
        };
        
        valuetype ForwardDeclared2 : ForwardDeclared1 {};
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());
}

#[test]
fn valuetype_inherit_from_only_forward_declared() {
    let idl = r"
        valuetype OnlyForward;
        
        valuetype Derived : OnlyForward {};
    ";

    let (result, _, diagnostics) = common::parse_and_resolve(idl);

    // Should have errors - OnlyForward is never defined
    assert!(!result.errors.is_empty());
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn struct_forward_declared_then_defined_then_inherited() {
    let idl = r"
        struct ForwardDeclared1;
        
        struct ForwardDeclared1 {
            string data;
        };
        
        struct ForwardDeclared2 : ForwardDeclared1 {};
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());
}

#[test]
fn interface_forward_declared_then_defined_then_inherited() {
    let idl = r"
        interface ForwardDeclared1;
        
        interface ForwardDeclared1 {
            void doSomething();
        };
        
        interface ForwardDeclared2 : ForwardDeclared1 {};
    ";

    let (result, _, _) = common::parse_and_resolve(idl);
    assert!(result.errors.is_empty());
}
