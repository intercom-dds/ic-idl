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

use ic_emit::case;
use syn::ext::IdentExt;
use syn::{Field, Path, Result, Type};

use crate::attrs::{LongOption, ShortOption, extract_doc_comment, parse_option_attrs};

/// Represents a parsed CLI option field.
pub struct CliOption {
    /// The field identifier
    pub field_name: syn::Ident,
    /// The field type
    pub field_type: syn::Type,
    /// Short option character (e.g., 'v' for -v)
    pub short: Option<char>,
    /// Long option name (e.g., "verbose" for --verbose)
    pub long: Option<String>,
    /// Argument name for help text
    pub arg_name: Option<String>,
    /// Whether this is a positional argument
    pub positional: bool,
    /// Whether this option is required
    pub required: bool,
    /// Documentation string extracted from doc comments
    pub doc: String,
    /// Section name for grouping options
    pub section: Option<(String, Path)>,
    /// The kind of option (flag vs value)
    pub kind: OptionKind,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptionKind {
    /// Boolean flag (no value required)
    Flag,
    /// Option that takes a value
    Value,
}

/// Parse a struct field into a CLI option.
pub fn parse_field(field: &Field) -> Result<Option<CliOption>> {
    let Some(ident) = &field.ident else {
        return Err(syn::Error::new_spanned(
            field,
            "tuple struct fields are not supported",
        ));
    };

    let attrs = parse_option_attrs(&field.attrs)?;
    let Some(attrs) = attrs else {
        return Ok(None);
    };

    // Determine option kind based on type
    let (kind, field_type) = match &field.ty {
        Type::Path(ty) if ty.path.is_ident("bool") => (OptionKind::Flag, field.ty.clone()),
        _ => (OptionKind::Value, field.ty.clone()),
    };

    // Derive short option
    let short = match attrs.short {
        Some(ShortOption::Auto) => Some(derive_short_option(ident)),
        Some(ShortOption::Explicit(c)) => Some(c),
        None => None,
    };

    // Derive long option
    let long = match attrs.long {
        Some(LongOption::Auto) => Some(derive_long_option(ident)),
        Some(LongOption::Explicit(s)) => Some(s),
        None => None,
    };

    // Extract section info
    let section = attrs.section.map(|name| {
        // We need the field type path for section handling
        if let Type::Path(ty) = &field.ty {
            (name, ty.path.clone())
        } else {
            // For non-path types, create a dummy path
            // This shouldn't happen in practice for section fields
            (name, syn::parse_quote!(()))
        }
    });

    Ok(Some(CliOption {
        field_name: ident.clone(),
        field_type,
        short,
        long,
        arg_name: attrs.arg_name,
        positional: attrs.positional,
        required: attrs.required,
        doc: extract_doc_comment(&field.attrs),
        section,
        kind,
    }))
}

/// Derive a short option from field name (first character).
fn derive_short_option(ident: &syn::Ident) -> char {
    ident
        .to_string()
        .chars()
        .next()
        .expect("field name cannot be empty")
}

/// Derive a long option from field name (kebab-case conversion).
fn derive_long_option(ident: &syn::Ident) -> String {
    ident
        .to_string()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Convert a variant name to kebab-case for subcommand names.
pub fn variant_to_kebab_case(ident: &syn::Ident) -> String {
    case::kebab(ident.unraw().to_string())
}
