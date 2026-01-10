// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

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
                $(#[$const_meta:meta])*
                $const_name:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        $vis struct $name(pub $type);

        impl $name {
            $(
                $(#[$const_meta])*
                $vis const $const_name: Self = Self($value);
            )*

            #[inline]
            #[must_use]
            pub const fn nil() -> Self {
                Self(0)
            }

            #[inline]
            #[must_use]
            pub const fn all() -> Self {
                Self(0 $(| $value)*)
            }

            #[inline]
            #[must_use]
            pub const fn bits(&self) -> $type {
                self.0
            }

            #[inline]
            #[must_use]
            pub const fn is_empty(&self) -> bool {
                self.0 == 0
            }

            #[inline]
            #[must_use]
            pub const fn is_all(&self) -> bool {
                self.0 == Self::all().0
            }

            #[inline]
            #[must_use]
            pub const fn all_of(&self, rhs: Self) -> bool {
                (self.0 & rhs.0) == rhs.0
            }

            #[inline]
            #[must_use]
            pub const fn contains(&self, rhs: Self) -> bool {
                (self.0 & rhs.0) != 0
            }

            #[inline]
            #[must_use]
            pub const fn union(&self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }

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
            fn marshal<'a, S>(&self, archive: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: $crate::encode::Serializer<'a>,
            {
                self.0.marshal(archive)
            }
        }

        impl $crate::Unmarshal for $name {
            fn unmarshal_mut<'a, D>(&mut self, archive: D) -> ::std::result::Result<(), D::Error>
            where
                D: $crate::decode::Deserializer<'a>,
            {
                self.0 = <$type>::unmarshal(archive)?;
                Ok(())
            }
        }
    };
}
