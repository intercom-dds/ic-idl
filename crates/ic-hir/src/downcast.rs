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

use intercom_cts::decode::Deserializer;
use intercom_cts::encode::{FieldSerializer, Serializer};
use intercom_cts::error::Error as _;
use intercom_cts::{Marshal, Unmarshal};

#[derive(Marshal, Unmarshal)]
struct Optional {
    value: bool,
}

use super::Context;
use crate::GenericAnn;

pub struct Downcast<'a> {
    ctx: &'a Context,
    ann: &'a GenericAnn,
}

impl<'a> Serializer for Downcast<'a> {
    type Ok = ();

    type Error = intercom_cts::cdr::Error;
    type Struct = intercom_cts::infallible::Never<Self::Ok, Self::Error>;
    type Union = intercom_cts::infallible::Never<Self::Ok, Self::Error>;
    type Enum = intercom_cts::infallible::Never<Self::Ok, Self::Error>;
    type Sequence = intercom_cts::infallible::Never<Self::Ok, Self::Error>;
    type Array = intercom_cts::infallible::Never<Self::Ok, Self::Error>;
    type Map = intercom_cts::infallible::Never<Self::Ok, Self::Error>;

    fn encode_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom(
            "Unsupported type: serializer does not support serializing 'bool's",
        ))
    }
}

impl<'a> FieldSerializer for Downcast<'a> {
    type Ok = <Self as Serializer>::Ok;
    type Error = <Self as Serializer>::Error;

    fn encode_field<T>(&mut self, _: usize, key: &str, value: &T) -> Result<(), Self::Error>
    where
        T: Marshal,
    {
        let name = self.ctx.str(self.ann.ident.name);
        let _param = self
            .ann
            .fields
            .iter()
            .find(|v| name == self.ctx.str(v.ident.as_ref().unwrap().name))
            .unwrap();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
