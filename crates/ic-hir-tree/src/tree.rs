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

use std::fmt::{self, Display};

use ic_cli::color::Colorize as _;

#[macro_export]
macro_rules! leaf {
    ($($arg:tt)*) => {{
        $crate::tree::Leaf::from(format!($($arg)*))
    }}
}

pub struct Leaf<D: Display> {
    root: D,
    leaves: Vec<Leaf<D>>,
}

impl<D: Display> Leaf<D> {
    pub fn push(&mut self, leaf: impl Into<Leaf<D>>) {
        self.leaves.push(leaf.into());
    }
}

impl<I, D: Display> Extend<I> for Leaf<D>
where
    I: Into<Leaf<D>>,
{
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        self.leaves.extend(iter.into_iter().map(Into::into));
    }
}

impl<D: Display> From<D> for Leaf<D> {
    fn from(node: D) -> Self {
        Self {
            root: node,
            leaves: vec![],
        }
    }
}

impl<D: Display> Display for Leaf<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn print_leaf<T: Display>(
            f: &mut fmt::Formatter,
            leaf: &Leaf<T>,
            prefix: &str,
            last: bool,
        ) -> fmt::Result {
            let (indent, glyph) = if last {
                ("   ", "└─")
            } else {
                ("│  ", "├─")
            };
            writeln!(f, "{}{} {}", prefix.gray(), glyph.gray(), leaf.root)?;

            let len = leaf.leaves.len();
            for (index, c) in leaf.leaves.iter().enumerate() {
                let is_last = index == len - 1;
                print_leaf(f, c, &format!("{prefix}{indent}"), is_last)?;
            }
            Ok(())
        }

        writeln!(f, "{}", self.root)?;
        for (index, child) in self.leaves.iter().enumerate() {
            let is_last = index == self.leaves.len() - 1;
            print_leaf(f, child, "", is_last)?;
        }
        Ok(())
    }
}
