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
