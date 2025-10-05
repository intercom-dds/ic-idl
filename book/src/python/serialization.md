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
