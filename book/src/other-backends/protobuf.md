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
