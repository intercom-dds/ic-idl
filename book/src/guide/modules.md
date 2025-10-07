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
