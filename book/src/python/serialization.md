# Serialization

The generated classes inherit from the `intercom_dds` runtime which provides
binary (CDR) and JSON encoding/decoding helpers compatible with the other
backends.

## Runtime support

- `intercom_dds.intercom_types.BaseStruct` and `BaseUnion` expose metadata
  tables (`_type_info`, `_member_info`) describing the shape of each type.
- The runtime uses this metadata to marshal values to CDR, validate bounds, and
  materialise objects from incoming payloads.
- Optional members, keys, and union discriminants are handled transparently by
  the runtime so long as you assign `None`/`BaseUnion.set_*` appropriately.

Refer to the `intercom_dds` package documentation for the exact helper
functions. A typical workflow looks like:

```python
# Depending on the runtime version you may need to import from
# `intercom_dds.cdr1` or a similar module.
from intercom_dds import cdr
from generated.python.demo.schema import Person

person = Person(name="Alice", age=30)

# Encode to CDR (little endian)
cdr_bytes = cdr.to_le_bytes(person)

# Decode back
copy = cdr.from_le_bytes(Person, cdr_bytes)
assert copy.name == "Alice"
```

The same pattern applies to JSON helpers if your deployment needs a textual
format for debugging or REST APIs.

## Interoperability

Because the Rust and C++ backends rely on the same metadata, data produced in
Python can be consumed by those languages without additional mapping. Make sure
both sides agree on the endianness and version of the CDR protocol.
