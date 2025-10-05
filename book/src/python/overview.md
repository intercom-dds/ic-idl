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

# Python Backend Overview

The Python backend generates a module hierarchy that mirrors your IDL modules
and builds on the `intercom_dds` runtime package. Each struct becomes a Python
class with runtime validation, enums reuse `Enum`, and unions provide a typed
interface for the active variant.

## Quick start

```bash
ic-idl schema.idl --python-out generated/python
```

```
generated/python/
└── demo/
    ├── __init__.py
    └── schema.py
```

Use the bindings directly:

```python
from generated.python.demo.schema import Person, Status

person = Person(name="Alice", age=30, status=Status.Active)
person.age = 31  # validated assignment
```

`__init__.py` files re-export everything to keep imports short.

## Runtime dependency

Install the runtime package alongside your project:

```bash
pip install intercom-dds
```

The generated code imports `intercom_dds.core.exceptions` and
`intercom_dds.intercom_types` for base classes, type information, and
serialisation helpers.

## Key features

- Classes derive from `BaseStruct`/`BaseUnion` and expose slots for attribute
  storage; property setters perform type conversions and range checks.
- Constructors accept keyword arguments for every field. Missing arguments fall
  back to sensible defaults (`0`, `0.0`, empty strings/containers, first
  enumerator, nested struct instances, …).
- Enums inherit from `BaseEnum` and use `enum.auto()` for subsequent values,
  preserving ordinal ordering compatible with other backends.
- Modules automatically import dependencies from sibling modules so cross-module
  references work out-of-the-box.
- Type annotations rely on `typing` so editors and type checkers understand the
  generated API.

## Next steps

- [Type mappings](./type-mappings.md)
- [Generated code tour](./generated-code.md)
- [Serialisation helpers](./serialization.md)
- [Build integration](./build-integration.md)
