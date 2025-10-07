# Generated Code Details

This page shows exactly what code is generated for each IDL construct.

## Structures

### Basic Struct

**IDL:**
```idl
struct Person {
    string name;
    long age;
    boolean active;
};
```

**Generated Rust:**
```rust
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Person {
    pub name: ::std::string::String,
    pub age: i32,
    pub active: bool,
}

impl Person {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: <::std::string::String>::default(),
            age: 0,
            active: false,
        }
    }
}

impl ::std::default::Default for Person {
    fn default() -> Self {
        Self::new()
    }
}

// Marshal and Unmarshal traits also generated (see Serialization page)
```

Depending on the member types the backend may also derive `Eq`, `Ord`, and
`Hash`. The example above shows the minimum set that is always available.

### Using Generated Structs

```rust
// Create with default values
let person = Person::new();
assert_eq!(person.name, "");
assert_eq!(person.age, 0);
assert!(!person.active);

// Create with struct initialization
let person = Person {
    name: "Alice".to_string(),
    age: 30,
    active: true,
};

// Modify fields
let mut person = Person::new();
person.name = "Bob".to_string();
person.age = 25;
person.active = true;
```

## Enumerations

**IDL:**
```idl
enum Status {
    Active,
    Inactive,
    Pending
};
```

**Generated Rust:**
```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(i32)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

impl Status {
    #[must_use]
    pub const fn new() -> Self {
        Self::Active  // First variant is default
    }
}

impl ::std::str::FromStr for Status {
    type Err = ::intercom_cts::error::UnknownVariant;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "Active" => Ok(Self::Active),
            "Inactive" => Ok(Self::Inactive),
            "Pending" => Ok(Self::Pending),
            _ => Err(::intercom_cts::error::UnknownVariant),
        }
    }
}

impl ::std::fmt::Display for Status {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::Active => f.write_str("Active"),
            Self::Inactive => f.write_str("Inactive"),
            Self::Pending => f.write_str("Pending"),
        }
    }
}

impl ::std::default::Default for Status {
    fn default() -> Self {
        Self::new()
    }
}
```

### Using Generated Enums

```rust
// Create default
let status = Status::new();  // Status::Active

// Pattern matching
match status {
    Status::Active => println!("Active"),
    Status::Inactive => println!("Inactive"),
    Status::Pending => println!("Pending"),
}

// Parse from string
use std::str::FromStr;
let status = Status::from_str("Active").unwrap();

// Display
println!("{}", status);  // Prints "Active"

// Comparison
assert!(Status::Active < Status::Inactive);
```

## Unions

**IDL:**
```idl
union Value switch (long) {
    case 1: long int_value;
    case 2: double float_value;
    case 3: string string_value;
    default: boolean bool_value;
};
```

**Generated Rust:**
```rust
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    IntValue(i32),
    FloatValue(f64),
    StringValue(String),
    BoolValue(bool),
}
```

### Using Generated Unions

```rust
// Create variants
let v1 = Value::IntValue(42);
let v2 = Value::StringValue("hello".to_string());

// Pattern matching
match v1 {
    Value::IntValue(i) => println!("Integer: {}", i),
    Value::FloatValue(f) => println!("Float: {}", f),
    Value::StringValue(s) => println!("String: {}", s),
    Value::BoolValue(b) => println!("Bool: {}", b),
}
```

## Collections

### Sequences

**IDL:**
```idl
struct Data {
    sequence<long> numbers;
    sequence<string, 100> limited_strings;
};
```

**Generated Rust:**
```rust
pub struct Data {
    pub numbers: Vec<i32>,
    pub limited_strings: Vec<String>,  // Bound enforced in serialization
}
```

### Arrays

**IDL:**
```idl
struct Transform {
    double matrix[4][4];
    double position[3];
};
```

**Generated Rust:**
```rust
pub struct Transform {
    pub matrix: [[f64; 4]; 4],
    pub position: [f64; 3],
}
```

### Maps

**IDL:**
```idl
struct Config {
    map<string, string> properties;
};
```

**Generated Rust:**
```rust
pub struct Config {
    pub properties: ::std::collections::BTreeMap<String, String>,
}
```

## Complete Example

**IDL:**
```idl
enum Role {
    Admin,
    User,
    Guest
};

struct User {
    string name;
    long id;
    Role role;
    sequence<string> permissions;
};
```

**Usage:**
```rust
use std::collections::BTreeMap;

let user = User {
    name: "Alice".to_string(),
    id: 123,
    role: Role::Admin,
    permissions: vec!["read".to_string(), "write".to_string()],
};

// Check role
if user.role == Role::Admin {
    println!("{} is an admin", user.name);
}

// Iterate permissions
for perm in &user.permissions {
    println!("Permission: {}", perm);
}
```

## Next Steps

- [Type Mappings](./type-mappings.md) - Complete type mapping reference
- [Serialization](./serialization.md) - Using Marshal and Unmarshal
