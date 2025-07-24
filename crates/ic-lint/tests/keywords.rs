// Copyright 2024 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use insta::assert_snapshot;

mod common;
use common::test_lint;

#[test]
fn valid_identifiers() {
    let source = r"
module ValidNames {
    struct MyStruct {
        long my_field;
        string another_field;
    };
    
    enum Status {
        ACTIVE,
        INACTIVE
    };
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_struct_name() {
    let source = r"
struct int32 {  // 'int32' is a primitive type keyword
    long value;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_field_name() {
    let source = r"
struct Data {
    octet octet;
    int32 int32;
    uint64 uint64;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_enum_value() {
    let source = r"
enum Type {
    int32,      // 'int32' is a keyword
    uint64,     // 'uint64' is a keyword
    octet       // 'octet' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_parameter_name() {
    let source = r"
interface Service {
    void process(in long int16, out string int32);
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_module_name() {
    let source = r"
module uint32 {  // 'uint32' is a primitive type keyword
    struct Point {
        long x;
        long y;
    };
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_union_member() {
    let source = r"
union Data switch (long) {
    case 0: string uint8;      // 'uint8' is a keyword
    case 1: long int64;        // 'int64' is a keyword
    case 2: boolean wchar;     // 'wchar' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_typedef_name() {
    let source = r"
typedef long uint32;
typedef boolean octet;
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_const_name() {
    let source = "const boolean abstract = TRUE;";
    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_exception_name() {
    let source = r"
exception uint16 {
    string message;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_interface_name() {
    let source = r"
interface abstract {
    void process();
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_bitmask_name() {
    let source = r"
bitmask abstract {
    FLAG1,
    FLAG2
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_bitset_name() {
    let source = r"
bitset uint64 {
    bitfield<5> bit0;
    bitfield<10> bit1;
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_interface_method() {
    let source = r"
interface Service {
    string context();
    long abstract();
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_native_type() {
    let source = r"
native Object;              // 'Object' is a keyword
native ValueBase;           // 'ValueBase' is a keyword
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn data_type_keywords() {
    let source = r"
struct DataTypes {
    long boolean;           // 'boolean' is a keyword
    long char;              // 'char' is a keyword  
    long int32;             // 'int32' is a keyword
    long uint16;            // 'uint16' is a keyword
    long octet;             // 'octet' is a keyword
    long abstract;          // 'abstract' is a keyword
    long uint64;            // 'uint64' is a keyword
    long truncatable;       // 'truncatable' is a keyword
    long multiple;          // 'multiple' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn integer_type_keywords() {
    let source = r"
struct IntTypes {
    long int8;              // 'int8' is a keyword
    long int16;             // 'int16' is a keyword
    long int32;             // 'int32' is a keyword
    long int64;             // 'int64' is a keyword
    unsigned long uint8;    // 'uint8' is a keyword
    unsigned long uint16;   // 'uint16' is a keyword
    unsigned long uint32;   // 'uint32' is a keyword
    unsigned long uint64;   // 'uint64' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn boolean_literal_keywords() {
    let source = r"
struct BoolLiterals {
    boolean int32;          // 'int32' is a keyword
    boolean uint8;          // 'uint8' is a keyword
    boolean octet;          // 'octet' is a keyword
    boolean wchar;          // 'wchar' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn component_keywords() {
    let source = r"
interface ComponentTest {
    void component();       // 'component' is a keyword
    void connector();       // 'connector' is a keyword
    void port();            // 'port' is a keyword
    void porttype();        // 'porttype' is a keyword
    void mirrorport();      // 'mirrorport' is a keyword
    void provides();        // 'provides' is a keyword
    void uses();            // 'uses' is a keyword
    void publishes();       // 'publishes' is a keyword
    void emits();           // 'emits' is a keyword
    void consumes();        // 'consumes' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn other_keywords() {
    let source = r"
struct OtherKeywords {
    long home;              // 'home' is a keyword
    string import;          // 'import' is a keyword
    boolean int16;          // 'int16' is a keyword (replacing 'local')
    float multiple;         // 'multiple' is a keyword
    double int32;           // 'int32' is a keyword (replacing 'private')
    octet uint8;            // 'uint8' is a keyword (replacing 'public')
    short int64;            // 'int64' is a keyword
    long truncatable;       // 'truncatable' is a keyword
    wchar typeid;           // 'typeid' is a keyword
    wstring typename;       // 'typename' is a keyword
    long typeprefix;        // 'typeprefix' is a keyword
    long eventtype;         // 'eventtype' is a keyword
    long finder;            // 'finder' is a keyword
    string manages;         // 'manages' is a keyword
    boolean primarykey;     // 'primarykey' is a keyword
    long abstract;          // 'abstract' is a keyword
    long custom;            // 'custom' is a keyword
};
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_as_map_type() {
    let source = r"
typedef map<string, long> uint16;  // 'uint16' is a keyword
";

    assert_snapshot!(test_lint(source));
}

#[test]
fn keyword_in_const_expression() {
    let source = r"
const long SIZE = 10;
const long abstract = SIZE + 5;      // 'any' is a keyword
";

    assert_snapshot!(test_lint(source));
}
