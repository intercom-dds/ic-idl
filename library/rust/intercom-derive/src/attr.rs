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

use std::collections::HashSet;

use syn::meta::ParseNestedMeta;
use syn::{Attribute, LitInt, LitStr};

const ID: &str = "id";
const RENAME: &str = "rename";
const NON_SERIALIZED: &str = "non_serialized";

pub fn is_unique<T, P, U>(iter: T, pred: P) -> bool
where
    T: IntoIterator,
    P: Fn(&T::Item) -> U,
    U: Eq + std::hash::Hash,
{
    let mut set = HashSet::new();
    iter.into_iter().all(move |v| set.insert(pred(&v)))
}

#[derive(Default)]
pub struct FieldAttrib {
    pub id: Option<usize>,
    pub rename: Option<String>,
    pub non_serialized: Option<bool>,
}

fn extract_meta(meta: &ParseNestedMeta<'_>, parsed: &mut FieldAttrib) -> syn::Result<()> {
    if meta.path.is_ident(ID) {
        let lit = meta.value()?.parse::<LitInt>()?;
        parsed.id = Some(lit.base10_parse()?);
    } else if meta.path.is_ident(RENAME) {
        let lit = meta.value()?.parse::<LitStr>()?;
        parsed.rename = Some(lit.value());
    } else if meta.path.is_ident(NON_SERIALIZED) {
        parsed.non_serialized = Some(true);
    } else {
        return Err(meta.error("Unknown attribute"));
    }
    Ok(())
}

pub fn field(attrs: &[Attribute]) -> syn::Result<FieldAttrib> {
    let mut parsed = FieldAttrib::default();
    let attrs = attrs.iter().filter(|v| v.path().is_ident("cts"));
    for attr in attrs {
        attr.parse_nested_meta(|meta| extract_meta(&meta, &mut parsed))?;
    }
    Ok(parsed)
}
