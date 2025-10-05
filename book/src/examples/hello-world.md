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
