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

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, parse_macro_input};

mod attrs;
mod codegen;
mod parsing;

use crate::codegen::{generate_enum_impl, generate_struct_impl};

/// Derive macro for generating CLI command implementations.
///
/// # Struct Example
/// ```ignore
/// #[derive(Command)]
/// struct MyApp {
///     #[option(short, long)]
///     verbose: bool,
///     
///     #[option(short = 'o', long = "output")]
///     output: String,
///     
///     #[option(positional)]
///     files: Vec<String>,
/// }
/// ```
///
/// # Enum Example
/// ```ignore
/// #[derive(Command)]
/// enum MyCommand {
///     Build(BuildOptions),
///     Test(TestOptions),
/// }
/// ```
#[allow(clippy::missing_panics_doc)]
#[proc_macro_derive(Command, attributes(option, command))]
pub fn derive_cli(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let result = match &input.data {
        Data::Struct(data) => generate_struct_impl(ident, data, &input.attrs),
        Data::Enum(data) => generate_enum_impl(ident, data, &input.attrs),
        Data::Union(_) => {
            syn::Error::new_spanned(&input, "unions are not supported").to_compile_error()
        }
    };

    TokenStream::from(result)
}
