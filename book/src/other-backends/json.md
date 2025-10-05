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

# JSON Representation Backend

The JSON backend generates a JSON representation of the IDL type definitions. This is useful for:
- Tooling that needs to parse IDL structure
- Introspection and reflection
- Dynamic code generation
- IDE integration

## Quick Start

Generate JSON representation from an IDL file:

```bash
ic-idl --json-out json/ schema.idl
```

This creates a single `.json` file containing all type definitions in a structured format.

## What Gets Generated

The JSON backend generates a single `.json` file containing:

- All type definitions with their structure
- Type metadata (kind, members, etc.)
- Field names and types
- Enum variants

The output is a JSON object where keys are type names and values describe the type structure.

## Format

### Structures

Structs are represented with `"kind": "struct"` and a `members` array:

**IDL:**
```idl
struct Person {
    string name;
    long age;
    string email;
};
```

**JSON representation:**
```json
{
  "Person": {
    "kind": "struct",
    "members": [
      {
        "kind": "string",
        "name": "name"
      },
      {
        "kind": "int32",
        "name": "age"
      },
      {
        "kind": "string",
        "name": "email"
      }
    ]
  }
}
```

### Enumerations

Enums are represented with `"kind": "enum"` and an `enumerators` array:

**IDL:**
```idl
enum Status {
    Active,
    Inactive,
    Pending
};
```

**JSON representation:**
```json
{
  "Status": {
    "enumerators": [
      {
        "name": "Active"
      },
      {
        "name": "Inactive"
      },
      {
        "name": "Pending"
      }
    ],
    "kind": "enum"
  }
}
```

## Type Kinds

The JSON representation uses the following type kinds:

- `"string"` - String type
- `"int32"` - 32-bit signed integer (IDL `long`)
- `"int64"` - 64-bit signed integer (IDL `long long`)
- `"uint32"` - 32-bit unsigned integer
- `"uint64"` - 64-bit unsigned integer
- `"float"` - Single-precision float
- `"double"` - Double-precision float
- `"boolean"` - Boolean type
- `"struct"` - Structure type
- `"enum"` - Enumeration type

## Use Cases

### Code Generation Tools

Use the JSON output to build custom code generators:

```javascript
const fs = require('fs');

const schema = JSON.parse(fs.readFileSync('schema.json', 'utf8'));

for (const [typeName, typeDef] of Object.entries(schema)) {
  if (typeDef.kind === 'struct') {
    console.log(`Generating class ${typeName}...`);
    generateClass(typeName, typeDef.members);
  }
}
```

### Type Introspection

Query type information at runtime:

```python
import json

with open('schema.json') as f:
    schema = json.load(f)

# Find all enums
enums = {name: defn for name, defn in schema.items() 
         if defn['kind'] == 'enum'}

print(f"Found {len(enums)} enumerations")
```

### IDE Integration

Parse the JSON output to provide:
- Auto-completion
- Type hints
- Documentation lookup
- Syntax validation

## Complete Example

**IDL input (types.idl):**
```idl
enum Role {
    Admin,
    User,
    Guest
};

struct User {
    string name;
    long id;
    Role role;
};
```

**Generated JSON (types.json):**
```json
{
  "Role": {
    "enumerators": [
      {
        "name": "Admin"
      },
      {
        "name": "User"
      },
      {
        "name": "Guest"
      }
    ],
    "kind": "enum"
  },
  "User": {
    "kind": "struct",
    "members": [
      {
        "kind": "string",
        "name": "name"
      },
      {
        "kind": "int32",
        "name": "id"
      },
      {
        "kind": "Role",
        "name": "role"
      }
    ]
  }
}
```

Note how the `role` field references the `Role` type by name.

## Differences from JSON Schema

The JSON backend is **not** JSON Schema. Key differences:

| JSON Backend | JSON Schema Backend |
|--------------|---------------------|
| Describes IDL structure | Validates JSON data |
| Internal representation | Standard validation format |
| Tool/compiler input | Data validation |
| One file for all types | One file per type |

Use the JSON backend for tooling and introspection, and JSON Schema backend for validation.

## Next Steps

- [JSON Schema](./json-schema.md) - For JSON data validation
- [Protocol Buffers](./protobuf.md) - Generate .proto files
- [IDL Output](./idl-output.md) - Normalized IDL generation
