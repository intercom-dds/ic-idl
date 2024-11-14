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

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Deref;

use crate::decode::Deserializer;
use crate::encode::{Marshal, Serializer};
use crate::error::Error;
use crate::{Unmarshal, WString};

const OUT_OF_RANGE: &str = "value outside of valid range";

// While this could be expressed more generically through other means like
// `ExactSizeIterator`, we generally don't want to augment the serialization of
// the underlying container.
trait Container {
    fn exact_len(&self) -> usize;
}

impl<T> Container for Vec<T> {
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl<T> Container for BTreeSet<T>
where
    T: Marshal,
{
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl<K, V> Container for BTreeMap<K, V>
where
    K: Marshal,
    V: Marshal,
{
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl<T> Container for HashSet<T>
where
    T: Marshal,
{
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl<K, V> Container for HashMap<K, V>
where
    K: Marshal,
    V: Marshal,
{
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl Container for String {
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl Container for &str {
    fn exact_len(&self) -> usize {
        self.len()
    }
}

impl<T: Container> Container for Option<T> {
    fn exact_len(&self) -> usize {
        self.as_ref().map_or(0, Container::exact_len)
    }
}

impl<T: Container> Container for Box<T> {
    fn exact_len(&self) -> usize {
        self.deref().exact_len()
    }
}

impl<T> Container for WString<T>
where
    T: AsRef<str>,
{
    fn exact_len(&self) -> usize {
        self.0.as_ref().len()
    }
}

#[inline(never)]
fn bound_error<T, const N: usize, E: Error>() -> E {
    Error::custom(format!(
        "length of container '{}' has exceeded its bound of {N}",
        std::any::type_name::<T>(),
    ))
}

pub struct Bound<T, const N: usize>(pub T);

impl<T, const N: usize> Marshal for Bound<&T, N>
where
    T: Container + Marshal,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if Container::exact_len(self.0) <= N {
            self.0.marshal(archive)
        } else {
            Err(bound_error::<T, N, _>())
        }
    }
}

impl<T, const N: usize> Unmarshal for Bound<&mut T, N>
where
    T: Container + Unmarshal,
{
    #[inline]
    fn unmarshal_mut<D>(&mut self, archive: D) -> Result<(), D::Error>
    where
        D: Deserializer,
    {
        self.0.unmarshal_mut(archive)?;
        if Container::exact_len(self.0) <= N {
            Ok(())
        } else {
            Err(bound_error::<T, N, _>())
        }
    }
}

pub struct Min<T, N>(pub T, pub N);

impl<T, N> Marshal for Min<&T, N>
where
    T: Marshal + PartialOrd<N>,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *self.0 >= self.1 {
            self.0.marshal(archive)
        } else {
            Err(Error::custom(OUT_OF_RANGE))
        }
    }
}

impl<T, N> Unmarshal for Min<&mut T, N>
where
    T: Unmarshal + PartialOrd<N>,
{
    #[inline]
    fn unmarshal_mut<S>(&mut self, archive: S) -> Result<(), S::Error>
    where
        S: Deserializer,
    {
        self.0.unmarshal_mut(archive)?;
        if *self.0 >= self.1 {
            Ok(())
        } else {
            Err(Error::custom(OUT_OF_RANGE))
        }
    }
}

pub struct Max<T, N>(pub T, pub N);

impl<T, N> Marshal for Max<&T, N>
where
    T: Marshal + PartialOrd<N>,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *self.0 <= self.1 {
            self.0.marshal(archive)
        } else {
            Err(Error::custom(OUT_OF_RANGE))
        }
    }
}

impl<T, N> Unmarshal for Max<&mut T, N>
where
    T: Unmarshal + PartialOrd<N>,
{
    #[inline]
    fn unmarshal_mut<S>(&mut self, archive: S) -> Result<(), S::Error>
    where
        S: Deserializer,
    {
        self.0.unmarshal_mut(archive)?;
        if *self.0 <= self.1 {
            Ok(())
        } else {
            Err(Error::custom(OUT_OF_RANGE))
        }
    }
}

pub struct Range<T, N, M> {
    pub value: T,
    pub min: N,
    pub max: M,
}

impl<T, N, M> Marshal for Range<&T, N, M>
where
    T: Marshal + PartialOrd<N> + PartialOrd<M>,
{
    #[inline]
    fn marshal<S>(&self, archive: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *self.value >= self.min && *self.value <= self.max {
            self.value.marshal(archive)
        } else {
            Err(Error::custom(OUT_OF_RANGE))
        }
    }
}

impl<T, N, M> Unmarshal for Range<&mut T, N, M>
where
    T: Unmarshal + PartialOrd<N> + PartialOrd<M>,
{
    #[inline]
    fn unmarshal_mut<S>(&mut self, archive: S) -> Result<(), S::Error>
    where
        S: Deserializer,
    {
        self.value.unmarshal_mut(archive)?;
        if *self.value >= self.min && *self.value <= self.max {
            Ok(())
        } else {
            Err(Error::custom(OUT_OF_RANGE))
        }
    }
}
