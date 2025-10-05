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

# Modules

Modules provide namespaces to organize and scope declarations.

## Basic Syntax

```idl
module company {
    struct Employee {
        long id;
        string name;
    };
};
```

## Nested Modules

Modules can be nested to create hierarchies:

```idl
module company {
    module hr {
        struct Employee {
            long employee_id;
            string name;
        };
    };

    module finance {
        struct Salary {
            double amount;
            string currency;
        };
    };
};
```

## Scoped Names

Reference types using scope resolution (`::`):

```idl
module app {
    struct User {
        company::hr::Employee employee;
        company::finance::Salary salary;
    };
};
```

## Reopening Modules

You can reopen modules to add more declarations:

```idl
module company {
    struct Employee {};
};

// Later in the same file or different file
module company {
    struct Department {};  // Added to company module
};
```

## Generated Code Mapping

### Rust
```rust
mod company {
    pub mod hr {
        pub struct Employee { /* ... */ }
    }
}

use company::hr::Employee;
```

### Python
```python
# company/hr/__init__.py
class Employee:
    pass

from company.hr import Employee
```

### C++
```cpp
namespace company {
    namespace hr {
        class Employee { /* ... */ };
    }
}

using company::hr::Employee;
```

## Best Practices

- Keep module hierarchies shallow (2-3 levels max)
- Use modules to separate concerns
- Name modules clearly and consistently
- Avoid circular dependencies

## Next Steps

- [Interfaces](./interfaces.md) - Define service contracts
- [Annotations](./annotations.md) - Customize code generation
