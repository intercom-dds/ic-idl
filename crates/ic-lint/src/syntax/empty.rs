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

use ic_diagnostic::Diag;
use ic_syntax::visit::Visitor;
use ic_syntax::Definition;

use crate::{Category, Lint};

/// Verifies that enums, unions and valuetypes have at least one member.
///
/// Support for empty structs is an extension defined in the `DDS-RPC`
/// standard, thus not covered by this lint.
pub struct EmptyTypes;

impl Lint for EmptyTypes {
    fn new() -> Box<dyn Lint>
    where
        Self: Sized,
    {
        Box::new(EmptyTypes)
    }

    fn category(&self) -> crate::Category {
        Category::Pedantic
    }

    fn check(self: Box<Self>, ast: &[Definition]) -> Vec<Diag> {
        vec![]
    }
}

/// Verifies that enums, unions and valuetypes have at least one member, and
/// all modules have at least one definition.
///
/// Support for empty structs and exceptions is an extension defined in the
/// `DDS-RPC` standard, thus not covered by this lint.
pub fn empty_types() -> Box<dyn Lint> {
    Box::new(EmptyTypes)
}
