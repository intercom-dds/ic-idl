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

use quote::quote;
use syn::{Attribute, DataEnum, DataStruct, Ident};

use crate::attrs::{extract_doc_comment, extract_string_attr, has_command_flag};
use crate::parsing::{CliOption, OptionKind, parse_field, variant_to_kebab_case};

/// Generate the Command trait implementation for a struct.
pub fn generate_struct_impl(
    ident: &Ident,
    data: &DataStruct,
    attrs: &[Attribute],
) -> proc_macro2::TokenStream {
    // Parse all fields
    let mut options = Vec::new();
    let mut sections = Vec::new();
    let mut positionals = Vec::new();
    let mut errors = Vec::new();

    for field in &data.fields {
        match parse_field(field) {
            Ok(Some(opt)) => {
                if opt.positional {
                    positionals.push(opt);
                } else if opt.section.is_some() {
                    sections.push(opt);
                } else {
                    options.push(opt);
                }
            }
            Ok(None) => {}
            Err(e) => errors.push(e),
        }
    }

    // If there were errors, return them
    if !errors.is_empty() {
        let compile_errors = errors.iter().map(syn::Error::to_compile_error);
        return quote! { #(#compile_errors)* };
    }

    // Generate command building code
    let command_impl = generate_struct_command(attrs, &options, &sections, !positionals.is_empty());

    // Generate parsing code
    let parse_impl = generate_struct_parse(&options, &sections, &positionals);

    quote! {
        impl ::ic_cli::Command for #ident {
            fn command() -> ::ic_cli::CommandLine {
                #command_impl
            }

            #[allow(clippy::needless_update)]
            fn from_result(result: &::ic_cli::ParseResult) -> Self {
                #parse_impl
            }
        }
    }
}

/// Generate the Command trait implementation for an enum.
pub fn generate_enum_impl(
    ident: &Ident,
    data: &DataEnum,
    attrs: &[Attribute],
) -> proc_macro2::TokenStream {
    // Validate enum structure
    for variant in &data.variants {
        if variant.fields.len() != 1 {
            return syn::Error::new_spanned(variant, "enum variants must have exactly one field")
                .to_compile_error();
        }
    }

    let command_impl = generate_enum_command(data, attrs);
    let parse_impl = generate_enum_parse(data);

    quote! {
        impl ::ic_cli::Command for #ident {
            fn command() -> ::ic_cli::CommandLine {
                #command_impl
            }

            fn from_result(result: &::ic_cli::ParseResult) -> Self {
                #parse_impl
            }
        }
    }
}

/// Generate the command building code for a struct.
fn generate_struct_command(
    attrs: &[Attribute],
    options: &[CliOption],
    sections: &[CliOption],
    has_positionals: bool,
) -> proc_macro2::TokenStream {
    let doc = extract_doc_comment(attrs);
    let name = extract_string_attr("command", attrs)
        .map_or_else(|| quote! { env!("CARGO_PKG_NAME") }, |n| quote! { #n });

    let option_builders = options.iter().map(generate_option_builder);

    let section_builders = sections.iter().map(|opt| {
        let (section_name, section_type) = opt.section.as_ref().unwrap();
        quote! {
            .section(#section_name, #section_type::command())
        }
    });

    quote! {
        ::ic_cli::CommandLine::new(#name)
            .desc(#doc)
            .version(env!("CARGO_PKG_VERSION").to_string())
            .positionals(#has_positionals)
            .opts([
                #(#option_builders),*
            ])
            #(#section_builders)*
    }
}

/// Generate the command building code for an enum.
fn generate_enum_command(data: &DataEnum, attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let doc = extract_doc_comment(attrs);

    let subcommands = data
        .variants
        .iter()
        .filter(|variant| !has_command_flag("external", &variant.attrs))
        .map(|variant| {
            let name = variant_to_kebab_case(&variant.ident);
            let field_type = &variant.fields.iter().next().unwrap().ty;

            quote! {
                #field_type::command().name(#name)
            }
        });

    quote! {
        ::ic_cli::CommandLine::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .desc(#doc)
            .category(
                ic_cli::Category {
                    name: "commands",
                    commands: vec![
                        #(#subcommands,)*
                    ],
                }
            )
    }
}

/// Generate the parsing code for a struct.
fn generate_struct_parse(
    options: &[CliOption],
    sections: &[CliOption],
    positionals: &[CliOption],
) -> proc_macro2::TokenStream {
    let field_parsers = options
        .iter()
        .map(generate_field_parser)
        .chain(sections.iter().map(generate_section_parser))
        .chain(positionals.iter().map(generate_positional_parser));

    quote! {
        let default = Self::default();
        Self {
            #(#field_parsers)*
            ..default
        }
    }
}

/// Generate the parsing code for an enum.
fn generate_enum_parse(data: &DataEnum) -> proc_macro2::TokenStream {
    let match_arms = data
        .variants
        .iter()
        .filter(|variant| !has_command_flag("external", &variant.attrs))
        .map(|variant| {
            let variant_ident = &variant.ident;
            let name = variant_to_kebab_case(&variant.ident);
            let field_type = &variant.fields.iter().next().unwrap().ty;

            quote! {
                #name => Self::#variant_ident(#field_type::from_result(&cmd))
            }
        });

    let catch_all = data
        .variants
        .iter()
        .find(|variant| has_command_flag("external", &variant.attrs))
        .map_or_else(
            || quote! { _ => unreachable!() },
            |variant| {
                let variant_ident = &variant.ident;
                quote! { _ => Self::#variant_ident(cmd.clone()) }
            },
        );

    quote! {
        let cmd = result.subcommand().unwrap();
        match cmd.name() {
            #(#match_arms,)*
            #catch_all,
        }
    }
}

/// Generate an option builder for the `CommandLine`.
fn generate_option_builder(opt: &CliOption) -> proc_macro2::TokenStream {
    let tokens: Vec<String> = opt
        .short
        .map(|c| c.to_string())
        .into_iter()
        .chain(opt.long.clone())
        .collect();

    let doc = &opt.doc;
    let required = opt.required;
    let arg_name = opt.arg_name.as_deref().unwrap_or("arg");

    let kind = match opt.kind {
        OptionKind::Flag => quote! { ::ic_cli::Value::Flag },
        OptionKind::Value => quote! { ::ic_cli::Value::Multiple },
    };

    quote! {
        ::ic_cli::Opt::from([#(#tokens,)*])
            .desc(#doc)
            .required(#required)
            .value(#kind, #arg_name)
    }
}

/// Generate parser for a regular option field.
fn generate_field_parser(opt: &CliOption) -> proc_macro2::TokenStream {
    let field_name = &opt.field_name;
    let token = if let Some(long) = &opt.long {
        long.clone()
    } else if let Some(short) = opt.short {
        short.to_string()
    } else {
        panic!("option must have either short or long form");
    };

    quote! {
        #field_name: result.get_vec(#token)
            .map(|v| ::ic_cli::convert::convert_exit(v))
            .unwrap_or_else(|| default.#field_name),
    }
}

/// Generate parser for a section field.
fn generate_section_parser(opt: &CliOption) -> proc_macro2::TokenStream {
    let field_name = &opt.field_name;
    let field_type = &opt.field_type;

    quote! {
        #field_name: #field_type::from_result(&result),
    }
}

/// Generate parser for a positional argument field.
fn generate_positional_parser(opt: &CliOption) -> proc_macro2::TokenStream {
    let field_name = &opt.field_name;

    quote! {
        #field_name: if result.positionals().is_empty() {
            Default::default()
        } else {
            ::ic_cli::convert::convert_exit(&result.positionals())
        },
    }
}
