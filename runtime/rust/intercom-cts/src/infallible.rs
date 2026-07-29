// Copyright 2023 KONGSBERG
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

use std::convert::Infallible;
use std::marker::PhantomData;

use crate::decode::{
    ArrayDeserializer, BitmaskDeserializer, EnumDeserializer, EnumVisitor, MapDeserializer,
    OptionDeserializer, SeqDeserializer, StructDeserializer, UnionDeserializer,
};
use crate::encode::{
    ArraySerializer, BitmaskSerializer, EnumSerializer, MapSerializer, SeqSerializer,
    StructSerializer, UnionSerializer,
};
use crate::error::Error;
use crate::{Marshal, MemberInfo, Unmarshal};

pub struct Never<Ok, Err> {
    n: Infallible,
    _ok: PhantomData<fn() -> Ok>,
    _err: PhantomData<fn() -> Err>,
}

impl<Ok, Err: Error> StructSerializer<'_> for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_field<T>(&mut self, _: &MemberInfo<'_>, _: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        match self.n {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.n {}
    }
}

impl<Ok, Err: Error> UnionSerializer<'_> for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_discriminant<D>(&mut self, _: &D) -> Result<(), Self::Error>
    where
        D: Marshal,
    {
        match self.n {}
    }

    fn encode_variant<V>(self, _: &MemberInfo<'_>, _: &V) -> Result<Self::Ok, Self::Error>
    where
        V: Marshal,
    {
        match self.n {}
    }

    fn encode_null(self) -> Result<Self::Ok, Self::Error> {
        match self.n {}
    }
}

impl<'a, Ok, Err: Error> EnumSerializer<'a> for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_variant<T>(self, _: &MemberInfo<'a>, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
    {
        match self.n {}
    }
}

impl<'a, Ok, Err: Error> BitmaskSerializer<'a> for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_flag<T>(self, _: T, _: &[MemberInfo<'a>]) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal + Into<u64>,
    {
        match self.n {}
    }
}

impl<Ok, Err: Error> ArraySerializer for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_next<T>(&mut self, _: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        match self.n {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.n {}
    }
}

impl<Ok, Err: Error> SeqSerializer for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_next<T>(&mut self, _: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        match self.n {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.n {}
    }
}

impl<Ok, Err: Error> MapSerializer for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_pair<K, S>(&mut self, _: &K, _: &S) -> Result<(), Self::Error>
    where
        K: Marshal,
        S: Marshal,
    {
        match self.n {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.n {}
    }
}

impl<Ok, Err: Error> StructDeserializer<'_> for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn decode_field<T>(&mut self, _: &MemberInfo<'_>, _: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        match self.n {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.n {}
    }
}

impl<Ok, Err: Error> UnionDeserializer<'_> for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn decode_discriminant<T>(&mut self, _: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        match self.n {}
    }

    fn decode_variant<T>(self, _: &MemberInfo<'_>, _: &mut T) -> Result<Self::Ok, Self::Error>
    where
        T: Unmarshal,
    {
        match self.n {}
    }
}

impl<Ok, Err: Error> EnumDeserializer for Never<Ok, Err> {
    type Error = Err;

    fn decode_enumerator<T>(self, _: T) -> Result<T, Self::Error>
    where
        T: Unmarshal + EnumVisitor,
    {
        match self.n {}
    }
}

impl<Ok, Err: Error> MapDeserializer for Never<Ok, Err> {
    type Error = Err;

    fn decode_pair<K, V>(&mut self, _: &mut K, _: &mut V) -> Result<bool, Self::Error>
    where
        K: Unmarshal,
        V: Unmarshal,
    {
        match self.n {}
    }

    fn size_hint(&self) -> Option<usize> {
        match self.n {}
    }
}

impl<Ok, Err: Error> SeqDeserializer for Never<Ok, Err> {
    type Error = Err;

    fn decode_next<T>(&mut self, _: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        match self.n {}
    }

    fn size_hint(&self) -> Option<usize> {
        match self.n {}
    }
}

impl<Ok, Err: Error> ArrayDeserializer for Never<Ok, Err> {
    type Error = Err;

    fn decode_next<T>(&mut self, _: &mut T) -> Result<bool, Self::Error>
    where
        T: Unmarshal,
    {
        match self.n {}
    }
}

impl<Ok, Err: Error> OptionDeserializer for Never<Ok, Err> {
    type Error = Err;

    fn is_some(&mut self) -> bool {
        match self.n {}
    }

    fn decode_some<T>(self, _: &mut T) -> Result<(), Self::Error>
    where
        T: Unmarshal,
    {
        match self.n {}
    }
}

impl<'a, Ok, Err: Error> BitmaskDeserializer<'a> for Never<Ok, Err> {
    type Error = Err;

    fn decode_flags<T>(self, _: &[MemberInfo<'a>]) -> Result<T, Self::Error>
    where
        T: Unmarshal + Default,
    {
        match self.n {}
    }
}
