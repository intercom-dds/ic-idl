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

# Examples Overview

This section provides practical examples demonstrating IC-IDL features.

## Basic Examples

- [Hello World](./hello-world.md) - Minimal example
- [Basic Types](./basic-types.md) - All primitive and constructed types
- [User Service](./user-service.md) - Complete CRUD service

## Feature Examples

- [Modules](./modules.md) - Organizing code with namespaces
- [Annotations](./annotations.md) - Customizing code generation

## Integration Examples

- [Build Integration](./build-integration.md) - Cargo, CMake, setuptools
- [Multi-Language Project](./multi-language.md) - Using Rust, Python, and C++ together

## Running Examples

Each example includes:
- IDL schema definition
- Generated code samples
- Usage examples

To generate code from any example:

```bash
ic-idl --rust-out rust/ example.idl
ic-idl --python-out python/ example.idl
ic-idl --cpp-out cpp/ example.idl
```
