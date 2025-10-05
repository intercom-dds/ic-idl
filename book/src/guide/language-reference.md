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

# Language Reference

This section provides a complete reference for the IC-IDL language.

IC-IDL is based on the OMG Interface Definition Language (IDL) specification with extensions from DDS and custom features. It allows you to define data structures, interfaces, and protocols in a language-independent way.

## Topics

- **[Lexical Elements](./lexical-elements.md)** - Comments, identifiers, keywords, and basic syntax
- **[Primitive Types](./primitive-types.md)** - Boolean, integer, floating-point, character, and string types
- **[Constructed Types](./constructed-types.md)** - Structures, enumerations, unions, sequences, arrays, and maps
- **[Declarations](./declarations.md)** - Constants, type aliases, exceptions, bitmasks, and bitsets
- **[Modules](./modules.md)** - Organizing code with namespaces
- **[Interfaces](./interfaces.md)** - Defining service contracts and operations
- **[Annotations](./annotations.md)** - Controlling code generation and adding metadata
- **[Preprocessor](./preprocessor.md)** - Include files, macros, and conditional compilation

## Quick Example

Here's a simple IDL file demonstrating the basic syntax:

```idl
// Comments use C++ style
module example {
    // Enumeration
    enum Status {
        Active,
        Inactive
    };

    // Structure
    struct User {
        string name;
        long id;
        Status status;
    };

    // Interface
    interface UserService {
        User get_user(in long id);
        void update_status(in long id, in Status status);
    };
};
```

## Grammar Notation

Throughout this reference, we use the following notation:

- `keyword` - Literal keywords in the language
- `<rule>` - Grammar rules and placeholders
- `[optional]` - Optional elements
- `|` - Alternatives
- `...` - Repetition

## Standards Compliance

IC-IDL is compatible with:
- OMG IDL 4.2 specification
- DDS (Data Distribution Service) IDL extensions
- Custom extensions for modern code generation

Continue to the next sections to learn about each language feature in detail.
