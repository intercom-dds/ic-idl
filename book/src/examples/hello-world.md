# Hello World

A minimal "hello world" schema across Rust, Python, and C++.

## Schema

```idl
struct Message {
    string text;
    long timestamp;
};
```

## Generate code

```bash
ic-idl --rust-out rust/ --python-out python/ --cpp-out cpp/ message.idl
```

## Using the generated code

### Rust

```rust
use generated::Message;

let message = Message {
    text: "Hello, world!".to_string(),
    timestamp: 1_234_567_890,
};
```

### Python

```python
from generated.python.message import Message

message = Message(text="Hello, world!", timestamp=1_234_567_890)
```

### C++

```cpp
#include "generated/cpp/message.h"

message::Message msg;
msg.text = "Hello, world!";
msg.timestamp = 1234567890;
```

> **Tip:** leave generated code under version control and regenerate as part of
> your build or release process so consumers never need the IDL compiler at run
> time.
