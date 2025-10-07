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
