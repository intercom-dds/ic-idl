# IDL-TypeScript Mapping

This document defines how IDL constructs map to TypeScript types.

The mapping targets ES2022 and is designed to be compatible with DDS-JSON. The
types will transparently serialize to JSON via `JSON.stringify()` and
`JSON.parse()`.

## Primitive Types

| IDL Type | TypeScript Type | Notes |
|----------|-----------------|-------|
| `void` | `void` | Return types only |
| `boolean` | `boolean` | |
| `char`, `wchar` | `string` | Single character |
| `int8`, `octet` | `number` | |
| `uint8` | `number` | |
| `int16`, `short` | `number` | |
| `uint16`, `unsigned short` | `number` | |
| `int32`, `long` | `number` | |
| `uint32`, `unsigned long` | `number` | |
| `int64`, `long long` | `number \| string` | String outside ±2⁵³ |
| `uint64`, `unsigned long long` | `number \| string` | String outside ±2⁵³ |
| `float`, `double` | `number` | |
| `long double` | `number` | May lose precision |

### 64-bit Integers

DDS-JSON specifies that 64-bit integers in the range [−2⁵³ + 1, 2⁵³ − 1] are represented
as JSON numbers, while values outside that range are represented as strings. We map to
`number | string` to reflect this.

A command-line flag enables mapping to `bigint` instead, which is more ergonomic but
requires a custom JSON library like `lossless-json` for serialization.

```idl
struct Primitives {
    boolean flag;
    int32 count;
    int64 bigNumber;
};
```

```typescript
interface Primitives {
    flag: boolean;
    count: number;
    bigNumber: number | string;
}
```

## Structs

Structs become interfaces. We use interfaces rather than classes because structs are pure
data without behavior, and interfaces work directly with object literals and `JSON.parse()`
without requiring instantiation.

```idl
struct Point {
    long x;
    long y;
};

struct Point3D : Point {
    long z;
};
```

```typescript
interface Point {
    x: number;
    y: number;
}

interface Point3D extends Point {
    z: number;
}
```

Self-referencing works as expected:

```idl
struct Node {
    long value;
    sequence<Node> children;
};
```

```typescript
interface Node {
    value: number;
    children: Node[];
}
```

## Enums

We use regular enums rather than `const enum` because regular enums exist at runtime,
which is needed for reverse lookup during deserialization.

```idl
enum Color {
    RED,
    GREEN,
    BLUE
};

enum Status {
    OK = 0,
    WARNING = 100,
    ERROR = 200
};
```

```typescript
enum Color {
    RED = 0,
    GREEN = 1,
    BLUE = 2,
}

enum Status {
    OK = 0,
    WARNING = 100,
    ERROR = 200,
}
```

## Bitmasks

Bitmasks become enums with `2 ** n` values. We use `2 ** n` rather than `1 << n` because
JavaScript bitwise operators are 32-bit, which breaks for positions >= 31.

```idl
bitmask Permissions {
    READ,
    WRITE,
    EXECUTE
};
```

```typescript
enum Permissions {
    READ = 2 ** 0,
    WRITE = 2 ** 1,
    EXECUTE = 2 ** 2,
}
```

The `@position` annotation specifies bit index, not value:

```idl
bitmask Flags {
    @position(0) FLAG_A,
    @position(4) FLAG_B,
};
```

```typescript
enum Flags {
    FLAG_A = 2 ** 0,  // 1
    FLAG_B = 2 ** 4,  // 16
}
```

Bit positions are limited to 0-52 due to JavaScript's 53-bit integer precision. Combined
bitmask values have type `number`, not the enum type, because TypeScript doesn't
understand bitwise combinations.

## Unions

IDL unions become TypeScript discriminated unions with a `$discriminator` field, as
specified by DDS-JSON. Checking the discriminator narrows to the correct variant.

```idl
enum ValueType {
    INT,
    DOUBLE,
    STRING
};

union Value switch (ValueType) {
case INT:
    long int_value;
case DOUBLE:
    double double_value;
case STRING:
    string string_value;
};
```

```typescript
enum ValueType {
    INT = 0,
    DOUBLE = 1,
    STRING = 2,
}

type Value =
    | { $discriminator: ValueType.INT; int_value: number }
    | { $discriminator: ValueType.DOUBLE; double_value: number }
    | { $discriminator: ValueType.STRING; string_value: string };
```

Multiple labels per case become union types in the discriminator:

```idl
union Multi switch (long) {
case 1:
case 2:
case 3:
    long small;
default:
    boolean fallback;
};
```

```typescript
type Multi =
    | { $discriminator: 1 | 2 | 3; small: number }
    | { $discriminator: number; fallback: boolean };
```

## Exceptions

Exceptions become classes extending `Error`. Unlike structs, exceptions need to be
classes because they require throw/catch semantics and the Error prototype chain.

```idl
exception InvalidArgument {
    string message;
    long error_code;
};
```

```typescript
class InvalidArgument extends Error {
    error_code: number;

    constructor(message: string, error_code: number, options?: { cause?: Error }) {
        super(message, options);
        this.name = 'InvalidArgument';
        this.error_code = error_code;
    }
}
```

## Interfaces

IDL interfaces become TypeScript interfaces with methods.

```idl
interface Calculator {
    long add(in long a, in long b);
    long subtract(in long a, in long b);
};
```

```typescript
interface Calculator {
    add(a: number, b: number): number;
    subtract(a: number, b: number): number;
}
```

### Parameter Directions

The `in` direction is a normal parameter. The `out` and `inout` directions are included
in the return value rather than mutating parameters, since mutation is unidiomatic in
TypeScript.

When there's a single output (return value, `out`, or `inout`), we return it directly.
With multiple outputs, we bundle them into an object using parameter names as keys. If
there's a non-void return plus out/inout parameters, the return value uses `$return` as
its key.

```idl
interface Service {
    void getValues(out long x, out long y);
    void modify(inout long value);
    long compute(in long x, inout long state, out long debug);
};
```

```typescript
interface Service {
    getValues(): { x: number; y: number };
    modify(value: number): number;
    compute(x: number, state: number): {
        $return: number;
        state: number;
        debug: number
    };
}
```

## Valuetypes

Valuetypes have both data members and methods. We split them into a data interface
(for serialization) and a full interface (with methods).

```idl
valuetype ComplexValue {
    public long x;
    public long y;
    long magnitude();
};
```

```typescript
interface ComplexValueData {
    x: number;
    y: number;
}

interface ComplexValue extends ComplexValueData {
    magnitude(): number;
}
```

## Type Aliases

```idl
typedef long Integer;
typedef sequence<long> IntList;
```

```typescript
type Integer = number;
type IntList = number[];
```

## Collections

### Arrays and Sequences

Both become `T[]`. Bounds are validated at runtime.

```idl
typedef long Numbers[3];
typedef sequence<long, 100> BoundedList;
```

```typescript
type Numbers = number[];
type BoundedList = number[];
```

### Maps

Maps become `Record<K, V>` rather than ES6 `Map` because `Map` doesn't serialize
transparently to JSON.

TypeScript's `Record<K, V>` requires `K` to be `string | number | symbol`. Since JSON
object keys are always strings, non-string key types use template literal types to
represent string-encoded values:

| IDL Key Type | TypeScript Key Type |
|--------------|---------------------|
| `string` | `string` |
| Integer types | `` `${number}` `` |
| `boolean` | `` `${boolean}` `` |
| Enum types | `keyof typeof EnumType` |

For enums, we use `keyof typeof EnumType` because DDS-JSON prefers enumerator names
as keys (e.g., `"RED"` rather than `0`), though both are accepted. This gives the
type `"RED" | "GREEN" | "BLUE"` for the enum's member names.

```idl
typedef map<string, long> StringIntMap;
typedef map<long, string> IntStringMap;
typedef map<boolean, long> BoolIntMap;
```

```typescript
type StringIntMap = Record<string, number>;
type IntStringMap = Record<`${number}`, string>;
type BoolIntMap = Record<`${boolean}`, number>;
```

For complex key types (structs, unions), the keys are serialized as strings. Since
there is no well-defined canonical string representation, these maps use `string`
as the key type.

### Strings

Both `string` and `wstring` map to `string` since JavaScript strings are UTF-16
internally. Bounds are validated at runtime.

## Modules


IDL modules map to ES6 module files. We use ES6 modules rather than TypeScript
namespaces because they are the modern standard, support tree-shaking, and work
well with bundlers.

```idl
module math {
    struct Point {
        long x;
        long y;
    };
    const long ANSWER = 42;
};
```

```typescript
// math.ts
export interface Point {
    x: number;
    y: number;
}

export const ANSWER = 42;
```

Nested modules become subdirectories:

```idl
module outer {
    module inner {
        struct Data {
            long value;
        };
    };
};
```

```typescript
// outer/inner.ts
export interface Data {
    value: number;
}

// outer/index.ts
export * as inner from './inner';
```

## Constants

```idl
const long MAX_SIZE = 1024;
const string GREETING = "Hello";
```

```typescript
export const MAX_SIZE = 1024;
export const GREETING = "Hello";
```

## Optional Values

The `@optional` annotation maps to optional property syntax. Per DDS-JSON, `null`
values are omitted from serialization rather than included as explicit `null`.

```idl
struct User {
    string id;
    @optional string email;
};
```

```typescript
interface User {
    id: string;
    email?: string;
}
```

## The `any` Type

The IDL `any` type maps to `unknown`, not TypeScript's `any`. We use `unknown` because
it requires type checking before use, catching errors at compile time. With `any`,
TypeScript skips all type checking.

```idl
struct Message {
    any data;
};
```

```typescript
interface Message {
    data: unknown;
}
```

## Reserved Identifiers

Reserved names are escaped by appending an underscore (e.g., `class` becomes `class_`).

### Hard Reserved (ECMAScript)

These cannot be used as identifiers anywhere.

| | | | |
|:---|:---|:---|:---|
| break | case | catch | class |
| const | continue | debugger | default |
| delete | do | else | enum |
| export | extends | false | finally |
| for | function | if | import |
| in | instanceof | new | null |
| return | super | switch | this |
| throw | true | try | typeof |
| var | void | while | with |

### Strict Mode Reserved

TypeScript modules always run in strict mode, so these are effectively reserved.

| | | |
|:---|:---|:---|
| arguments | let | public |
| eval | package | static |
| implements | private | yield |
| interface | protected | |

### Contextual Keywords

These are technically valid as identifiers in some places, but using them causes
confusion or linter errors. We escape them in generated code.

| Keyword | Risk |
|:---|:---|
| any | TS primitive type |
| as | Casting syntax |
| async | Async functions |
| await | Reserved in async/top-level |
| boolean | TS primitive type |
| constructor | Reserved method name |
| declare | Declaration files |
| get | Getter definition |
| infer | Conditional types |
| is | Type guards |
| keyof | Index type queries |
| module | Module declaration |
| namespace | Namespace declaration |
| never | TS primitive type |
| number | TS primitive type |
| readonly | Property modifier |
| require | CommonJS import |
| set | Setter definition |
| string | TS primitive type |
| symbol | TS primitive type |
| type | Type alias definition |
| unique | unique symbol |
| unknown | TS primitive type |
