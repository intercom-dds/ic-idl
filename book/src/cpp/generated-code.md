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

Consider:

```idl
module demo {
    struct Person {
        string name;
        long age;
    };
}
```

The backend produces `demo/person.h` and `demo/person.cpp`.

### Header snippet

```cpp
#pragma once

#include <array>
#include <cstdint>
#include <map>
#include <memory>
#include <string>
#include <vector>

#include <ic_cts/member_info.h>
#include <ic_cts/memory.h>

namespace demo {

struct Person {
    Person();
    Person(const Person&);
    Person& operator=(const Person&);
    Person(Person&&) noexcept;
    Person& operator=(Person&&) noexcept;
    ~Person() noexcept;

    bool operator==(const Person& other) const;
    bool operator!=(const Person& other) const;
    bool operator<(const Person& other) const;

    ::std::string name;
    int32_t age;
};

}
```

The struct exposes canonical special members, comparison operators, and public
fields. Optional helpers such as stream operators and `{fmt}` formatters are
added when corresponding CLI flags are used.

### Implementation snippet

```cpp
#include "person.h"
#include <ic_cts/dds_xtypes_constants.h>

namespace demo {

namespace {
constexpr ::ic_cts::TypeInfo PERSON_TYPE_INFO{
    "demo::Person",
    ::ic_cts::TypeFlag::IS_APPENDABLE,
    ::ic_cts::TypeKind::Struct,
    ::ic_cts::TypeKind::None,
    ::ic_cts::TypeKind::None,
};

constexpr ::ic_cts::MemberInfo PERSON_MEMBERS[]{
    {"name", 0, ::ic_cts::MemberFlag::nil()},
    {"age", 1, ::ic_cts::MemberFlag::nil()},
};
} // namespace

Person::Person() : name(), age(0) {}

// copy/move constructors ...

::ic_cts::Result Person::marshal(::ic_cts::Archive& ar) const {
    ar.encode_struct(PERSON_TYPE_INFO);
    ar.encode_field(PERSON_MEMBERS[0], name);
    ar.encode_field(PERSON_MEMBERS[1], age);
    ar.end_struct();
    return ::ic_cts::Result::Ok;
}
```

The `.cpp` file hosts the metadata tables and serialisation routines. Every
backed type has equivalent support functions.

### Dependencies

Headers include the standard library pieces they use as well as the CTS headers.
When one IDL file references types from another the generator inserts
`#include` directives for the corresponding generated headers.

### Unions and interfaces

Unions come with discriminant accessors, typed getters/setters, and a private
`union` storing the active payload. Interfaces map to abstract classes with
pure-virtual methods and helper result types.

For more elaborate examples inspect the integration tests under
`e2e-tests/cpp/` or generate code for the `tests/idl` corpus.
