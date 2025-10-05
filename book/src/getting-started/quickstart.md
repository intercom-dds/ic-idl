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

# Quick Start

This short walkthrough compiles a single IDL file and inspects the generated
Rust, Python, and C++ code.

## 1. Write an IDL file

Create `hello.idl` with a struct, enum, and interface:

```idl
module demo {
    enum Status {
        Active,
        Inactive,
    };

    struct Person {
        string name;
        long age;
        Status status;
    };

    interface Directory {
        Person lookup(in string user_id);
    };
};
```

## 2. Run the compiler

From the repository root (or anywhere `ic-idl` is on your `PATH`):

```bash
cargo run -p ic-idl -- \
    --rust-out generated/rust \
    --python-out generated/python \
    --cpp-out generated/cpp \
    hello.idl
```

The output directory now contains one subdirectory per backend. Each backend
mirrors the module hierarchy from the IDL file.

```
generated/
├── cpp/
│   ├── demo/hello.h
│   └── demo/hello.cpp
├── python/
│   └── demo/hello.py
└── rust/
    ├── lib.rs
    └── demo/hello.rs
```

Use `--list` if you only want to preview the files without writing them, and
`--purge-dirs` to clear existing output directories before regenerating.

## 3. Use the Rust output

`generated/rust/lib.rs` re-exports the modules for convenient inclusion. In your
application crate:

```rust
mod generated;

use generated::demo::{Directory, Person, Status};

fn main() {
    // Public fields are initialised with Rust defaults by `Person::new()`.
    let mut person = Person::new();
    person.name = "Alice".to_string();
    person.age = 30;
    person.status = Status::Active;

    println!("{:?}", person);
}
```

Every struct/union/exception gains `new()` and `Default` implementations, plus
`Marshal`/`Unmarshal` so they can be serialised through the `intercom-cts`
runtime.

## 4. Use the Python output

The Python backend produces classes backed by the `intercom_dds` runtime. Each
field is exposed as a property with validation.

```python
from generated.python.demo.hello import Person, Status

person = Person(name="Alice", age=30, status=Status.Active)
print(person.status)
person.age = 31
```

The generated package contains `__init__.py` files so it can be imported as a
regular module tree. Install `intercom-dds` into your environment when working
with the Python bindings.

## 5. Use the C++ output

The C++ backend creates a header/implementation pair per source file. Include
`library/cpp/defs` when compiling so the generated code can reference the
serialization helpers.

```cpp
#include "generated/cpp/demo/hello.h"

int main() {
    demo::Person person;
    person.name = "Alice";
    person.age = 30;
    person.status = demo::Status::Active;
    return 0;
}
```

Compile with your usual build system, adding `library/cpp/defs` to the header
search path and linking against the generated `.cpp` file and the CTS runtime as
needed.

## Next steps

- Explore the [language reference](../guide/language-reference.md) for detailed
  syntax and annotation support.
- Visit the backend-specific guides for
  [Rust](../rust/overview.md), [Python](../python/overview.md), and
  [C++](../cpp/overview.md).
- Check the [CLI reference](../guide/cli.md) to discover additional flags.
