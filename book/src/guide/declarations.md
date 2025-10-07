# Declarations

This page covers various declaration types in IC-IDL.

## Constants

Define compile-time constants with the `const` keyword:

```idl
const long MAX_SIZE = 1024;
const double PI = 3.14159265359;
const string VERSION = "1.0.0";
const boolean DEBUG = false;
```

### Using Constants

Constants can be used in array bounds and other constant expressions:

```idl
const long BUFFER_SIZE = 512;

struct Buffer {
    octet data[BUFFER_SIZE];
};
```

### Constant Expressions

Simple arithmetic is supported:

```idl
const long KB = 1024;
const long MB = KB * 1024;
const long GB = MB * 1024;
```

## Type Aliases

Create alternative names for types using `typedef`:

```idl
typedef long UserId;
typedef string<255> ShortString;
typedef sequence<long> NumberList;
typedef map<string, Person> UserMap;
```

###Using Type Aliases

```idl
struct Group {
    UserId owner_id;
    sequence<UserId> member_ids;
};
```

## Exceptions

Define error types for operations (used with interfaces):

```idl
exception NotFound {
    string entity_type;
    string entity_id;
    string message;
};

exception ValidationError {
    string field_name;
    string error_message;
};

exception PermissionDenied {
    string required_permission;
};
```

### Using Exceptions

```idl
interface UserService {
    User get_user(in string user_id)
        raises (NotFound, PermissionDenied);

    void create_user(in User user)
        raises (ValidationError);
};
```

## Bitmasks

Define bit flags (DDS extension):

```idl
@bit_bound(8)
bitmask FilePermissions {
    @position(0) READ,
    @position(1) WRITE,
    @position(2) EXECUTE,
    @position(3) DELETE
};
```

### Using Bitmasks

```idl
struct FileInfo {
    string filename;
    FilePermissions permissions;
};
```

Generated code supports bitwise operations:
```rust
let perms = FilePermissions::READ | FilePermissions::WRITE;
```

## Bitsets

Structured bit fields (DDS extension):

```idl
bitset ControlFlags {
    boolean enabled;
    boolean verbose;
    @bit_bound(4) unsigned short priority;
};
```

## Forward Declarations

Declare types before defining them:

```idl
struct Node;  // Forward declaration
struct Tree;

struct Node {
    long value;
    Tree* subtree;
};

struct Tree {
    sequence<Node> nodes;
};
```

## Module-Level Declarations

All declarations can appear at module scope:

```idl
module app {
    const long VERSION = 1;

    typedef long Id;

    exception AppError {
        string message;
    };

    struct Config {
        // ...
    };
};
```

## Next Steps

- [Modules](./modules.md) - Organize declarations with modules
- [Interfaces](./interfaces.md) - Define service contracts
- [Annotations](./annotations.md) - Customize generation
