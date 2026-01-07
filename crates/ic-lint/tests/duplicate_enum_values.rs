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

use ic_vfs::SourceMap;

#[test]
fn test_duplicate_enum_values_error() {
    let input = r"
        enum Color {
            RED = 1,
            GREEN = 2,
            BLUE = 1  // Duplicate value - should be an error
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, &vfs);
    let hir = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed.tree));
    let report = ic_lint::lint_hir(&hir, &vfs);

    // Should have errors for duplicate value
    assert!(!report.errors.is_empty());
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.to_string().contains("duplicate value"))
    );
}

#[test]
fn test_duplicate_bitmask_values_allowed() {
    let input = r"
        bitmask Permissions {
            READ = 1,
            WRITE = 2,
            EXECUTE = 4,
            READ_WRITE = 3,  // Alias for READ | WRITE - should be allowed
            ALL = 7          // Alias for all permissions - should be allowed
        };
    ";

    let mut vfs = SourceMap::default();
    let file_id = vfs.embed_with_name("<test>", input);
    let parsed = ic_parse::from_file(file_id, &vfs);
    let hir = ic_hir_lower::from_ast(ic_hir_lower::AstInput::User(parsed.tree));
    let report = ic_lint::lint_hir(&hir, &vfs);

    // Should have no errors - duplicate values in bitmasks are allowed
    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.warnings.len(), 0);
}
