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

# C++ Backend Overview

The C++ backend emits header/implementation pairs targeting C++17. It mirrors
the IDL module hierarchy using nested namespaces and relies on the bundled CTS
runtime located in `library/cpp/defs`.

## Quick start

```bash
ic-idl schema.idl --cpp-out generated/cpp
```

```
generated/cpp/
└── demo/
    ├── schema.h
    └── schema.cpp
```

Compile the output together with the runtime headers:

```cmake
include_directories(${CMAKE_SOURCE_DIR}/library/cpp/defs)
add_library(demo STATIC
    generated/cpp/demo/schema.cpp
)
```

Include the headers from consumer code:

```cpp
#include "generated/cpp/demo/schema.h"

int main() {
    demo::Person person;
    person.name = "Alice";
    person.age = 30;
    return 0;
}
```

## Key features

- Structs, unions, valuetypes, and exceptions map to `struct` definitions with
  public members.
- Each source file produces a single header plus `.cpp` file. The header contains
  declarations, inline helpers, and metadata; the `.cpp` file stores the
  serialisation tables.
- Generated code depends on the CTS helpers (`ic_cts`) for type metadata,
  serialisation, and memory utilities. The headers already include the relevant
  runtime headers (`<ic_cts/member_info.h>`, etc.).
- Options such as `--scoped-enums`, `--no-stream-op`, `--use-fmt`, and
  `--header-ext` customise the style of the output.

## Build requirements

- C++17 compiler (`clang++`, `g++`, or MSVC 19.3x+)
- `library/cpp/defs` on the include path; link its object files or compile the
  inline `.ic` files depending on your build setup.

## Where to next?

- [Type mappings](./type-mappings.md)
- [Generated code tour](./generated-code.md)
- [Serialisation](./serialization.md)
- [Integrating with build systems](./build-integration.md)
