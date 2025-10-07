# C++ Type Mappings

How IDL constructs appear in the generated C++ output.

## Primitive types

| IDL type | C++ type |
|----------|----------|
| `boolean` | `bool` |
| `char` | `char` |
| `wchar` | `char16_t` |
| `octet` | `uint8_t` |
| `short` / `long` / `long long` | `int16_t` / `int32_t` / `int64_t` |
| Unsigned counterparts | `uint16_t` / `uint32_t` / `uint64_t` |
| DDS extended ints (`int8`, `uint8`, …) | fixed-width `<cstdint>` aliases |
| `float` / `double` / `long double` | `float` / `double` / `long double` |
| `string` | `::std::string` |
| `wstring` | `::std::wstring` |
| `any`, `null`, `void`, `fixed` | `void` |

## Structures and valuetypes

Structures map to `struct` with public members:

```cpp
struct Person {
    ::std::string name;
    int32_t age;
};
```

Constructors, copy/move assignment operators, and comparison operators are
provided. Valuetypes follow the same structure but may include helper methods
for operations and attributes.

## Enumerations

Enums default to unscoped `enum` declarations. Pass `--scoped-enums` to request
`enum class`. The generator also emits stream operators (unless disabled with
`--no-stream-op`) and `{fmt}` formatters when `--use-fmt` is supplied.

## Unions

Unions become `struct` wrappers containing a discriminator, an anonymous union
for the payload, constructors, and strongly-typed accessors:

```cpp
struct Value {
    Value();
    int32_t _d() const;
    void _d(int32_t);

    int32_t& int_value();
    void int_value(int32_t value);
    // ... other variants ...

private:
    union {
        int32_t int_value;
        ::std::string string_value;
    } ic_union_value_;
    int32_t discriminator_;
};
```

## Collections

- `sequence<T>` / `sequence<T, N>` → `::std::vector<T>`
- `array<T, N>` → `::std::array<T, N>`
- `map<K, V>` → `::std::map<K, V>`

Bounds are validated by the CTS serializer rather than the container type.

## Optional members

The `@Optional` annotation sets the appropriate flags in the metadata tables so
serialisation can elide the member. The C++ field type remains unchanged;
assign a sentinel (e.g. `nullptr` or an empty container) to express absence.

## Constants and aliases

- Constants become `constexpr` values or inline variables if initialisation
  requires constructors.
- Type aliases resolve to `using` declarations. When an alias targets an
  interface or valuetype that carries member functions, the alias forwards via
  `using` to preserve those methods.

## Interfaces

Interfaces are emitted as abstract classes with virtual methods that accept/return
IDL-mapped types. Generated stubs rely on the CTS runtime to marshal arguments.

## Metadata

Each type declares `ic_cts::TypeInfo` and `ic_cts::MemberInfo` tables in the `.cpp`
file. These power serialisation, hashing, and DDS/XTypes compatibility.

See [Generated code](./generated-code.md) for annotated examples.
