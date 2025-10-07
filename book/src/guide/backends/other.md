# Other Backends

IC-IDL supports additional output formats.

## Protocol Buffers

Generate `.proto` files:

```bash
ic-idl schema.idl --proto-out proto/
protoc --cpp_out=. proto/schema.proto
```

## JSON Schema

Generate JSON Schema (draft-07):

```bash
ic-idl schema.idl --json-schema-out schemas/
```

## IDL Output

Normalize and reformat IDL:

```bash
ic-idl schema.idl --idl-out clean/
```

Options:
- `--idl-doxygen` - Doxygen-compatible output
- `--idl-legacy` - Compatible with older parsers

See [Code Generation Guide](../code-generation.md) for more details.
