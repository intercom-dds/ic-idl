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

use std::fmt;

use intercom_cts::json::{self, Object, Value};

use crate::Diag;

pub struct Position {
    pub line: u32,
    pub column: u32,
}

pub struct ResolvedLabel {
    pub file: String,
    pub start: Position,
    pub end: Position,
    pub message: String,
}

fn position(pos: &Position) -> Value {
    let mut obj = Object::new();
    obj.insert("line".to_owned(), Value::Number(pos.line.into()));
    obj.insert("column".to_owned(), Value::Number(pos.column.into()));
    Value::Object(obj)
}

fn label(resolved: &ResolvedLabel) -> Value {
    let mut obj = Object::new();
    obj.insert("file".to_owned(), Value::String(resolved.file.clone()));
    obj.insert("start".to_owned(), position(&resolved.start));
    obj.insert("end".to_owned(), position(&resolved.end));

    if !resolved.message.is_empty() {
        obj.insert(
            "message".to_owned(),
            Value::String(resolved.message.clone()),
        );
    }

    Value::Object(obj)
}

fn value(diag: &Diag, labels: &[ResolvedLabel]) -> Value {
    let mut obj = Object::new();
    obj.insert(
        "level".to_owned(),
        Value::String(diag.title.text.to_owned()),
    );
    obj.insert("message".to_owned(), Value::String(diag.msg.clone()));

    let optional = [
        ("code", &diag.code),
        ("help", &diag.help),
        ("note", &diag.note),
        ("description", &diag.desc),
    ];

    for (key, field) in optional {
        if let Some(text) = field {
            obj.insert(key.to_owned(), Value::String(text.clone()));
        }
    }

    if !labels.is_empty() {
        obj.insert(
            "labels".to_owned(),
            Value::Array(labels.iter().map(label).collect()),
        );
    }

    Value::Object(obj)
}

pub fn line(diag: &Diag, labels: &[ResolvedLabel]) -> Result<String, json::Error> {
    json::to_string(&value(diag, labels), false)
}

pub fn write(f: &mut dyn fmt::Write, diag: &Diag, labels: &[ResolvedLabel]) -> fmt::Result {
    let text = line(diag, labels).map_err(|_| fmt::Error)?;
    writeln!(f, "{text}")
}
