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

//! C# code generator for IDL.
//!
//! This crate implements the OMG IDL4 to C# Language Mapping specification,
//! generating C# code from IDL definitions.

mod codegen;

use ic_cli::Command;
use ic_emit::File;
use ic_emit::case::Case;
use ic_hir_xform::rename;

/// C# keywords that must be escaped with `@` prefix
#[rustfmt::skip]
const CSHARP_KEYWORDS: &[&str] = &[
    // Reserved keywords
    "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked", "class",
    "const", "continue", "decimal", "default", "delegate", "do", "double", "else", "enum", "event",
    "explicit", "extern", "false", "finally", "fixed", "float", "for", "foreach", "goto", "if",
    "implicit", "in", "int", "interface", "internal", "is", "lock", "long", "namespace", "new",
    "null", "object", "operator", "out", "override", "params", "private", "protected", "public",
    "readonly", "ref", "return", "sbyte", "sealed", "short", "sizeof", "stackalloc", "static",
    "string", "struct", "switch", "this", "throw", "true", "try", "typeof", "uint", "ulong",
    "unchecked", "unsafe", "ushort", "using", "virtual", "void", "volatile", "while",

    // Contextual keywords that cause compilation errors when used as identifiers
    "file", "record", "required", "scoped",
];

/// Reserved member names that conflict with inherited `System.Object` methods
const RESERVED_MEMBER_NAMES: &[&str] = &[
    "Equals",
    "GetHashCode",
    "GetType",
    "ToString",
    "MemberwiseClone",
    "Finalize",
];

/// Members inherited from `System.Exception` that require `new` keyword to hide
pub const EXCEPTION_MEMBER_NAMES: &[&str] = &[
    "Data",
    "HelpLink",
    "HResult",
    "InnerException",
    "Message",
    "Source",
    "StackTrace",
    "TargetSite",
];

/// Options for C# code generation.
#[derive(Command, Copy, Clone, Debug, Default)]
pub struct CSharpOptions {
    /// Do not rename generated types
    #[option(long)]
    pub no_rename: bool,

    /// Emit each constant in its own class
    #[option(long)]
    pub const_classes: bool,
}

fn escape_csharp(ctx: rename::RenameContext) -> Option<String> {
    let name = ctx.name;

    if CSHARP_KEYWORDS.contains(&name) {
        return Some(format!("@{name}"));
    }

    match ctx.kind {
        rename::IdentifierKind::Member(_)
        | rename::IdentifierKind::Variant
        | rename::IdentifierKind::Operation
        | rename::IdentifierKind::Attribute
        | rename::IdentifierKind::Struct
        | rename::IdentifierKind::Union
        | rename::IdentifierKind::Valuetype
        | rename::IdentifierKind::Exception
            if RESERVED_MEMBER_NAMES.contains(&name) =>
        {
            return Some(format!("{name}_"));
        }
        _ => {}
    }

    None
}

const DOTNET_CONVENTION: rename::Convention = rename::Convention {
    struct_type: Some(Case::Pascal),
    union_type: Some(Case::Pascal),
    enum_type: Some(Case::Pascal),
    interface: Some(Case::Pascal),
    valuetype: Some(Case::Pascal),
    alias: Some(Case::Pascal),
    bitmask: Some(Case::Pascal),
    bitset: Some(Case::Pascal),
    exception: Some(Case::Pascal),
    annotation: Some(Case::Pascal),
    member: Some(Case::Pascal),
    variant: Some(Case::Pascal),
    enumerator: Some(Case::Pascal),
    bit_flag: Some(Case::Pascal),
    bitset_field: Some(Case::Pascal),
    constant: Some(Case::Pascal),
    module: Some(Case::Pascal),
    operation: Some(Case::Pascal),
    attribute: Some(Case::Pascal),
    parameter: Some(Case::Camel),
    name_preprocessor: Some(rename::strip_common_suffixes),
    strip_enum_prefix: true,
};

/// Generate C# code from the HIR.
#[must_use]
pub fn codegen_csharp(
    hir: &ic_hir::ResolvedGraph,
    source_map: &ic_vfs::SourceMap,
    options: CSharpOptions,
) -> Vec<File> {
    // Resolve and strip typedefs
    let hir = ic_hir_xform::strip_typedefs::transform(hir.clone());

    // Squash modules together
    let hir = ic_hir_xform::squash_modules::transform(hir);

    // Group constants into a `Constants` class by default
    let hir = if options.const_classes {
        hir
    } else {
        ic_hir_xform::move_constants::transform(hir, |name| format!("{name}_"))
    };

    // Apply naming convention based on options
    let convention = if options.no_rename {
        rename::Convention::default()
    } else {
        DOTNET_CONVENTION
    };

    let hir = ic_hir_xform::rename::transform(
        hir,
        &rename::Target {
            convention,
            keyword_escape: Some(escape_csharp),
            ..Default::default()
        },
    );

    let generator = codegen::CSharpGen::new(&hir, source_map, options);
    generator.generate()
}
