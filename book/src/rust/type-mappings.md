# Rust Type Mappings

This page enumerates how IDL constructs map to Rust when using the `--rust-out`
backend. Bounds and additional semantics are enforced by the `intercom-cts`
runtime unless explicitly noted.

## Primitive types

| IDL type | Rust type | Notes |
|----------|-----------|-------|
| `boolean` | `bool` | |
| `char` / `wchar` | `char` | Stored as Unicode scalar values. |
| `octet` | `u8` | |
| `short` / `long` / `long long` | `i16` / `i32` / `i64` | |
| `unsigned short` / `unsigned long` / `unsigned long long` | `u16` / `u32` / `u64` | |
| `int8` / `int16` / `int32` / `int64` | `i8` / `i16` / `i32` / `i64` | DDS extensions. |
| `uint8` / `uint16` / `uint32` / `uint64` | `u8` / `u16` / `u32` / `u64` | DDS extensions. |
| `float` / `double` / `long double` | `f32` / `f64` / `f64` | 128-bit floats map to `f64`. |
| `string` / `wstring` | `::std::string::String` | Bounds (e.g. `string<16>`) are checked during (de)serialisation. |
| `any` / `fixed` / `null` | `()` | Placeholder types; primarily for annotations or reserved constructs. |

## Constructed types

### Structures

IDL structures become `pub struct` definitions with `pub` fields:

```idl
struct Point {
    double x;
    double y;
};
```

```rust
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

Inheritance is flattened: derived structs include all base members.

### Enumerations

Enumerations map to `enum` with explicit discriminants and helper impls:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum Status {
    Active,
    Inactive,
}
```

`FromStr`, `Display`, and `Default` implementations are generated automatically.

### Bitmasks and bitsets

Bitmasks use the `intercom_cts::bitmask!` macro which creates a thin wrapper
around the underlying integer along with flag helpers. Bitsets become structs
containing named flag fields.

### Unions

IDL unions map to Rust enums with one variant per case. The discriminant and the
payload are stored together, so idiomatic pattern matching is available.

### Sequences and arrays

- `sequence<T>` / `sequence<T, N>` → `Vec<T>` (bounds validated at runtime).
- `T[N]` → `[T; N]`.

### Maps

`map<K, V>` becomes `::std::collections::BTreeMap<K, V>` to guarantee a stable
iteration order and trait support (`Ord`, `Hash`, …) across the generated APIs.

### Type aliases

Aliases turn into `pub type` for simple cases. If an alias targets an interface
(or another definition that requires inherent impls) it is re-exported via
`pub use` instead so trait implementations stay attached to the original type.

## Optional members

The `@Optional` annotation marks a field as optional in the type metadata.
Generated structs keep the original Rust type (for example `i32`), but the
metadata flag allows the CTS runtime to encode the absence of a value. If you
need `Option<T>` semantics on the Rust side, wrap the field in a newtype and add
conversion helpers in user code.

## Constants

Trivial constants become `pub const`. Non-trivial values (e.g. large arrays) are
wrapped in `::std::sync::LazyLock` so the initialisation is deferred until first
use.

## Interfaces

Interfaces are rendered as traits. Each operation appears as a trait method with
`Result`-like return types when the IDL declares `raises(...)`. Attributes map to
methods that follow Rust naming conventions (`get_foo`, `set_foo`).

## Metadata and serialization

Every type ships with `intercom_cts::TypeInfo` and `MemberInfo` definitions.
These drive the automatically generated `Marshal`/`Unmarshal` implementations,
enable runtime reflection, and power the end-to-end tests that verify
cross-language compatibility.

Refer to [Generated code](./generated-code.md) for full examples covering all
constructs.
