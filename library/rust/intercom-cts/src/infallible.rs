// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2023 KONGSBERG - All rights reserved

use std::convert::Infallible;
use std::marker::PhantomData;

use crate::decode::{
    ArrayDeserializer, EnumDeserializer, EnumVisitor, MapDeserializer, OptionDeserializer,
    SeqDeserializer, StructDeserializer, UnionDeserializer,
};
use crate::encode::{
    ArraySerializer, EnumSerializer, MapSerializer, SeqSerializer, StructSerializer,
    UnionSerializer,
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

impl<Ok, Err: Error> EnumSerializer for Never<Ok, Err> {
    type Ok = Ok;
    type Error = Err;

    fn encode_variant<T>(self, _: &str, _: T) -> Result<Self::Ok, Self::Error>
    where
        T: Marshal,
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
