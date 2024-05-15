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

use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};
use std::path::PathBuf;

use crate::color::Colorize;

#[derive(Clone, Debug)]
pub enum ConvertError {
    InvalidValue(String),
}

pub type Result<T> = std::result::Result<T, ConvertError>;

impl Error for ConvertError {}

impl Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ConvertError::InvalidValue(ref msg) = self;
        write!(f, "invalid value: {msg}")
    }
}

pub trait Convert: Sized {
    fn from_result(input: &[String]) -> Result<Self>;
}

pub fn convert_exit<T: Convert>(input: &[String]) -> T {
    match T::from_result(input) {
        Ok(v) => v,
        Err(e) => {
            let err = "error:".red();
            eprintln!("{err} {e}");
            std::process::exit(1);
        }
    }
}

impl Convert for char {
    fn from_result(input: &[String]) -> Result<Self> {
        Ok(input.last().unwrap().chars().next().unwrap())
    }
}

impl Convert for String {
    fn from_result(input: &[String]) -> Result<Self> {
        Ok(input.last().unwrap().clone())
    }
}

impl Convert for PathBuf {
    fn from_result(input: &[String]) -> Result<Self> {
        Ok(PathBuf::from(input.last().unwrap().clone()))
    }
}

impl<T: Convert> Convert for Option<T> {
    fn from_result(input: &[String]) -> Result<Self> {
        Ok(Some(Convert::from_result(input)?))
    }
}

impl<T: Convert> Convert for Vec<T> {
    fn from_result(input: &[String]) -> Result<Self> {
        let mut values = vec![];
        for v in input {
            values.push(Convert::from_result(&[v.to_string()])?);
        }
        Ok(values)
    }
}

impl<T, S> Convert for HashSet<T, S>
where
    T: Convert + Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_result(input: &[String]) -> Result<Self> {
        Ok(HashSet::from_iter(Vec::<T>::from_result(input)?))
    }
}

impl Convert for bool {
    fn from_result(input: &[String]) -> Result<Self> {
        let lower = input.last().unwrap().to_lowercase();
        match lower.as_str() {
            "true" | "yes" | "y" | "1" => Ok(true),
            "false" | "no" | "n" | "0" => Ok(false),
            _ => Err(ConvertError::InvalidValue(format!(
                "expected boolean, found '{lower}'"
            ))),
        }
    }
}

macro_rules! impl_primitive {
    ($($type:ty),+ $(,)*) => {
        $(impl Convert for $type {
            fn from_result(input: &[String]) -> Result<Self> {
                input
                    .last()
                    .unwrap()
                    .parse::<Self>()
                    .map_err(|e| ConvertError::InvalidValue(e.to_string()))
            }
        })*
    };
}

impl_primitive![u8, i8, u16, i16, u32, i32, u64, i64, i128, u128, isize, usize, f32, f64];
