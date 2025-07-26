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

use syn::{Attribute, Expr, ExprLit, Lit, Meta, Result};

/// Parsed attributes from #[option(...)]
#[derive(Debug, Default)]
pub struct OptionAttrs {
    /// Whether short option was explicitly set
    pub short: Option<ShortOption>,
    /// Whether long option was explicitly set
    pub long: Option<LongOption>,
    /// Custom argument name
    pub arg_name: Option<String>,
    /// Whether this is a positional argument
    pub positional: bool,
    /// Whether this option is required
    pub required: bool,
    /// Section for grouping
    pub section: Option<String>,
}

#[derive(Debug)]
pub enum ShortOption {
    /// Use default (first character of field name)
    Auto,
    /// Use specific character
    Explicit(char),
}

#[derive(Debug)]
pub enum LongOption {
    /// Use default (kebab-case field name)
    Auto,
    /// Use specific string
    Explicit(String),
}

/// Parse attributes from a field to extract option configuration.
pub fn parse_option_attrs(attrs: &[Attribute]) -> Result<Option<OptionAttrs>> {
    let mut result = OptionAttrs::default();
    let mut found_option = false;

    for attr in attrs {
        if !attr.path().is_ident("option") {
            continue;
        }

        found_option = true;
        attr.parse_nested_meta(|meta| {
            match meta
                .path
                .get_ident()
                .map(std::string::ToString::to_string)
                .as_deref()
            {
                Some("short") => {
                    result.short = Some(if meta.input.peek(syn::Token![=]) {
                        meta.input.parse::<syn::Token![=]>()?;
                        let lit: syn::LitChar = meta.input.parse()?;
                        ShortOption::Explicit(lit.value())
                    } else {
                        ShortOption::Auto
                    });
                }
                Some("long") => {
                    result.long = Some(if meta.input.peek(syn::Token![=]) {
                        meta.input.parse::<syn::Token![=]>()?;
                        let lit: syn::LitStr = meta.input.parse()?;
                        LongOption::Explicit(lit.value())
                    } else {
                        LongOption::Auto
                    });
                }
                Some("arg") => {
                    meta.input.parse::<syn::Token![=]>()?;
                    let lit: syn::LitStr = meta.input.parse()?;
                    result.arg_name = Some(lit.value());
                }
                Some("section") => {
                    meta.input.parse::<syn::Token![=]>()?;
                    let lit: syn::LitStr = meta.input.parse()?;
                    result.section = Some(lit.value());
                }
                Some("positional") => {
                    result.positional = true;
                }
                Some("required") => {
                    result.required = true;
                }
                Some(unknown) => {
                    return Err(meta.error(format!("unknown option attribute: '{unknown}'")));
                }
                None => {
                    return Err(meta.error("expected attribute name"));
                }
            }
            Ok(())
        })?;
    }

    if found_option {
        // Validate that positional arguments don't have short/long options
        if result.positional && (result.short.is_some() || result.long.is_some()) {
            return Err(syn::Error::new_spanned(
                attrs.first().unwrap(),
                "positional arguments cannot have short or long options",
            ));
        }
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

/// Extract documentation from doc comment attributes.
pub fn extract_doc_comment(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        if let Meta::NameValue(meta) = &attr.meta {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) = &meta.value
            {
                lines.push(s.value().trim_start().to_string());
            }
        }
    }

    lines.join("\n")
}

/// Extract a string attribute value (e.g., #[command = "name"]).
pub fn extract_string_attr(name: &str, attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident(name) {
            continue;
        }

        if let Meta::NameValue(meta) = &attr.meta {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) = &meta.value
            {
                return Some(s.value());
            }
        }
    }

    None
}
