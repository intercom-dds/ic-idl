# Preprocessor

IC-IDL includes a C-style preprocessor for includes, macros, and conditional compilation.

## Include Files

### Syntax

```idl
#include "local_file.idl"
#include <system/file.idl>
```

- `"file.idl"` - Search current directory first, then include path
- `<file.idl>` - Search include path only

### Include Path

Specify include directories with `-I`:

```bash
ic-idl -I /usr/include/idl -I ./common schema.idl
```

### Example

**types.idl:**
```idl
struct Point {
    double x;
    double y;
};
```

**main.idl:**
```idl
#include "types.idl"

struct Line {
    Point start;
    Point end;
};
```

## Macros

### Object-Like Macros

```idl
#define MAX_SIZE 1024
#define VERSION "1.0.0"

struct Buffer {
    octet data[MAX_SIZE];
};
```

### Function-Like Macros

```idl
#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define MAX(a, b) ((a) > (b) ? (a) : (b))

const long LIMIT = MIN(100, 200);
```

### Undefine Macros

```idl
#define TEMP 42
// ... use TEMP ...
#undef TEMP
```

## Conditional Compilation

### #ifdef / #ifndef

```idl
#define FEATURE_ADVANCED

#ifdef FEATURE_ADVANCED
    struct AdvancedOptions {
        long option1;
        long option2;
    };
#endif

#ifndef FEATURE_BASIC
    const long DEFAULT_MODE = 1;
#endif
```

### #if / #elif / #else

```idl
#define VERSION 3

#if VERSION >= 3
    struct V3Feature {};
#elif VERSION == 2
    struct V2Feature {};
#else
    struct V1Feature {};
#endif
```

### Defined Operator

```idl
#if defined(DEBUG) && defined(VERBOSE)
    const boolean LOGGING = true;
#endif
```

## Predefined Macros

- `__FILE__` - Current file name
- `__LINE__` - Current line number
- `__DATE__` - Compilation date
- `__TIME__` - Compilation time

```idl
#pragma message("Compiling " __FILE__)
```

## Pragma Directives

### #pragma message

```idl
#pragma message("Warning: Deprecated feature")
```

### #pragma once

Prevent multiple inclusion:

```idl
#pragma once

struct Example {};
```

Equivalent to include guards:

```idl
#ifndef EXAMPLE_IDL
#define EXAMPLE_IDL

struct Example {};

#endif
```

## Command-Line Defines

Define macros from command line:

```bash
ic-idl -D DEBUG -D VERSION=2 schema.idl
```

Equivalent to:

```idl
#define DEBUG
#define VERSION 2
```

## Practical Examples

### Feature Flags

```idl
#ifdef FEATURE_ENCRYPTION
    struct EncryptedData {
        octet cipher[256];
        string algorithm;
    };
#endif
```

### Version-Specific Definitions

```idl
#if VERSION >= 2
    struct UserV2 {
        string id;
        string email;
        string phone;  // Added in V2
    };
#else
    struct User {
        string id;
        string email;
    };
#endif
```

### Platform-Specific Code

```idl
#ifdef PLATFORM_EMBEDDED
    typedef short ResourceId;  // 16-bit on embedded
#else
    typedef long ResourceId;   // 32-bit elsewhere
#endif
```

## Best Practices

- Use `#pragma once` or include guards
- Keep conditional compilation simple
- Document feature flags
- Avoid complex macro expressions
- Use constants instead of macros when possible

## Next Steps

- [Code Generation](./code-generation.md) - Using the code generators
- [CLI Reference](./cli.md) - Command-line options
