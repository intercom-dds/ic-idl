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

# Interfaces

Interfaces define service contracts with operations and attributes.

## Basic Syntax

```idl
interface UserService {
    User get_user(in string user_id);
    void create_user(in User user);
    void delete_user(in string user_id);
};
```

## Operations

Operations are methods/functions defined in an interface.

### Parameters

Parameters have a direction specifier:

- `in` - Input parameter (passed to operation)
- `out` - Output parameter (returned from operation)
- `inout` - Input/output parameter (modified by operation)

```idl
interface Calculator {
    // Input parameters only
    long add(in long a, in long b);

    // Output parameters
    void divide(in double num, in double denom,
                out double result, out double remainder);

    // Input/output parameter
    void swap(inout long a, inout long b);
};
```

### Return Types

Operations can return values or be `void`:

```idl
interface DataService {
    long count();                    // Returns long
    User find_user(in string id);    // Returns User
    void log_event(in string msg);   // No return value
};
```

### Exceptions

Operations can declare exceptions they may raise:

```idl
exception NotFound {
    string message;
};

exception ValidationError {
    string field_name;
};

interface UserService {
    User get_user(in string id)
        raises (NotFound);

    void create_user(in User user)
        raises (ValidationError);

    void update_user(in string id, in User user)
        raises (NotFound, ValidationError);
};
```

## Attributes

Attributes are properties of an interface:

```idl
interface Service {
    // Read-write attribute
    attribute string name;

    // Read-only attribute
    readonly attribute long connection_count;
};
```

## One-Way Operations

One-way operations don't wait for a response:

```idl
interface Logger {
    oneway void log(in string message);
};
```

## Interface Inheritance

Interfaces can inherit from other interfaces:

```idl
interface BasicService {
    string get_version();
};

interface UserService : BasicService {
    User get_user(in string id);
};

// Multiple inheritance
interface AdminService : UserService, BasicService {
    void admin_operation();
};
```

## Complete Example

```idl
exception ServiceError {
    long error_code;
    string message;
};

exception NotFound {
    string entity_id;
};

interface DataStore {
    // Attributes
    readonly attribute long item_count;
    attribute string name;

    // Operations
    void put(in string key, in string value)
        raises (ServiceError);

    string get(in string key)
        raises (NotFound, ServiceError);

    void delete(in string key)
        raises (NotFound);

    sequence<string> list_keys();

    // One-way operation
    oneway void notify(in string event);
};
```

## Generated Code

Interfaces generate different constructs depending on the backend:

### Rust
```rust
pub trait UserService {
    fn get_user(&self, user_id: &str) -> Result<User, NotFound>;
    fn create_user(&mut self, user: User) -> Result<(), ValidationError>;
}
```

### Python
```python
class UserService(ABC):
    @abstractmethod
    def get_user(self, user_id: str) -> User:
        raise NotImplementedError

    @abstractmethod
    def create_user(self, user: User) -> None:
        raise NotImplementedError
```

### C++
```cpp
class UserService {
public:
    virtual User get_user(const std::string& user_id) = 0;
    virtual void create_user(const User& user) = 0;
    virtual ~UserService() = default;
};
```

## Best Practices

- Keep interfaces focused (single responsibility)
- Use exceptions for error conditions
- Prefer `in` parameters for most cases
- Use `readonly attribute` for computed values
- Document operations with comments

## Next Steps

- [Annotations](./annotations.md) - Customize code generation
- [Preprocessor](./preprocessor.md) - Macros and includes
