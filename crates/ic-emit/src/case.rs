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

//! Utility functions for converting identifiers to different capitalizations
//! and naming conventions.

/// The supported naming conventions.
#[derive(Copy, Clone, Debug)]
pub enum Case {
    /// `snake_case`
    Snake,

    /// `camelCase`
    Camel,

    /// `PascalCase`
    Pascal,

    /// `kebab-case`
    Kebab,
}

/// Converts the given string to the specified capitalization.
///
/// Whitespace and ASCII punctuation characters are treated as delimiters
/// between words. Multiple consecutive delimiters will be stripped. For
/// example, "foo bar...baz" would be converted to "fooBarBaz" in `camelCase`.
///
/// In most cases you'll probably want to use [`snake`], [`camel`], [`pascal`],
/// or [`kebab`] instead of this function.
///
/// # Example
///
/// ```rust
/// # use ic_emit::case::{convert, Case};
/// #
/// let converted = convert("foo_bar_baz", Case::Pascal);
/// assert_eq!(converted, "FooBarBaz");
/// ```
pub fn convert<A: AsRef<str>>(input: A, case: Case) -> String {
    let delim = match case {
        Case::Kebab => '-',
        _ => '_',
    };

    let state = Converter {
        first: true,
        case,
        delim,
    };
    state.convert(input.as_ref())
}

/// Converts the given string to `snake_case`. See [`convert`] for more
/// information.
///
/// # Example
///
/// ```rust
/// # use ic_emit::case::snake;
/// #
/// let converted = snake("FooBarBaz3");
/// assert_eq!(converted, "foo_bar_baz3");
/// ````
pub fn snake<A: AsRef<str>>(input: A) -> String {
    convert(input, Case::Snake)
}

/// Converts the given string to `camelCase`. See [`convert`] for more
/// information.
///
/// # Example
///
/// ```rust
/// # use ic_emit::case::camel;
/// #
/// let converted = camel("foo_Bar_baz3");
/// assert_eq!(converted, "fooBarBaz3");
/// ````
pub fn camel<A: AsRef<str>>(input: A) -> String {
    convert(input, Case::Camel)
}

/// Converts the given string to `PascalCase`. See [`convert`] for more
/// information.
///
/// # Example
///
/// ```rust
/// # use ic_emit::case::pascal;
/// #
/// let converted = pascal("foo_Bar_baz3");
/// assert_eq!(converted, "FooBarBaz3");
/// ````
pub fn pascal<A: AsRef<str>>(input: A) -> String {
    convert(input, Case::Pascal)
}

/// Converts the given string to `kebab-case`. See [`convert`] for more
/// information.
///
/// # Example
///
/// ```rust
/// # use ic_emit::case::kebab;
/// #
/// let converted = kebab("FooBar_baz3");
/// assert_eq!(converted, "foo-bar-baz3");
/// ````
pub fn kebab<A: AsRef<str>>(input: A) -> String {
    convert(input, Case::Kebab)
}

struct Converter {
    first: bool,
    case: Case,
    delim: char,
}

fn is_delim(c: char) -> bool {
    c.is_whitespace() || c.is_ascii_punctuation()
}

impl Converter {
    fn append(&mut self, word: &str, buffer: &mut String) {
        if !word.is_empty() {
            match self.case {
                Case::Pascal => Self::to_pascal(word, buffer),
                Case::Camel => self.to_camel(word, buffer),
                Case::Snake | Case::Kebab => self.snake_delim(word, buffer),
            }
        }
        self.first = false;
    }

    fn snake_delim(&self, word: &str, buffer: &mut String) {
        if !self.first {
            buffer.push(self.delim);
        }
        *buffer += &word.to_lowercase();
    }

    fn to_pascal(word: &str, buffer: &mut String) {
        let mut iter = word.chars();
        if let Some(c) = iter.next() {
            buffer.extend(c.to_uppercase());
        }
        buffer.extend(iter.flat_map(char::to_lowercase));
    }

    fn to_camel(&self, word: &str, buffer: &mut String) {
        let mut iter = word.chars();
        if let Some(c) = iter.next() {
            if self.first {
                buffer.extend(c.to_lowercase());
            } else {
                buffer.extend(c.to_uppercase());
            }
        }
        buffer.extend(iter.flat_map(char::to_lowercase));
    }

    fn convert(mut self, input: &str) -> String {
        let mut start = 0;
        let mut was_upper = false;
        let mut iter = input.chars().enumerate().peekable();
        let mut buffer = String::with_capacity(2 * input.len());

        while let Some((i, c)) = iter.next() {
            if is_delim(c) {
                if i == start {
                    start += 1;
                }
                continue;
            }

            if let Some((_, peek)) = iter.peek() {
                let len = i - start;

                if is_delim(*peek) || (c.is_lowercase() && peek.is_uppercase()) {
                    self.append(&input[start..=(start + len)], &mut buffer);
                    start = i + 1;
                } else if was_upper && c.is_uppercase() && peek.is_lowercase() {
                    self.append(&input[start..start + len], &mut buffer);
                    start = i;
                }
            } else {
                self.append(&input[start..], &mut buffer);
            }
            was_upper = c.is_uppercase();
        }
        buffer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_char_suffix_snake() {
        // snake
        assert_eq!(snake("suffix_t"), "suffix_t");
        assert_eq!(snake("suffix_1"), "suffix_1");
        assert_eq!(snake("suffix__"), "suffix");
        assert_eq!(snake("suffix_1_"), "suffix_1");
        assert_eq!(snake("abc_t_def"), "abc_t_def");
        assert_eq!(snake("FooBarBaz"), "foo_bar_baz");
        assert_eq!(snake("Foo1bar"), "foo1bar");
        assert_eq!(snake("foo bar.baz"), "foo_bar_baz");

        // pascal
        assert_eq!(pascal("suffix_t"), "SuffixT");
        assert_eq!(pascal("suffix_1"), "Suffix1");
        assert_eq!(pascal("suffix__"), "Suffix");
        assert_eq!(pascal("suffix_1_"), "Suffix1");
        assert_eq!(pascal("abc_t_def"), "AbcTDef");

        // camel
        assert_eq!(camel("suffix_t"), "suffixT");
        assert_eq!(camel("suffix_1"), "suffix1");
        assert_eq!(camel("suffix__"), "suffix");
        assert_eq!(camel("suffix_1_"), "suffix1");
        assert_eq!(camel("abc_t_def"), "abcTDef");

        // kebab
        assert_eq!(kebab("suffix_t"), "suffix-t");
        assert_eq!(kebab("suffix_1"), "suffix-1");
        assert_eq!(kebab("suffix__"), "suffix");
        assert_eq!(kebab("suffix_1_"), "suffix-1");
        assert_eq!(kebab("abc_t_def"), "abc-t-def");
    }

    #[test]
    fn multiple_upper() {
        assert_eq!(pascal("JSONParser"), "JsonParser");
        assert_eq!(snake("IDLType"), "idl_type");
        assert_eq!(snake("PROTO3Buffer"), "proto3buffer");
    }

    #[test]
    fn single_underscore() {
        assert_eq!(snake("P_Arbitration_AU_PSM"), "p_arbitration_au_psm");
        assert_eq!(pascal("P_Arbitration_AU_PSM"), "PArbitrationAuPsm");
        assert_eq!(camel("P_Arbitration_AU_PSM"), "pArbitrationAuPsm");
        assert_eq!(kebab("P_Arbitration_AU_PSM"), "p-arbitration-au-psm");
    }
}
