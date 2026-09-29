// Copyright 2026 KONGSBERG
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

intercom_cts::include_idl!("corpus");

#[macro_export]
macro_rules! assert_approx {
    ($a:expr, $b:expr, $e:expr) => {
        assert!(
            ($a - $b).abs() <= $e,
            "assertion failed: `(left approx == right)` \nleft: `{:?}`, \nright: `{:?}` \ndiff: \
             `{:?}` > max: `{:?}`",
            $a,
            $b,
            ($a - $b).abs(),
            $e
        );
    };
}

#[cfg(test)]
mod test_annotations;
#[cfg(test)]
mod test_any_type;
#[cfg(test)]
mod test_bitmasks;
#[cfg(test)]
mod test_bounded_types;
#[cfg(test)]
mod test_circular_types;
#[cfg(test)]
mod test_comparison;
#[cfg(test)]
mod test_constants;
#[cfg(test)]
mod test_deep_generics;
#[cfg(test)]
mod test_defaults;
#[cfg(test)]
mod test_enums;
#[cfg(test)]
mod test_exceptions;
#[cfg(test)]
mod test_interfaces;
#[cfg(test)]
mod test_multi_module;
#[cfg(test)]
mod test_nested_modules;
#[cfg(test)]
mod test_strings;
#[cfg(test)]
mod test_structs;
#[cfg(test)]
mod test_typedefs;
#[cfg(test)]
mod test_unions;
#[cfg(test)]
mod test_valuetypes;
