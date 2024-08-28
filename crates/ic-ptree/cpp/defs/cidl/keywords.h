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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#pragma once

const char* const CPP_KEYWORDS[] = {
    "alignas",       "alignof",      "and",       "and_eq",    "asm",  //
    "auto",          "bitand",       "bitor",     "bool",      "break",        "case",
    "catch",         "char",         "char16_t",  "char32_t",  "char8_t",      "class",
    "co_await",      "co_return",    "co_yield",  "compl",     "concept",      "const",
    "const_cast",    "consteval",    "constexpr", "constinit", "continue",     "decltype",
    "default",       "delete",       "do",        "double",    "dynamic_cast", "else",
    "enum",          "explicit",     "export",    "extern",    "false",        "float",
    "for",           "friend",       "goto",      "if",        "import",       "inline",
    "int",           "long",         "module",    "mutable",   "namespace",    "new",
    "noexcept",      "not",          "not_eq",    "nullptr",   "operator",     "or",
    "or_eq",         "private",      "protected", "public",    "register",     "reinterpret_cast",
    "requires",      "return",       "short",     "signed",    "sizeof",       "static",
    "static_assert", "static_cast",  "struct",    "switch",    "synchronized", "template",
    "this",          "thread_local", "throw",     "true",      "try",          "typedef",
    "typeid",        "typename",     "union",     "unsigned",  "using",        "virtual",
    "void",          "volatile",     "wchar_t",   "while",     "xor",          "xor_eq",
    nullptr
};

const char* const IDL_KEYWORDS[] = {
    "abstract",  "any",       "alias",      "attribute",  "bitfield",  "bitmask",   "bitset",
    "boolean",   "case",      "char",       "component",  "connector", "const",     "consumes",
    "context",   "custom",    "default",    "double",     "exception", "emits",     "enum",
    "eventtype", "factory",   "false",      "finder",     "fixed",     "float",     "getraises",
    "home",      "import",    "in",         "inout",      "interface", "local",     "long",
    "manages",   "map",       "mirrorport", "module",     "multiple",  "native",    "object",
    "octet",     "oneway",    "out",        "primarykey", "private",   "port",      "porttype",
    "provides",  "public",    "publishes",  "raises",     "readonly",  "setraises", "sequence",
    "short",     "string",    "struct",     "supports",   "switch",    "true",      "truncatable",
    "typedef",   "typeid",    "typename",   "typeprefix", "unsigned",  "union",     "uses",
    "valuebase", "valuetype", "void",       "wchar",      "wstring",   "int8",      "uint8",
    "int16",     "int32",     "int64",      "uint16",     "uint32",    "uint64",    nullptr
};

const char* const JAVA_KEYWORDS[] = {
    "abstract",     "default",   "if",     "private",    "throw",    "boolean",  "do",
    "implements",   "protected", "throws", "break",      "double",   "import",   "public",
    "transient",    "byte",      "else",   "instanceof", "return",   "try",      "case",
    "extends",      "int",       "short",  "void",       "catch",    "final",    "interface",
    "static",       "volatile",  "char",   "finally",    "long",     "super",    "while",
    "class",        "float",     "native", "switch",     "const",    "for",      "new",
    "synchronized", "continue",  "goto",   "package",    "this",     "true",     "false",
    "null",         "clone",     "equals", "finalize",   "getClass", "hashCode", "notify",
    "notifyAll",    "toString",  "wait",   nullptr
};

const char* const ADA_KEYWORDS[] = {
    "abort",   "abs",        "abstract",  "accept",    "access",  "aliased",      "all",
    "and",     "array",      "at",        "begin",     "body",    "case",         "constant",
    "declare", "delay",      "delta",     "digits",    "do",      "else",         "elsif",
    "end",     "entry",      "exception", "exit",      "for",     "function",     "generic",
    "goto",    "if",         "in",        "interface", "is",      "limited",      "loop",
    "mod",     "new",        "not",       "null",      "of",      "or",           "others",
    "out",     "overriding", "package",   "pragma",    "private", "procedure",    "protected",
    "raise",   "range",      "record",    "rem",       "renames", "requeue",      "return",
    "reverse", "select",     "separate",  "some",      "subtype", "synchronized", "tagged",
    "task",    "terminate",  "then",      "type",      "until",   "use",          "when",
    "while",   "with",       "xor",       nullptr
};

const char* const CS_KEYWORDS[] = {
    "abstract", "as",         "base",    "bool",     "break",     "byte",     "case",
    "catch",    "char",       "checked", "class",    "const",     "continue", "decimal",
    "default",  "delegate",   "do",      "double",   "else",      "enum",     "event",
    "explicit", "extern",     "false",   "finally",  "fixed",     "float",    "for",
    "foreach",  "goto",       "if",      "implicit", "in",        "int",      "interface",
    "internal", "is",         "lock",    "long",     "namespace", "new",      "null",
    "object",   "operator",   "out",     "override", "params",    "private",  "protected",
    "public",   "readonly",   "ref",     "return",   "sbyte",     "sealed",   "short",
    "sizeof",   "stackalloc", "static",  "string",   "struct",    "switch",   "this",
    "throw",    "true",       "try",     "typeof",   "uint",      "ulong",    "unchecked",
    "unsafe",   "ushort",     "using",   "using",    "static",    "virtual",  "void",
    "volatile", "while",      nullptr
};

const char* const PYTHON_KEYWORDS[] = {
    "and",        "as",       "assert",   "break",     "class",        "continue", "def",
    "del",        "elif",     "else",     "except",    "False",        "finally",  "for",
    "from",       "global",   "if",       "import",    "in",           "is",       "lambda",
    "None",       "nonlocal", "not",      "or",        "pass",         "raise",    "return",
    "True",       "try",      "while",    "with",      "yield",        "abs",      "aiter",
    "all",        "any",      "anext",    "ascii",     "bin",          "bool",     "breakpoint",
    "bytearray",  "bytes",    "callable", "chr",       "classmethod",  "compile",  "complex",
    "delattr",    "dict",     "dir",      "divmod",    "enumerate",    "eval",     "exec",
    "filter",     "float",    "format",   "frozenset", "getattr",      "globals",  "hasattr",
    "hash",       "help",     "hex",      "id",        "input",        "int",      "isinstance",
    "issubclass", "iter",     "len",      "list",      "locals",       "map",      "max",
    "memoryview", "min",      "next",     "object",    "oct",          "open",     "ord",
    "pow",        "print",    "property", "range",     "repr",         "reversed", "round",
    "set",        "setattr",  "slice",    "sorted",    "staticmethod", "str",      "sum",
    "super",      "tuple",    "type",     "vars",      "zip",          nullptr
};

const char* const RUST_KEYWORDS[] = {
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
    // not keywords, but types from the prelude we reserve to make things more readable
    "String",
    "Option",
    "Box",
    "Vec",
    nullptr
};

const char* const PROTO_KEYWORDS[] = {
    "syntax",   "map",      "int32",   "import",   "extensions", "int64",    "weak",
    "reserved", "uint32",   "public",  "rpc",      "uint64",     "package",  "stream",
    "sint32",   "option",   "returns", "sint64",   "inf",        "to",       "fixed32",
    "nan",      "max",      "fixed64", "message",  "repeated",   "sfixed32", "enum",
    "optional", "sfixed64", "service", "required", "bool",       "extend",   "string",
    "float",    "group",    "bytes",   "double",   "oneof",      nullptr
};
