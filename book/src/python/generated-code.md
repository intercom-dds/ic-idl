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

# Generated code tour

This section dissects the Python output for a small schema.

```idl
module demo {
    enum Status { Active, Inactive };

    struct Person {
        string name;
        long age;
        Status status;
    };
}
```

The backend emits `demo/schema.py`:

```python
from enum import auto as _auto_
import intercom_dds.core.exceptions as _except_
import intercom_dds.intercom_types
import typing as _typing_

class Status(intercom_dds.intercom_types.BaseEnum):
    Active = 0
    Inactive = _auto_()

class Person(intercom_dds.intercom_types.BaseStruct):
    __slots__ = ('_name', '_age', '_status')

    def __init__(self, name: str = None, age: int = None, status: 'Status' = None):
        super().__init__()
        self.name = "" if name is None else name
        self.age = int(0) if age is None else age
        self.status = Status.Active if status is None else status

    @property
    def name(self) -> str:
        return self._name

    @name.setter
    def name(self, value: str) -> None:
        if not isinstance(value, str):
            raise TypeError("name must be str")
        self._name = value

    # Similar properties for `age` and `status`...
```

Highlights:

- `__slots__` keeps attribute storage minimal and prevents accidental attribute
  creation.
- Setters normalise types (numbers go through `int()`/`float()`) and enforce
  invariants (e.g. enum instances).
- When a field references another generated type the setter verifies the value
  is either `None` or an instance of the referenced class.
- Default values mirror the Rust/C++ backends so serialisation remains
  deterministic.

### Metadata

Each class defines `_type_info` and `_member_info` tables consumed by the CTS
runtime. You rarely need to touch them manually, but they enable features such
as optional members, key fields, and union discriminants.

### Unions

Unions expose helpers for manipulating the active case:

```python
class Value(intercom_dds.intercom_types.BaseUnion):
    __slots__ = ('_kind', '_int_value', '_string_value')

    def as_int_value(self) -> int:
        if self._kind != 0:
            raise _except_.BadUnionAccess("Expected discriminator 0")
        return self._int_value

    def set_int_value(self, value: int) -> None:
        self._kind = 0
        self._int_value = int(value)
```

### Imports between modules

When a struct refers to a type defined in another module, the generator inserts
plain `import` statements at the top of the file so qualified names are
available (e.g. `import demo.common`). The `scoped_name` helper ensures
references stay valid regardless of the module hierarchy.

See also [Python type mappings](./type-mappings.md) and
[Serialization](./serialization.md) for additional details.
