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

use std::rc::Rc;

const INLINE_SIZE: usize = 30;

const _: () = assert!(std::mem::size_of::<InlineStr>() == 32);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InlineStr(Storage);

impl InlineStr {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Storage::Inline { len, .. } => *len == 0,
            Storage::Ref(v) => v.is_empty(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match &self.0 {
            Storage::Inline { len, .. } => usize::from(*len),
            Storage::Ref(v) => v.len(),
        }
    }
}

impl From<String> for InlineStr {
    // If the string is already heap allocated, we can just continue using that
    // instead of trying to inline it.
    fn from(value: String) -> Self {
        Self(Storage::Ref(Rc::from(value)))
    }
}

// We explicitly check the length before casting it to `u8`
#[allow(clippy::cast_possible_truncation)]
impl<'a> From<&'a str> for InlineStr {
    fn from(value: &'a str) -> Self {
        if value.len() <= INLINE_SIZE {
            Self(Storage::Inline {
                len: value.len() as u8,
                buffer: [0; INLINE_SIZE],
            })
        } else {
            Self(Storage::Ref(Rc::from(value)))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Storage {
    Inline { len: u8, buffer: [u8; INLINE_SIZE] },
    Ref(Rc<str>),
}
