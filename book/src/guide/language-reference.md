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
