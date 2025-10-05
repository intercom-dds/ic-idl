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
