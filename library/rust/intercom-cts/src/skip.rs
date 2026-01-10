// KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,
// contain information which is proprietary and confidential to KONGSBERG or its licensors.
// Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed
// with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,
// or disassemble the software, unless such acts are allowed under applicable mandatory law or
// explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,
// must include this legend. (C) 2025 KONGSBERG - All rights reserved

use std::marker::PhantomData;

use crate::encode::{
    ArraySerializer, EnumSerializer, StructSerializer, MapSerializer, SeqSerializer, UnionSerializer,
};
use crate::error::Error;
use crate::Marshal;

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

impl<Ok, Err: Error> StructSerializer for Skip<Ok, Err> {
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
