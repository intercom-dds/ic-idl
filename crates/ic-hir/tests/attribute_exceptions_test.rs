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
fn test_attribute_with_valid_exceptions() {
    let idl = r#"
        exception InvalidOperation {};
        exception ConfigurationError {};
        
        interface Foo {
            attribute string value
                getraises (InvalidOperation)
                setraises (InvalidOperation, ConfigurationError);
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    
    assert!(result.errors.is_empty());
    
    // Find the interface
    let interface = result.context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Foo")
        .expect("Interface Foo not found");
    
    // Check that the attribute has the correct exceptions
    if let ic_hir::hir::DefKind::Interface(iface) = &interface.1.kind {
        assert_eq!(iface.attributes.len(), 1);
        let attr = &iface.attributes[0];
        assert_eq!(attr.getraises.len(), 1);
        assert_eq!(attr.setraises.len(), 2);
    } else {
        panic!("Expected interface");
    }
}

#[test]
fn test_attribute_with_unknown_exception() {
    let idl = r#"
        exception InvalidOperation {};
        
        interface Foo {
            attribute string value
                getraises (UnknownException)
                setraises (InvalidOperation, AnotherUnknown);
        };
    "#;

    let (result, _, diagnostics) = common::parse_and_resolve(idl);
    
    assert_eq!(result.errors.len(), 2);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_attribute_with_non_exception_type() {
    let idl = r#"
        exception InvalidOperation {};
        struct NotAnException {
            string data;
        };
        
        interface Foo {
            attribute string value
                getraises (InvalidOperation)
                setraises (NotAnException);
        };
    "#;

    let (result, _, diagnostics) = common::parse_and_resolve(idl);
    
    assert_eq!(result.errors.len(), 1);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_readonly_attribute_with_raises() {
    let idl = r#"
        exception InvalidOperation {};
        exception ConfigurationError {};
        
        interface Foo {
            readonly attribute string value raises (InvalidOperation);
            readonly attribute string value2 raises (InvalidOperation, ConfigurationError);
        };
    "#;

    let (result, _, _) = common::parse_and_resolve(idl);
    
    assert!(result.errors.is_empty());
    
    // Find the interface
    let interface = result.context
        .definitions
        .iter()
        .find(|(_, def)| def.ident.name == "Foo")
        .expect("Interface Foo not found");
    
    // Check that the readonly attributes have raises in getraises
    if let ic_hir::hir::DefKind::Interface(iface) = &interface.1.kind {
        assert_eq!(iface.attributes.len(), 2);
        
        let attr1 = &iface.attributes[0];
        assert!(attr1.is_readonly);
        assert_eq!(attr1.getraises.len(), 1);
        assert_eq!(attr1.setraises.len(), 0);
        
        let attr2 = &iface.attributes[1];
        assert!(attr2.is_readonly);
        assert_eq!(attr2.getraises.len(), 2);
        assert_eq!(attr2.setraises.len(), 0);
    } else {
        panic!("Expected interface");
    }
}

#[test]
fn test_readonly_attribute_with_unknown_raises() {
    let idl = r#"
        exception InvalidOperation {};
        
        interface Foo {
            readonly attribute string value raises (UnknownException);
            readonly attribute string value2 raises (InvalidOperation, AnotherUnknown);
        };
    "#;

    let (result, _, diagnostics) = common::parse_and_resolve(idl);
    
    assert_eq!(result.errors.len(), 2);
    insta::assert_snapshot!(diagnostics);
}

#[test]
fn test_readonly_attribute_with_non_exception_raises() {
    let idl = r#"
        struct NotAnException {
            string data;
        };
        
        interface Foo {
            readonly attribute string value raises (NotAnException);
        };
    "#;

    let (result, _, diagnostics) = common::parse_and_resolve(idl);
    
    assert_eq!(result.errors.len(), 1);
    insta::assert_snapshot!(diagnostics);
}