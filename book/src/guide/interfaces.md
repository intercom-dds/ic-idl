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
