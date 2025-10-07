# Build integration

The generated code is ordinary C++17. Two pieces are required at build time:

1. The generated headers/implementation files.
2. The CTS runtime headers and implementation from `library/cpp/defs`.

## CMake with idl_generate.cmake (Recommended)

The `idl_generate.cmake` module provides a CMake function that handles code
generation automatically.

### Basic usage

```cmake
# Include the cmake module
include(idl_generate)

idl_generate(
    LANGUAGE CPP
    DESTINATION ${CMAKE_BINARY_DIR}/generated/cpp
    INPUT_IDL schema.idl
)

# The function sets IC_GENERATE_OUTPUTS variable with generated files
add_library(demo-idl ${IC_GENERATE_OUTPUTS})

target_include_directories(demo-idl PUBLIC
    ${CMAKE_BINARY_DIR}/generated/cpp
    ${CMAKE_SOURCE_DIR}/library/cpp/defs
)
```

### Multiple IDL files

```cmake
idl_generate(
    LANGUAGE CPP
    DESTINATION ${CMAKE_BINARY_DIR}/generated/cpp
    INPUT_IDL
        types.idl
        services.idl
        models.idl
)

add_library(my-idl ${IC_GENERATE_OUTPUTS})
```

### Additional options

```cmake
idl_generate(
    LANGUAGE CPP
    DESTINATION generated/
    INPUT_IDL schema.idl
    INCLUDE_DIRECTORIES
        ${CMAKE_SOURCE_DIR}/common/idl
        /usr/share/idl
    FLAGS
        --scoped-enums
)
```

**Available parameters:**
- `LANGUAGE` - Target language: `CPP`, `PYTHON`, `RUST`, `IDL`, `PROTOBUF` (default: `CPP`)
- `DESTINATION` - Output directory (default: `CMAKE_CURRENT_BINARY_DIR`)
- `INPUT_IDL` - List of IDL files to process
- `INCLUDE_DIRECTORIES` - Include paths for `#include` directives
- `FLAGS` - Additional flags passed to `ic-idl`
- `OUTPUT_VAR` - Variable name for output files (default: `IC_GENERATE_OUTPUTS`)
- `OUTPUT_ACCUMULATED` - Append to this variable instead of overwriting

### Multi-language generation

Generate bindings for multiple languages:

```cmake
# Generate C++
idl_generate(
    LANGUAGE CPP
    DESTINATION ${CMAKE_BINARY_DIR}/generated/cpp
    INPUT_IDL schema.idl
)
add_library(demo-cpp ${IC_GENERATE_OUTPUTS})

# Generate Python
idl_generate(
    LANGUAGE PYTHON
    DESTINATION ${CMAKE_BINARY_DIR}/generated/python
    INPUT_IDL schema.idl
)
```
