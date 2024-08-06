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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
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
use proc_macro::TokenStream;
use proc_macro2::{Ident, Punct, TokenTree};
use quote::{quote, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, ExprLit, Field, Meta,
    Token, Type,
};

fn derive_short(input: &Ident, value: &Option<syn::LitChar>) -> char {
    if let Some(value) = value {
        value.value()
    } else {
        input
            .to_string()
            .chars()
            .next()
            .expect("option cannot be empty")
    }
}

fn derive_long(input: &Ident, value: &Option<syn::LitStr>) -> String {
    if let Some(value) = value {
        value.value()
    } else {
        input
            .to_string()
            .chars()
            .map(|v| if v.is_ascii_alphanumeric() { v } else { '-' })
            .collect()
    }
}

struct Opt {
    tokens: Vec<String>,
    comment: String,
    kind: Kind,
    required: bool,
    arg_name: String,
    positional: bool,
}

impl ToTokens for Opt {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let Opt {
            tokens,
            comment,
            kind,
            required,
            arg_name,
            ..
        } = self;

        let tree = quote! {
            ::ic_cli::Opt::from([#(#tokens,)*])
                .desc(#comment)
                .required(#required)
                .value(#kind, #arg_name)
        };
        tree.to_tokens(stream);
    }
}

#[derive(PartialEq)]
enum Kind {
    Flag,
    Option,
}

impl ToTokens for Kind {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let tree = if *self == Kind::Flag {
            quote! { ::ic_cli::Value::Flag }
        } else {
            quote! { ::ic_cli::Value::Single }
        };
        tree.to_tokens(stream);
    }
}

fn doc_attr(attrs: &Vec<Attribute>) -> String {
    let mut lines = vec![];
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        if let Meta::NameValue(syn::MetaNameValue {
            value:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }),
            ..
        }) = &attr.meta
        {
            lines.push(s.value().trim_start().to_string());
        }
    }
    lines.join("\n")
}

fn attr_lit<'a>(name: &str, attrs: &'a [Attribute]) -> Option<&'a ExprLit> {
    let attr = attrs.iter().find(|v| v.path().is_ident(name));
    if let Some(attr) = attr {
        if let Meta::NameValue(syn::MetaNameValue {
            value: syn::Expr::Lit(expr),
            ..
        }) = &attr.meta
        {
            return Some(expr);
        }
    }
    None
}

fn attr_str(name: &str, attrs: &[Attribute]) -> Option<String> {
    if let Some(ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = attr_lit(name, attrs)
    {
        Some(s.value())
    } else {
        None
    }
}

#[derive(Default)]
struct OptAttr {
    short: (bool, Option<syn::LitChar>),
    long: (bool, Option<syn::LitStr>),
    arg_name: Option<syn::LitStr>,
    positional: bool,
    required: bool,
    is_option: bool,
}

fn option_attr(attrs: &Vec<Attribute>) -> OptAttr {
    fn parse_expr<T: Parse>(input: ParseStream) -> Option<T> {
        if input.peek(Token![=]) {
            let _: Punct = input.parse().unwrap();
            Some(input.parse().unwrap())
        } else {
            None
        }
    }

    let mut arg_attr = OptAttr::default();
    for attr in attrs {
        if !attr.path().is_ident("option") {
            continue;
        }

        arg_attr.is_option = true;
        let _ = attr.parse_args_with(|input: ParseStream| {
            while let Some(token) = input.parse()? {
                if let TokenTree::Ident(value) = token {
                    if value == "short" {
                        arg_attr.short = (true, parse_expr(input));
                    } else if value == "long" {
                        arg_attr.long = (true, parse_expr(input));
                    } else if value == "arg" {
                        arg_attr.arg_name = parse_expr(input);
                    } else if value == "positional" {
                        arg_attr.positional = true;
                    } else if value == "required" {
                        arg_attr.required = true;
                    } else {
                        panic!("unknown attribute: '{value}'");
                    }
                }
            }
            Ok(())
        });
    }

    assert!(
        !((arg_attr.short.0 || arg_attr.long.0) && arg_attr.positional),
        "options cannot be positionals"
    );
    arg_attr
}

fn handle_option(field: &Field) -> Option<Opt> {
    let Some(ref ident) = field.ident else {
        panic!("tuple structs are not supported");
    };

    let mut tokens = vec![];
    let attrs = option_attr(&field.attrs);
    if !attrs.is_option {
        return None;
    }

    if attrs.short.0 {
        tokens.push(derive_short(ident, &attrs.short.1).to_string());
    }
    if attrs.long.0 {
        tokens.push(derive_long(ident, &attrs.long.1));
    }

    let kind = if let Type::Path(ref ty) = field.ty {
        if ty.path.is_ident("bool") {
            Kind::Flag
        } else {
            Kind::Option
        }
    } else {
        panic!("unsupported type");
    };

    let arg_name = attrs
        .arg_name
        .map_or_else(|| "arg".to_string(), |v| v.value());

    Some(Opt {
        tokens,
        comment: doc_attr(&field.attrs),
        arg_name,
        kind,
        required: attrs.required,
        positional: attrs.positional,
    })
}

fn enum_command(input: &DataEnum, attrs: &Vec<Attribute>) -> proc_macro2::TokenStream {
    let commands = input.variants.iter().map(|v| {
        let name = case::kebab(v.ident.unraw().to_string());
        let field = v.fields.iter().next().unwrap();

        quote! {
            #field::command().name(#name)
        }
    });

    let name = quote! { env!("CARGO_PKG_NAME") };
    let version = quote! { env!("CARGO_PKG_VERSION") };
    let doc = doc_attr(attrs);

    quote! {
        ::ic_cli::CommandLine::new(#name)
            .version(#version)
            .desc(#doc)
            .category(
                ic_cli::Category {
                    name: "commands",
                    commands: vec![
                        #(#commands,)*
                    ],
                }
            )
    }
}

fn struct_command(input: &DataStruct, attrs: &Vec<Attribute>) -> proc_macro2::TokenStream {
    let doc = doc_attr(attrs);
    let attr = attr_str("command", attrs);
    let mut options = vec![];
    let mut positionals = false;

    for field in &input.fields {
        if let Some(option) = handle_option(field) {
            if option.positional {
                positionals = true;
            } else {
                options.push(option);
            }
        }
    }

    let name = if let Some(name) = attr {
        quote! { #name }
    } else {
        quote! {
            env!("CARGO_PKG_NAME")
        }
    };

    quote! {
        ::ic_cli::CommandLine::new(#name)
            .desc(#doc)
            .version(env!("CARGO_PKG_VERSION").to_string())
            .positionals(#positionals)
            .opts([
                #(#options),*
            ])
    }
}

fn enum_parse(input: &DataEnum) -> proc_macro2::TokenStream {
    let variants = input.variants.iter().map(|v| {
        let name = case::kebab(v.ident.unraw().to_string());
        let variant = v
            .fields
            .iter()
            .next()
            .expect("Tuple variants must contain exactly one member");

        let ident = &v.ident;
        let path = &variant.ty;

        quote! {
            #name => Self::#ident(#path::from_result(&cmd))
        }
    });

    quote! {
        let cmd = result.subcommand().unwrap();
        match cmd.name() {
            #(#variants,)*
            _ => unreachable!(),
        }
    }
}

fn struct_parse(input: &DataStruct) -> proc_macro2::TokenStream {
    let mut stream = proc_macro2::TokenStream::new();
    for field in &input.fields {
        let ident = field.ident.as_ref().unwrap();
        let attrs = option_attr(&field.attrs);

        if !attrs.is_option {
            continue;
        }

        let tree = if attrs.positional {
            quote! {
                #ident: if result.positionals().is_empty() {
                    Default::default()
                } else {
                    ::ic_cli::convert::convert_exit(&result.positionals())
                },
            }
        } else {
            let token = if attrs.long.0 {
                derive_long(field.ident.as_ref().unwrap(), &attrs.long.1)
            } else {
                derive_short(field.ident.as_ref().unwrap(), &attrs.short.1).to_string()
            };

            quote! {
                #ident: result.get_vec(#token)
                    .map(|v| ::ic_cli::convert::convert_exit(v))
                    .unwrap_or_else(|| default.#ident),
            }
        };
        tree.to_tokens(&mut stream);
    }

    quote! {
        let default = Self::default();
        Self {
            #stream
            ..default
        }
    }
}

#[allow(clippy::missing_panics_doc)]
#[proc_macro_derive(Command, attributes(option, command))]
pub fn derive_cli(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let (command, parse) = match &input.data {
        Data::Struct(v) => (struct_command(v, &input.attrs), struct_parse(v)),
        Data::Enum(v) => (enum_command(v, &input.attrs), enum_parse(v)),
        Data::Union(_) => panic!("unions are not supported"),
    };

    let expanded = quote! {
        impl ::ic_cli::Command for #ident {
            fn command() -> ::ic_cli::CommandLine {
                #command
            }

            fn from_result(result: &::ic_cli::ParseResult) -> Self {
                #parse
            }
        }
    };
    TokenStream::from(expanded)
}
