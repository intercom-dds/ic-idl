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

#[test]
fn test_interface_nested_types() {
    let input = r"
        interface FileSystem {
            struct FileInfo {
                string name;
                long size;
            };
            
            enum FileType {
                FILE,
                DIRECTORY
            };
            
            exception NotFound {
                string path;
            };
            
            // Methods using nested types
            FileInfo getInfo(in string path) raises (NotFound);
            FileType getType(in string path);
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );
}

#[test]
fn test_interface_nested_type_in_typedef() {
    let input = r"
        interface Service {
            struct Request {
                string method;
                any params;
            };
            
            typedef sequence<Request> RequestList;
            
            RequestList getPendingRequests();
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        result.errors.is_empty(),
        "Expected no errors, got: {:?}",
        result.errors
    );
}

#[test]
fn test_interface_type_not_visible_outside() {
    let input = r"
        interface Service {
            struct InternalData {
                long value;
            };
        };
        
        // This should fail - InternalData is not visible outside Service
        struct Container {
            InternalData data;
        };
    ";

    let parsed = ic_parse::from_str(input);
    assert!(!parsed.tree.is_empty(), "Failed to parse input");
    assert!(
        parsed.errors.is_empty(),
        "Parse errors: {:?}",
        parsed.errors
    );

    let result = ic_hir::from_ast(parsed.tree);
    assert!(
        !result.errors.is_empty(),
        "Expected error for type not visible outside interface"
    );
    let error_msg = format!("{:?}", result.errors[0]);
    assert!(
        error_msg.contains("undefined type"),
        "Error message should contain 'undefined type': {error_msg}"
    );
}
