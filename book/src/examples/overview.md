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
