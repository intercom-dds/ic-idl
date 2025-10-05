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

# Protocol Buffers backend

The Protocol Buffers backend converts IDL into proto3 `.proto` files. It groups
strongly connected type graphs (mutually-referential types) into the same file
and mirrors the module hierarchy with protobuf packages.

## Quick start

```bash
ic-idl --proto-out proto/ schema.idl
```

`ic-idl` writes one file per component under `proto/`, inserting `package`
statements that reflect the IDL module path. Dependent components are linked via
`import` directives.

## Type mappings

| IDL type | Proto type |
|----------|------------|
| `boolean` | `bool` |
| `char`, `wchar`, `octet`, unsigned integers up to 32 bits | `uint32` |
| Signed integers up to 32 bits | `int32` |
| Signed/unsigned 64-bit integers | `int64` / `uint64` |
| Floating point (`float`, `double`, `long double`) | `float`, `double` |
| `string`, `wstring` | `string` |
| `sequence<T>` | `repeated T` |
| `map<K, V>` | `map<K, V>` |

IDL unions become `message` definitions with `oneof` fields.

## Example

**IDL**
```idl
module demo {
    struct Person {
        string name;
        long age;
    };
}
```

**Generated protobuf (`demo/Person.proto`)**
```protobuf
syntax = "proto3";
package demo;

message Person {
    string name = 1;
    int32 age = 2;
}
```

## Compiling the output

Use `protoc` or your preferred protobuf compiler:

```bash
protoc --cpp_out=out proto/**/*.proto
protoc --python_out=out proto/**/*.proto
```

## Notes and limitations

- Proto3 treats fields as optional by default. Bounded strings/sequences are
  documented in comments but not enforced by protobuf itself.
- Arrays are expressed as `repeated` fields; bounds need to be validated by
  consumers.
- Some IDL constructs (e.g. 128-bit floats) are approximated because protobuf
  lacks a native representation.

## Related topics

- [IDL output](./idl-output.md)
- [JSON Schema](./json-schema.md)
