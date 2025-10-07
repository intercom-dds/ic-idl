# C++ Backend

Generate modern C++ code with STL types.

## Generation

```bash
ic-idl schema.idl --cpp-out include/generated
```

## Type Mappings

| IDL Type | C++ Type |
|----------|----------|
| `boolean` | `bool` |
| `long` | `int32_t` |
| `string` | `std::string` |
| `sequence<T>` | `std::vector<T>` |
| `map<K,V>` | `std::map<K, V>` |

## Options

- `--scoped-enums` - Generate `enum class`
- `--use-fmt` - Generate fmt formatters
- `--dll-export` - Add DLL export macros

See [Code Generation Guide](../code-generation.md) for details.
