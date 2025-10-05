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

# Annotations

Annotations customize code generation and add metadata to IDL definitions.

## Syntax

```idl
@AnnotationName
struct Example {};

@AnnotationName(arg1, arg2)
struct WithArgs {};

@AnnotationName(key=value)
struct WithKeyValue {};
```

## Common Annotations

### @Rename

Change the name in generated code:

```idl
@Rename("User")
struct user_record {
    string username;
};
// Generated as "User" in all languages
```

### @Optional

Make fields nullable/optional:

```idl
struct Person {
    string name;
    @Optional string middle_name;
    @Optional long age;
};
```

Generates:
- Serialisation runtimes mark the field as optional; the surface type remains
  unchanged across backends.
- C++: `std::optional<T>`

### @Range

Add validation for numeric fields:

```idl
struct Temperature {
    @Range(min=-273.15, max=1000.0)
    double celsius;
};
```

### @Key

Mark primary key fields (DDS):

```idl
struct User {
    @Key string user_id;
    string name;
};
```

### @Immutable

Make types read-only after construction:

```idl
@Immutable
struct Config {
    string api_key;
    long timeout;
};
```

### @Extensibility

Control version compatibility (DDS):

```idl
@Extensibility(FINAL)
struct FinalType {
    long field;
};

@Extensibility(APPENDABLE)
struct ExtendableType {
    long field;
    // Can add fields in future versions
};

@Extensibility(MUTABLE)
struct FlexibleType {
    long field;
    // Can reorder and modify fields
};
```

### @position

Specify bit positions in bitmasks:

```idl
@bit_bound(8)
bitmask Permissions {
    @position(0) READ,
    @position(1) WRITE,
    @position(2) EXECUTE
};
```

### @bit_bound

Specify size for bitmasks:

```idl
@bit_bound(16)
bitmask Flags {
    FLAG_A,
    FLAG_B
};
```

## Multiple Annotations

Combine annotations on a single declaration:

```idl
@Immutable
@Rename("DatabaseConfig")
struct db_config {
    string host;
    @Optional string password;
};
```

## Backend-Specific Behavior

Some annotations affect specific backends:

- `@Rename` - All backends
- `@Optional` - All backends
- `@Range` - Rust, Python (validation)
- `@Key` - DDS-based systems
- `@Extensibility` - DDS, Protocol Buffers

## Best Practices

- Use `@Rename` to match target language conventions
- Use `@Optional` for fields that may be absent
- Use `@Immutable` for configuration objects
- Use `@Range` to enforce invariants
- Document custom annotations

## Next Steps

- [Preprocessor](./preprocessor.md) - Macros and conditional compilation
- [Code Generation](./code-generation.md) - Using generated code
