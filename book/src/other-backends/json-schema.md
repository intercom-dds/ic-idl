# JSON Schema Backend

The JSON Schema backend generates [JSON Schema](https://json-schema.org/) definitions (draft 2019-09) from IDL files. This enables validation and documentation of JSON data structures.

## Quick Start

Generate JSON Schema from an IDL file:

```bash
ic-idl --json-schema-out schemas/ schema.idl
```

This creates one `.json` file per type definition containing a JSON Schema.

## What Gets Generated

The JSON Schema backend generates individual `.json` files for each type:

- **Structures** → JSON Schema with `object` type and `properties`
- **Enumerations** → JSON Schema with `enum` constraint
- **Unions** → JSON Schema with `oneOf` or `anyOf`

All generated schemas use JSON Schema draft 2019-09 specification.

## Type Mappings

| IDL Type | JSON Schema Type |
|----------|------------------|
| `boolean` | `"type": "boolean"` |
| `octet`, `short`, `long`, etc. | `"type": "integer"` |
| `float`, `double` | `"type": "number"` |
| `string` | `"type": "string"` |
| `sequence<T>` | `"type": "array"` with `items` |
| `map<K, V>` | `"type": "object"` with `additionalProperties` |
| struct | `"type": "object"` with `properties` |
| enum | `"enum": [...]` |

## Examples

### Struct to Object Schema

**IDL:**
```idl
struct Person {
    string name;
    long age;
    string email;
};
```

**Generated JSON Schema (Person.json):**
```json
{
  "$id": "file:///Person.json",
  "$schema": "https://json-schema.org/draft/2019-09/schema#",
  "properties": {
    "age": {
      "type": "integer"
    },
    "email": {
      "type": "string"
    },
    "name": {
      "type": "string"
    }
  },
  "required": [
    "name",
    "age",
    "email"
  ],
  "title": "Person"
}
```

### Enum to Enum Schema

**IDL:**
```idl
enum Status {
    Active,
    Inactive,
    Pending
};
```

**Generated JSON Schema (Status.json):**
```json
{
  "$id": "file:///Status.json",
  "$schema": "https://json-schema.org/draft/2019-09/schema#",
  "enum": [
    "Active",
    "Inactive",
    "Pending"
  ],
  "title": "Status"
}
```

### Collections

**IDL:**
```idl
struct Team {
    string name;
    sequence<string> members;
    map<string, long> scores;
};
```

**Generated JSON Schema (Team.json):**
```json
{
  "$id": "file:///Team.json",
  "$schema": "https://json-schema.org/draft/2019-09/schema#",
  "properties": {
    "name": {
      "type": "string"
    },
    "members": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "scores": {
      "type": "object",
      "additionalProperties": {
        "type": "integer"
      }
    }
  },
  "required": ["name", "members", "scores"],
  "title": "Team"
}
```

## Using Generated Schemas

### Validation (JavaScript/Node.js)

```javascript
const Ajv = require("ajv");
const ajv = new Ajv();

const schema = require("./Person.json");
const validate = ajv.compile(schema);

const data = {
  name: "Alice",
  age: 30,
  email: "alice@example.com"
};

const valid = validate(data);
if (!valid) {
  console.log(validate.errors);
}
```

### Validation (Python)

```python
import json
import jsonschema

with open("Person.json") as f:
    schema = json.load(f)

data = {
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com"
}

jsonschema.validate(instance=data, schema=schema)
```

### Documentation

JSON Schema can be used with documentation tools:

```bash
# Generate HTML documentation
npx @apidevtools/json-schema-ref-parser bundle Person.json > Person.dereferenced.json
```

## Features

### Schema References

Types that reference other types will include `$ref` links:

**IDL:**
```idl
struct Address {
    string street;
    string city;
};

struct Person {
    string name;
    Address address;
};
```

The `Person` schema will reference the `Address` schema.

### Required Fields

All struct fields are marked as required by default in the generated schema.

### Bounded Collections

Sequences with size bounds in IDL are translated to `minItems` and `maxItems` constraints:

**IDL:**
```idl
struct Data {
    sequence<long, 10> values;
};
```

**Generated Schema:**
```json
{
  "properties": {
    "values": {
      "type": "array",
      "items": {"type": "integer"},
      "maxItems": 10
    }
  }
}
```

## Use Cases

- **API Documentation**: Generate schemas for REST API request/response validation
- **Configuration Validation**: Validate JSON configuration files
- **Code Generation**: Use with tools like `quicktype` to generate types in other languages
- **OpenAPI Integration**: Include schemas in OpenAPI specifications

## Next Steps

- [Protocol Buffers](./protobuf.md) - Generate .proto files
- [IDL Output](./idl-output.md) - Normalized IDL generation
