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
