# Python Type Mappings

How IDL constructs translate to the generated Python API.

## Primitive types

| IDL type | Python type | Notes |
|----------|-------------|-------|
| `boolean` | `bool` |
| `char`, `wchar` | `str` (length 1) |
| `octet` | `int` |
| Integer types (`short`, `long`, …) | `int` | Values are coerced with `int()` and checked against the IDL range.
| Floating-point types | `float` |
| `string`, `wstring` | `str` |
| `any`, `null`, `void` | `None` |

## Structures and valuetypes

- Become classes derived from `intercom_dds.intercom_types.BaseStruct`.
- `__init__` accepts keyword arguments with optional values (default `None`).
- Missing values are initialised to defaults (empty string, zero, nested type
  instance, empty list/dict).
- Each field exposes a property with validation logic; setters raise `TypeError`
  or `ValueError` if the assigned value is incompatible.

## Enumerations

Enums inherit from `BaseEnum` and `enum.Enum`. The first enumerator is assigned
value `0`; later ones use `enum.auto()`. String conversion and comparison are
provided by the base class.

## Unions

Unions derive from `BaseUnion`. For each case the generator creates accessor
methods and validates the active member. Null/default cases are supported.

## Collections

- `sequence<T>` / `sequence<T, N>` → `typing.List[T]`
- `array<T, N>` → `typing.List[T]` with default values repeated `N` times
- `map<K, V>` → `typing.Dict[K, V]`

The generated property setters ensure elements are instances of the expected
Python type. Bounds (e.g. `sequence<T, 4>`) are enforced at runtime when the CTS
serialiser encodes the structure.

## Optional members

Fields annotated with `@Optional` remain the same Python type but are flagged in
`MemberInfo` so the serializer can omit them. Assign `None` to represent the
absence of a value.

## Interfaces and operations

Interfaces generate abstract base classes with one method per operation. You can
subclass them to provide concrete behaviour or use them as typing contracts in
application code.

## Constants and aliases

- Constants become module-level assignments; complex constants are created with
their generated type.
- Type aliases resolve to direct aliases in the module namespace.

See [Generated code](./generated-code.md) for full examples covering each case.
