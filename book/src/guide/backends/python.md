# Python Backend

Generate Python code with type hints and property validation.

## Generation

```bash
ic-idl schema.idl --python-out src/generated
```

## Type Mappings

| IDL Type | Python Type |
|----------|-------------|
| `boolean` | `bool` |
| All integers | `int` |
| `float`, `double` | `float` |
| `string` | `str` |
| `sequence<T>` | `list[T]` |
| `map<K, V>` | `dict[K, V]` |

## Options

- `--use-pep8` - Rename types to PEP-8 style
- `--global-postfix` - Add suffix to global modules

See [Code Generation Guide](../code-generation.md) for details.
