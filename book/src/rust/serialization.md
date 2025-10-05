# Copyright 2025 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

# Serialization with intercom-cts

All generated Rust types implement `Marshal` and `Unmarshal` traits from the `intercom-cts` library, enabling binary serialization using CDR (Common Data Representation).

## Overview

`intercom-cts` (Intercom Common Type System) provides:
- CDR encoding/decoding (compatible with OMG CORBA and DDS)
- Support for multiple serialization formats (CDR, JSON, etc.)
- Automatic trait implementations via code generation

## Generated Traits

For each struct, enum, and union, IC-IDL generates implementations of:

```rust
pub trait Marshal {
    fn marshal<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

pub trait Unmarshal {
    fn unmarshal_mut<D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer;
}
```

## Basic Usage

The `intercom-cts` library provides serializers for different formats:

- **CDR1** - CDR version 1 (binary format, DDS standard)
- **JSON** - For debugging and interop

### Serializing to CDR1

```rust
use intercom_cts::cdr1;

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: "alice@example.com".to_string(),
};

// Serialize to little-endian CDR
let bytes = cdr1::to_le_bytes(&person)?;

// Or serialize to big-endian CDR
let bytes = cdr1::to_be_bytes(&person)?;

// Or use native endianness
let bytes = cdr1::to_bytes(&person)?;
```

### Deserializing from CDR1

```rust
use intercom_cts::cdr1;

// Deserialize from bytes (endianness auto-detected)
let mut person = Person::new();
cdr1::from_bytes_mut(&bytes, &mut person)?;

// Or create new instance and deserialize
let person = cdr1::from_bytes::<Person>(&bytes)?;

// Explicitly specify endianness
let person = cdr1::from_le_bytes::<Person>(&bytes)?;
let person = cdr1::from_be_bytes::<Person>(&bytes)?;
```

### Serializing to JSON

```rust
use intercom_cts::json;

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: "alice@example.com".to_string(),
};

// Serialize to JSON string
let json_string = json::to_string(&person)?;
println!("{}", json_string);
// Output: {"name":"Alice","age":30,"email":"alice@example.com"}

// Serialize to JSON bytes
let json_bytes = json::to_bytes(&person)?;
```

### Deserializing from JSON

```rust
use intercom_cts::json;

let json_str = r#"{"name":"Alice","age":30,"email":"alice@example.com"}"#;

// Deserialize into existing instance
let mut person = Person::new();
json::from_string_mut(json_str, &mut person)?;

// Or create new instance
let person = json::from_str::<Person>(json_str)?;
```

### Complete Example

```rust
use intercom_cts::{cdr1, json};

// Create a person
let person = Person {
    name: "Bob".to_string(),
    age: 25,
    email: "bob@example.com".to_string(),
};

// Serialize to CDR (binary)
let cdr_bytes = cdr1::to_bytes(&person)?;
println!("CDR size: {} bytes", cdr_bytes.len());

// Serialize to JSON (text)
let json_string = json::to_string(&person)?;
println!("JSON: {}", json_string);

// Deserialize from CDR
let person_from_cdr = cdr1::from_bytes::<Person>(&cdr_bytes)?;
assert_eq!(person.name, person_from_cdr.name);

// Deserialize from JSON
let person_from_json = json::from_str::<Person>(&json_string)?;
assert_eq!(person.age, person_from_json.age);
```

## Type Information

Generated code includes type metadata for introspection:

```rust
const TYPE_INFO: ::intercom_cts::TypeInfo<'static> = ::intercom_cts::TypeInfo {
    name: "Person",
    flags: ::intercom_cts::TypeFlag::IS_APPENDABLE,
    kind: ::intercom_cts::TypeKind::Struct,
    key_kind: ::intercom_cts::TypeKind::None,
    element_kind: ::intercom_cts::TypeKind::None,
};
```

This metadata is used by the serialization framework to handle versioning and compatibility.

## Interoperability

Data serialized with `intercom-cts` in Rust can be exchanged with:
- C++ code using CDR libraries (FastCDR, OpenDDS CDR)
- Python code with CDR support  
- Java DDS implementations
- Any system supporting OMG CDR encoding

The format is standardized, ensuring cross-language compatibility.

## Next Steps

- [Build Integration](./build-integration.md) - Using with Cargo

For detailed API documentation, see the `intercom-cts` crate documentation.
