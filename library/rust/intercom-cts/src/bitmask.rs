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

/// Helper macro for defining newtype bitmasks.
///
/// # Examples
///
/// ```
/// # use intercom_cts::bitmask;
/// bitmask! {
///     #[derive(Copy, Clone)]
///     MyBitmask: u32 {
///         FLAG_ONE = 1 << 1,
///         FLAG_TWO = 1 << 2,
///         FLAG_FOUR = 1 << 4,
///     }
/// }
/// ````
///
/// Given the invocation above, the following `struct` will be generated, while
/// also implementing [`Debug`], [`BitOr`], [`BitOrAssign`], [`BitXor`],
/// [`BitXorAssign`], [`BitAnd`], [`BitAndAssign`], and [`BitNot`]:
///
/// ```
/// #[derive(Copy, Clone)]
/// #[repr(transparent)]
/// struct MyBitmask(pub u32);
///
/// impl MyBitmask {
///     const FLAG_ONE: Self = Self(1 << 1);
///     const FLAG_TWO: Self = Self(1 << 2);
///     const FLAG_FOUR: Self = Self(1 << 4);
/// }
/// ```
///
/// [`Debug`]: std::fmt::Debug
/// [`BitOr`]: std::ops::BitOr
/// [`BitOrAssign`]: std::ops::BitOrAssign
/// [`BitXor`]: std::ops::BitXor
/// [`BitXorAssign`]: std::ops::BitXorAssign
/// [`BitAnd`]: std::ops::BitAnd
/// [`BitAndAssign`]: std::ops::BitAndAssign
/// [`BitNot`]: std::ops::Not
#[macro_export]
macro_rules! bitmask {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident: $type:ty {
            $(
                $(#[$bit_meta:meta])*
                $const_name:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        $vis struct $name(pub $type);

        impl $name {
            $(
                $(#[$bit_meta])*
                $vis const $const_name: Self = Self($value);
            )*

            /// Constructs a bitmask where all bits are zeroed out.
            #[inline]
            pub const fn nil() -> Self {
                Self(0)
            }

            /// Constructs a bitmask where all bit are set.
            #[inline]
            pub const fn all() -> Self {
                Self(0 $(| $value)*)
            }

            /// Returns the underlying bits.
            #[inline]
            pub const fn bits(&self) -> $type {
                self.0
            }

            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.0 == 0
            }

            /// Checks if all bits are set.
            #[inline]
            pub const fn is_all(&self) -> bool {
                self.0 == Self::all().0
            }

            /// Checks if all bits of the given bitmask are set.
            #[inline]
            pub const fn all_of(&self, rhs: Self) -> bool {
                (self.0 & rhs.0) == rhs.0
            }

            /// Checks if the there is an overlap between the two bitmasks.
            #[inline]
            pub const fn contains(&self, rhs: Self) -> bool {
                (self.0 & rhs.0) != 0
            }

            /// Clears the bitmask.
            #[inline]
            pub fn clear(&mut self) {
                self.0 = 0
            }
        }

        impl ::std::ops::BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl ::std::ops::BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl ::std::ops::BitXor for $name {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl ::std::ops::BitXorAssign for $name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0
            }
        }

        impl ::std::ops::BitAnd for $name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl ::std::ops::BitAndAssign for $name {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
        }

        impl ::std::ops::Not for $name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let mut fields = vec![];
                $(
                    if self.all_of(Self::$const_name) {
                        fields.push(stringify!($const_name));
                    }
                )*

                if fields.is_empty() {
                    fields.push("0");
                }
                write!(f, "{}({})", stringify!($name), fields.join(" | "))
            }
        }

        impl $crate::Marshal for $name {
            fn marshal<S>(&self, archive: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: $crate::encode::Serializer,
            {
                self.0.marshal(archive)
            }
        }

        impl $crate::Unmarshal for $name {
            fn unmarshal_mut<D>(&mut self, archive: D) -> ::std::result::Result<(), D::Error>
            where
                D: $crate::decode::Deserializer,
            {
                self.0 = <$type>::unmarshal(archive)?;
                Ok(())
            }
        }
    };
}
