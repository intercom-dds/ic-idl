# IDL Output Backend

The IDL output backend generates normalized IDL files. This is useful for:
- Formatting and standardizing existing IDL files
- Merging multiple IDL files into a single output
- Removing preprocessor directives and comments
- Validating IDL syntax

## Quick Start

Generate normalized IDL from an IDL file:

```bash
ic-idl --idl-out normalized/ schema.idl
```

This creates a normalized `.idl` file with consistent formatting.

## What Gets Generated

The IDL backend generates a single normalized `.idl` file containing:

- All type definitions from the input file(s)
- Standardized formatting and indentation
- Resolved includes (if preprocessing is enabled)
- Cleaned syntax without preprocessor directives

## Features

### Normalization

The IDL output backend normalizes the following:

- **Formatting**: Consistent indentation and spacing
- **Type names**: Standardized type references
- **Syntax**: Clean, canonical IDL syntax
- **Pragmas**: Preserved as `#pragma` directives

### Example

**Input IDL (with variations in formatting):**
```idl
struct Person{
  string name;
     long age;
string email;
};
enum Status {Active,Inactive,
    Pending};
```

**Generated normalized IDL:**
```idl
#pragma once

struct Person {
    string name;
    int32 age;
    string email;
};
enum Status {
    Active,
    Inactive,
    Pending
};
```

## Use Cases

### Code Formatting

Use the IDL output backend to format existing IDL files:

```bash
ic-idl --idl-out . my_schema.idl
```

### Merging Multiple Files

Combine multiple IDL files into a single normalized file:

```bash
ic-idl --idl-out merged/ types.idl services.idl interfaces.idl
```

### Validating Syntax

Generate normalized output to validate IDL syntax is correct:

```bash
ic-idl --idl-out /tmp/validate schema.idl
```

If the command succeeds, the IDL syntax is valid.

## Options

- `--idl-out <dir>` - Output directory for normalized IDL files

## Limitations

- Comments in the original file are not preserved
- Preprocessor macros are expanded (if preprocessing is enabled)
- The normalized format may differ slightly from the input format

## Practical Examples

### Clean Up Legacy IDL

```bash
ic-idl --idl-out cleaned/ legacy.idl
```

### Validate Before Code Generation

```bash
# First normalize to check syntax
ic-idl --idl-out /tmp/check schema.idl

# Then generate code
ic-idl --rust-out src/generated schema.idl
```

## Next Steps

- [Protocol Buffers](./protobuf.md) - Generate .proto files
- [JSON Schema](./json-schema.md) - JSON Schema generation
