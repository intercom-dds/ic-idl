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

use std::marker::PhantomData;

use crate::Marshal;
use crate::encode::{
    ArraySerializer, EnumSerializer, FieldSerializer, MapSerializer, SeqSerializer, UnionSerializer,
};
use crate::error::Error;

/// Gratuitously skips all types, always returning `Ok`.
pub struct Skip<Ok, Err> {
    ok: Ok,
    _err: PhantomData<Err>,
}

impl<Ok: Default, Err> Default for Skip<Ok, Err> {
    fn default() -> Self {
        Self::new(Ok::default())
    }
}

impl<Ok, Err> Skip<Ok, Err> {
    pub fn new(value: Ok) -> Self {
        Self {
            ok: value,
            _err: PhantomData,
        }
    }
}

impl<Ok, Err: Error> FieldSerializer for Skip<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_field<T>(&mut self, _: usize, _: &str, _: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.ok)
    }
}

impl<Ok, Err: Error> UnionSerializer for Skip<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_discriminant<D>(&mut self, _: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        Ok(())
    }

    fn encode_variant<V>(self, _: usize, _: &str, _: &V) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal,
    {
        Ok(self.ok)
    }

    fn encode_null(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.ok)
    }
}

impl<Ok, Err: Error> EnumSerializer for Skip<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_variant<T>(self, _: &str, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        Ok(self.ok)
    }
}

impl<Ok, Err: Error> ArraySerializer for Skip<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_next<T>(&mut self, _: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.ok)
    }
}

impl<Ok, Err: Error> SeqSerializer for Skip<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_next<T>(&mut self, _: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.ok)
    }
}

impl<Ok, Err: Error> MapSerializer for Skip<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_pair<K, S>(&mut self, _: &K, _: &S) -> Result<(), Self::Error>
    where
        K: Marshal,
        S: Marshal,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.ok)
    }
}
