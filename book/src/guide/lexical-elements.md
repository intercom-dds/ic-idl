# Lexical Elements

This page describes the basic lexical elements of the IC-IDL language: comments, identifiers, and keywords.

## Comments

IC-IDL supports C and C++ style comments:

### Single-Line Comments

```idl
// This is a single-line comment
struct Example {};  // Comment after code
```

### Multi-Line Comments

```idl
/* This is a multi-line
   comment spanning
   several lines */
```

### Documentation Comments

Documentation comments use three slashes and apply to the following declaration:

```idl
/// This struct represents a user in the system.
/// It contains basic user information.
struct User {
    /// The user's unique identifier
    long id;

    /// The user's display name
    string name;
};
```

Documentation comments are preserved in generated code and can be used by documentation generators.

## Identifiers

Identifiers name types, variables, modules, and other program elements.

### Rules

- Must start with a letter (`a-z`, `A-Z`) or underscore (`_`)
- Can contain letters, digits (`0-9`), and underscores
- Case-sensitive
- Cannot be keywords

### Examples

Valid identifiers:
```idl
ValidIdentifier
_private
snake_case_name
CamelCaseName
name123
HTTP2Protocol
```

Invalid identifiers:
```idl
2invalid      // Cannot start with digit
my-name       // Hyphens not allowed
struct        // Reserved keyword
```

### Naming Conventions

While IC-IDL doesn't enforce naming conventions, we recommend:

- **Types** (structs, enums, interfaces): `PascalCase`
- **Fields and operations**: `snake_case`
- **Constants**: `UPPER_SNAKE_CASE`
- **Modules**: `lowercase` or `snake_case`

Example:
```idl
const long MAX_SIZE = 1024;

module user_management {
    enum UserStatus {
        Active,
        Inactive
    };

    struct UserProfile {
        string user_name;
        UserStatus status;
    };
};
```

## Keywords

The following words are reserved and cannot be used as identifiers:

```
abstract        any             alias           attribute
bitfield        bitmask         bitset          boolean
case            char            component       connector
const           consumes        context         custom
default         double          exception       emits
enum            eventtype       factory         false
finder          fixed           float           getraises
home            import          in              inout
interface       local           long            manages
map             mirrorport      module          multiple
native          object          octet           oneway
out             primarykey      private         port
porttype        provides        public          publishes
raises          readonly        setraises       sequence
short           string          struct          supports
switch          true            truncatable     typedef
typeid          typename        typeprefix      unsigned
union           uses            valuebase       valuetype
void            wchar           wstring
```

### Extended Keywords (DDS)

Additional keywords from DDS extensions:

```
int8            uint8           int16           int32
int64           uint16          uint32          uint64
```

## Scoped Names

Identifiers can be scoped using the `::` operator:

```idl
module company {
    module hr {
        struct Employee {};
    };
};

// Reference from outside
company::hr::Employee emp;
```

Leading `::` refers to the global scope:

```idl
::company::hr::Employee emp;  // Absolute reference
```

## Character Set

IC-IDL source files use UTF-8 encoding. Identifiers can use ASCII characters. String literals can contain Unicode characters:

```idl
struct Message {
    string text;  // Can store Unicode: "Hello, 世界"
};
```

## Whitespace

Whitespace (spaces, tabs, newlines) is generally not significant except for separating tokens:

```idl
struct Person{string name;long age;};  // Valid but not recommended

// Preferred formatting
struct Person {
    string name;
    long age;
};
```

## Literals

### Integer Literals

```idl
42           // Decimal
0x2A         // Hexadecimal
052          // Octal
```

### Floating-Point Literals

```idl
3.14
1.0e-10
2.5E+3
```

### Boolean Literals

```idl
true
false
```

### Character Literals

```idl
'a'
'\n'    // Newline
'\t'    // Tab
'\\'    // Backslash
'\''    // Single quote
```

### String Literals

```idl
"Hello, world!"
"Line 1\nLine 2"
"Unicode: \u0041"    // 'A'
```

### Escape Sequences

Supported escape sequences in character and string literals:

| Escape | Meaning |
|--------|---------|
| `\n` | Newline |
| `\t` | Tab |
| `\r` | Carriage return |
| `\\` | Backslash |
| `\'` | Single quote |
| `\"` | Double quote |
| `\0` | Null character |
| `\xHH` | Hexadecimal byte |
| `\uHHHH` | Unicode character (4 hex digits) |

## Semicolons

Most declarations end with a semicolon:

```idl
struct Point {
    double x;
    double y;
};  // Semicolon after closing brace

const long MAX = 100;  // Semicolon after constant
```

## Next Steps

Now that you understand the lexical elements, continue to:
- [Primitive Types](./primitive-types.md) - Learn about built-in types
- [Constructed Types](./constructed-types.md) - Learn about complex types
