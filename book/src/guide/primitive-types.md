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

# Primitive Types

IC-IDL provides a rich set of primitive types for representing basic values.

## Boolean Type

The `boolean` type represents true/false values.

```idl
struct Flags {
    boolean enabled;
    boolean verbose;
};
```

**Literals**: `true`, `false`

## Integer Types

IC-IDL provides signed and unsigned integers of various sizes.

### Standard Integer Types

| Type | Description | Size | Range |
|------|-------------|------|-------|
| `octet` | Unsigned 8-bit | 1 byte | 0 to 255 |
| `short` | Signed 16-bit | 2 bytes | -32,768 to 32,767 |
| `unsigned short` | Unsigned 16-bit | 2 bytes | 0 to 65,535 |
| `long` | Signed 32-bit | 4 bytes | -2³¹ to 2³¹-1 |
| `unsigned long` | Unsigned 32-bit | 4 bytes | 0 to 2³²-1 |
| `long long` | Signed 64-bit | 8 bytes | -2⁶³ to 2⁶³-1 |
| `unsigned long long` | Unsigned 64-bit | 8 bytes | 0 to 2⁶⁴-1 |

### DDS Extended Integer Types

Shorter, more familiar names from DDS:

| Type | Equivalent | Size |
|------|------------|------|
| `int8` | `signed octet` | 1 byte |
| `uint8` | `octet` | 1 byte |
| `int16` | `short` | 2 bytes |
| `uint16` | `unsigned short` | 2 bytes |
| `int32` | `long` | 4 bytes |
| `uint32` | `unsigned long` | 4 bytes |
| `int64` | `long long` | 8 bytes |
| `uint64` | `unsigned long long` | 8 bytes |

### Example

```idl
struct IntegerExample {
    octet byte_value;
    short temperature;
    unsigned short port;
    long user_id;
    unsigned long timestamp;
    long long big_number;

    // DDS style
    int32 count;
    uint64 file_size;
};
```

## Floating-Point Types

| Type | Description | Size | Precision |
|------|-------------|------|-----------|
| `float` | Single precision | 4 bytes | ~7 decimal digits |
| `double` | Double precision | 8 bytes | ~15 decimal digits |
| `long double` | Extended precision | 16 bytes | Platform-dependent |

### Example

```idl
struct FloatExample {
    float temperature;
    double latitude;
    double longitude;
    long double high_precision;
};
```

## Character Types

| Type | Description | Size |
|------|-------------|------|
| `char` | 8-bit character | 1 byte |
| `wchar` | Wide character (Unicode) | Platform-dependent |

### Example

```idl
struct CharExample {
    char initial;
    wchar unicode_char;
};
```

**Literals**:
```idl
const char NEWLINE = '\n';
const wchar EURO = L'€';
```

## String Types

Strings represent sequences of characters.

### Unbounded Strings

```idl
struct Message {
    string text;        // No size limit
    wstring unicode;    // Wide string (Unicode)
};
```

### Bounded Strings

Limit the maximum length:

```idl
struct User {
    string<255> username;      // Max 255 characters
    string<1024> description;  // Max 1024 characters
    wstring<100> display_name; // Max 100 wide chars
};
```

**Note**: The bound specifies maximum length, not including null terminator.

## Type Mappings to Target Languages

### Rust

| IDL Type | Rust Type |
|----------|-----------|
| `boolean` | `bool` |
| `octet`, `uint8` | `u8` |
| `short`, `int16` | `i16` |
| `unsigned short`, `uint16` | `u16` |
| `long`, `int32` | `i32` |
| `unsigned long`, `uint32` | `u32` |
| `long long`, `int64` | `i64` |
| `unsigned long long`, `uint64` | `u64` |
| `float` | `f32` |
| `double` | `f64` |
| `char` | `u8` or `char` |
| `string` | `String` |
| `string<N>` | `String` (with validation) |

### Python

| IDL Type | Python Type |
|----------|-------------|
| `boolean` | `bool` |
| All integer types | `int` |
| `float`, `double` | `float` |
| `char` | `int` (byte value) |
| `string`, `wstring` | `str` |

### C++

| IDL Type | C++ Type |
|----------|----------|
| `boolean` | `bool` |
| `octet`, `uint8` | `uint8_t` |
| `short`, `int16` | `int16_t` |
| `unsigned short`, `uint16` | `uint16_t` |
| `long`, `int32` | `int32_t` |
| `unsigned long`, `uint32` | `uint32_t` |
| `long long`, `int64` | `int64_t` |
| `unsigned long long`, `uint64` | `uint64_t` |
| `float` | `float` |
| `double` | `double` |
| `long double` | `long double` |
| `char` | `char` |
| `wchar` | `wchar_t` |
| `string` | `std::string` |
| `wstring` | `std::wstring` |

## Best Practices

### Choose Appropriate Integer Sizes

Use the smallest integer type that fits your data:

```idl
struct Config {
    octet flags;          // 0-255 is enough
    unsigned short port;  // Port numbers fit in 16 bits
    long user_id;         // Most systems use 32-bit IDs
    long long timestamp;  // Unix timestamps need 64 bits
};
```

### Use Bounded Strings for Validation

Prevent unbounded memory allocation:

```idl
struct User {
    string<50> username;   // Reasonable limit
    string<10000> bio;     // Larger limit for longer content
};
```

### Float vs Double

- Use `float` for values where precision isn't critical (graphics, approximations)
- Use `double` for scientific calculations, coordinates, financial data

```idl
struct Position {
    float x;  // Graphics coordinate - float is fine
    float y;
};

struct GeoLocation {
    double latitude;   // Need precision for GPS
    double longitude;
};
```

## Constants with Primitive Types

You can define constants of primitive types:

```idl
const boolean DEBUG = true;
const long MAX_USERS = 1000;
const double PI = 3.14159265359;
const string APP_NAME = "MyApp";

struct Config {
    long max_connections;  // Can use MAX_USERS as default
};
```

## Next Steps

- [Constructed Types](./constructed-types.md) - Learn about complex types like structs and enums
- [Declarations](./declarations.md) - Learn about constants and type aliases
