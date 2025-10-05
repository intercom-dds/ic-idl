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

# Constructed Types

Constructed types allow you to build complex data structures from primitive types and other constructed types.

## Structures

Structures group related fields together.

### Basic Syntax

```idl
struct Person {
    string name;
    long age;
    string email;
};
```

### Field Types

Fields can be any type:

```idl
struct Complex {
    long primitive;
    string text;
    sequence<long> numbers;
    OtherStruct nested;
};
```

### Struct Inheritance

Structures can inherit from other structures:

```idl
struct Employee {
    long employee_id;
    string name;
    string department;
};

struct Manager : Employee {
    long team_size;
    double budget;
};
```

The `Manager` struct inherits all fields from `Employee`. In generated code:
- **Rust**: Fields are flattened into the child struct
- **C++**: Standard C++ inheritance
- **Python**: Composition or inheritance depending on backend options

### Forward Declarations

Use forward declarations for recursive or mutually referential types:

```idl
struct Node;  // Forward declaration

struct Tree {
    sequence<Node> children;
};

struct Node {
    long value;
    Tree subtree;
};
```

## Enumerations

Enumerations define a set of named integer constants.

### Basic Syntax

```idl
enum Status {
    Active,
    Inactive,
    Pending
};
```

Values start at 0 by default and increment by 1.

### Explicit Values

You can assign explicit values:

```idl
enum ErrorCode {
    Success = 0,
    NotFound = 404,
    ServerError = 500,
    BadRequest = 400
};
```

### Using Enums

```idl
struct User {
    string name;
    Status status;
};
```

Generated code:
- **Rust**: `pub enum Status { Active = 0, Inactive = 1, ... }`
- **Python**: `class Status(IntEnum): Active = 0, ...`
- **C++**: `enum class Status : int32_t { Active = 0, ... }`

## Unions

Unions are discriminated (tagged) unions that hold one of several possible types.

### Basic Syntax

```idl
union Value switch (long) {
    case 1: long int_value;
    case 2: double float_value;
    case 3: string string_value;
    default: boolean bool_value;
};
```

### Discriminator Types

The discriminator (switch type) can be:
- Integer types (`long`, `short`, etc.)
- `enum` types
- `boolean`

### Enum Discriminator

```idl
enum DataType {
    Integer,
    Float,
    String
};

union Data switch (DataType) {
    case Integer: long i;
    case Float: double f;
    case String: string s;
};
```

### Multiple Case Labels

Multiple cases can map to the same member:

```idl
union Result switch (long) {
    case 0:
    case 1:
    case 2: long success_value;
    default: string error_message;
};
```

### Default Case

The `default` case handles all unspecified discriminator values:

```idl
union Command switch (long) {
    case 1: long start;
    case 2: long stop;
    default: string unknown;  // All other values
};
```

## Sequences

Sequences are variable-length arrays.

### Unbounded Sequences

```idl
typedef sequence<long> Numbers;
typedef sequence<string> StringList;
typedef sequence<Person> People;
```

### Bounded Sequences

Limit the maximum number of elements:

```idl
typedef sequence<long, 100> Limited Numbers;
typedef sequence<Person, 1000> UserDatabase;
```

### Using Sequences

```idl
struct DataSet {
    sequence<double> values;
    sequence<string, 50> labels;
};
```

Generated code:
- **Rust**: `Vec<T>`
- **Python**: `list[T]`
- **C++**: `std::vector<T>`

## Arrays

Arrays are fixed-size sequences.

### One-Dimensional Arrays

```idl
typedef long Vector3[3];
typedef double Coefficients[10];
```

### Multi-Dimensional Arrays

```idl
typedef long Matrix3x3[3][3];
typedef double Grid[10][10][10];
```

### Using Arrays

```idl
struct Transform {
    double matrix[4][4];
    double translation[3];
};
```

Generated code:
- **Rust**: `[T; N]`
- **Python**: `list[T]` (with validation)
- **C++**: `std::array<T, N>`

## Maps

Maps represent key-value associations (DDS extension).

### Unbounded Maps

```idl
typedef map<string, long> StringToInt;
typedef map<long, Person> UserMap;
```

### Bounded Maps

Limit the maximum number of entries:

```idl
typedef map<string, double, 1000> Cache;
```

### Key Types

Key types must be comparable:
- Integer types
- String types
- Enum types

### Using Maps

```idl
struct Configuration {
    map<string, string> properties;
    map<long, User> users;
};
```

Generated code:
- **Rust**: `BTreeMap<K, V>` or `HashMap<K, V>`
- **Python**: `dict[K, V]`
- **C++**: `std::map<K, V>`

## Type Aliases

Create alternative names for types using `typedef`:

```idl
typedef long UserId;
typedef string<255> ShortString;
typedef sequence<Person> PersonList;
typedef map<UserId, Person> UserDatabase;

struct Group {
    UserId owner;
    PersonList members;
};
```

## Nested Types

You can nest type definitions within modules:

```idl
module geometry {
    struct Point {
        double x;
        double y;
    };

    struct Line {
        Point start;
        Point end;
    };

    typedef sequence<Point> Path;
};
```

## Examples

### Complete Example

```idl
// Enum for status
enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled
};

// Simple struct
struct Address {
    string street;
    string city;
    string postal_code;
    string country;
};

// Struct with inheritance
struct Customer {
    long customer_id;
    string name;
    string email;
};

struct PremiumCustomer : Customer {
    double discount_rate;
    long loyalty_points;
};

// Union for payment methods
union PaymentMethod switch (long) {
    case 1: string credit_card;
    case 2: string bank_account;
    case 3: string crypto_wallet;
};

// Complex struct using all types
struct Order {
    long order_id;
    Customer customer;
    Address shipping_address;
    OrderStatus status;
    sequence<long> item_ids;
    map<long, long> item_quantities;
    PaymentMethod payment;
    double total_amount;
};
```

## Best Practices

### Use Structs for Related Data

Group related fields together:

```idl
// Good
struct Dimensions {
    double width;
    double height;
    double depth;
};

// Instead of separate fields
```

### Use Enums for Fixed Sets

When a field has a fixed set of possible values:

```idl
enum Priority {
    Low,
    Normal,
    High,
    Urgent
};

struct Task {
    string title;
    Priority priority;  // Not just 'long level'
};
```

### Use Unions for Variant Data

When data can be one of several types:

```idl
union ResponseData switch (long) {
    case 200: SuccessData success;
    case 404: NotFoundError not_found;
    case 500: ServerError server_error;
};
```

### Bounded vs Unbounded Collections

- Use **bounded** for known limits (better performance, prevent DoS)
- Use **unbounded** when size is truly unknown

```idl
struct Config {
    sequence<string, 10> tags;        // Bounded - reasonable limit
    sequence<LogEntry> history;       // Unbounded - can grow indefinitely
};
```

## Next Steps

- [Declarations](./declarations.md) - Learn about constants, exceptions, and more
- [Modules](./modules.md) - Organize types with modules
- [Annotations](./annotations.md) - Customize generated code
