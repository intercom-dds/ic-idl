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

#include <algorithm>
#include <cctype>
#include <filesystem>
#include <fstream>
#include <stdexcept>
#include <string>

#include "casing.h"
#include "interp.h"
#include "template.h"

namespace intercom::icgen {

/// A collection of builtin functions that are available to all Template instances.
namespace builtin {

/// Converts the specified identifier to snake_case.
inline std::string to_snake_case(Scope*, const FunctionData::Args& args) {
    auto input = args.at(0).str();
    CaseConverter conv(Case::Snake);
    return conv.convert(input);
}

/// Converts the specified identifier to camelCase.
inline std::string to_camel_case(Scope*, const FunctionData::Args& args) {
    auto input = args.at(0).str();
    CaseConverter conv(Case::Camel);
    return conv.convert(input);
}

/// Converts the specified identifier to PascalCase.
inline std::string to_pascal_case(Scope*, const FunctionData::Args& args) {
    auto input = args.at(0).str();
    CaseConverter conv(Case::Pascal);
    return conv.convert(input);
}

/// Converts the specified identifier to kebab-case.
inline std::string to_kebab_case(Scope*, const FunctionData::Args& args) {
    auto input = args.at(0).str();
    CaseConverter conv(Case::Kebab);
    return conv.convert(input);
}

/// Converts the input string to upper case.
inline std::string to_upper(Scope*, const FunctionData::Args& args) {
    auto value = args.at(0).str();
    for (auto& c : value) {
        c = std::toupper(c, std::locale());
    }
    return value;
}

/// Converts the input string to lower case.
inline std::string to_lower(Scope*, const FunctionData::Args& args) {
    auto value = args.at(0).str();
    for (auto& c : value) {
        c = std::tolower(c, std::locale());
    }
    return value;
}

/// Checks if the specified variable exists in the current scope.
/// This is useful if strict mode is enabled.
inline bool exists(Scope* scope, const FunctionData::Args& args) {
    auto value = args.at(0).str();
    return scope->contains(value);
}

/// Concatenates the given values into a single string, e.g.
///     `concat(my_var, "my_string", my_other_var)`
inline std::string concat(Scope*, const FunctionData::Args& args) {
    std::string result;
    for (const auto& arg : args) {
        result += arg.str();
    }
    return result;
}

/// Joins the specified list with the specified delimiter.
/// Takes an optional third param which specifies if the upper bound is inclusive.
inline std::string join(Scope*, const FunctionData::Args& args) {
    auto list = args.at(0).list();
    auto delim = args.at(1).str();
    bool inclusive = false;

    if (args.size() > 2) {
        inclusive = Evaluator::is_true(args.at(2));
    }

    std::string result;
    for (const auto& elem : list) {
        result += elem.str();

        if (&elem != &list.back() || inclusive) {
            result += delim;
        }
    }
    return result;
}

/// Joins the specified list with the specified delimiter, and
/// prefixes each element with the third parameter.
inline std::string join_prefix(Scope*, const FunctionData::Args& args) {
    auto list = args.at(0).list();
    auto delim = args.at(1).str();
    auto prefix = args.at(2).str();

    std::string result;
    for (const auto& elem : list) {
        result += prefix + elem.str();

        if (&elem != &list.back()) {
            result += delim;
        }
    }
    return result;
}

/// Equvialent to strftime.
/// Returns the date/time in the specified format, e.g.:
///     `time("%Y-%m-%d")`
inline std::string time(Scope*, const FunctionData::Args& args) {
    auto format = args.at(0).str();
    auto now = ::time(nullptr);
    auto info = localtime(&now);
    char buf[80];
    strftime(buf, sizeof(buf), format.data(), info);
    return buf;
}

/// Processes the specified file and inlines it in the current document.
/// The directory it searches can be overridden by setting the `data_dir` variable.
inline std::string include(Scope* scope, const FunctionData::Args& args) {
    std::stringstream stream;
    auto data_dir = scope->find("data_dir");
    std::filesystem::path dir = std::filesystem::current_path();

    if (data_dir) {
        dir = data_dir->str();
    }

    for (const auto& arg : args) {
        auto name = dir / arg.str();
        std::ifstream file(name);

        if (file.fail()) {
            throw std::runtime_error("couldn't open file '" + name.native() + '\'');
        }

        Template tmpl(scope);
        tmpl.process(file, stream);
        stream << '\n';
    }
    return stream.str();
}

/// Searches the input string for all instances of <param2> and replaces them with <param3>.
///     `replace(input_var, "replace_me", "replacement")`
inline std::string replace(Scope*, const FunctionData::Args& args) {
    auto input = args.at(0).str();
    auto replace = args.at(1).str();
    auto replacement = args.at(2).str();

    size_t pos{};
    while ((pos = input.find(replace)) != std::string::npos) {
        input.replace(pos, replace.length(), replacement);
    }
    return input;
}

/// Returns the value of <param1> if the variable exists, otherwise defaults to <param2>.
///     `value_or("my_var", "fallback value")`
inline Value value_or(Scope* scope, const FunctionData::Args& args) {
    const auto& var = args.at(0).str();
    const auto& fallback = args.at(1);

    if (scope->contains(var)) {
        return *scope->find(var);
    }
    return fallback;
}

/// Throws a runtime error with the specified message.
inline std::string error(Scope* scope, const FunctionData::Args& args) {
    auto msg = concat(scope, args);
    throw std::runtime_error(msg);
}

/// Returns a new list whose order of elements has been reversed.
inline std::vector<Value> reverse(Scope*, const FunctionData::Args& args) {
    auto list = args.at(0).list();
    std::reverse(list.begin(), list.end());
    return list;
}

/// Returns a single list containing all passed values.
inline std::vector<Value> to_list(Scope*, const FunctionData::Args& args) {
    std::vector<Value> values;
    for (const auto& elem : args) {
        values.emplace_back(elem);
    }
    return values;
}

// Helper function for print and dump. Not defined in the template instance.
inline void print_value(std::ostream& stream, const Value& value) {
    switch (value.kind()) {
    case Value::Kind::None:
        break;
    case Value::Kind::Bool:
        stream << (value.boolean() ? "true" : "false");
        break;
    case Value::Kind::String:
        stream << '"' << value.str() << '"';
        break;
    case Value::Kind::List: {
        const auto& list = value.list();
        stream << "list(";
        for (auto it = list.begin(); it != list.end(); ++it) {
            print_value(stream, *it);
            if (std::next(it) != list.end()) {
                stream << ", ";
            }
        }
        stream << ")";
    } break;
    case Value::Kind::Map: {
        stream << "map(";
        const auto& map = value.map()->values();
        for (auto it = map.begin(); it != map.end(); ++it) {
            stream << it->first << ": ";
            print_value(stream, it->second);

            if (std::next(it) != map.end()) {
                stream << ", ";
            }
        }
        stream << ")";
    } break;
    }
}

/// Prints out the parameters to stdout. Useful for debugging.
inline bool print(Scope*, const FunctionData::Args& args) {
    std::stringstream stream;

    for (auto it = args.begin(); it != args.end(); ++it) {
        print_value(stream, *it);
        if (std::next(it) != args.end()) {
            stream << ", ";
        }
    }
    std::cout << stream.str() << std::endl;
    return true;
}

/// Dumps all variables that are visible in the current lexical scope.
inline bool dump(Scope* scope, const FunctionData::Args&) {
    std::stringstream stream;
    stream << "dump(): values defined in the current scope:\n";

    std::function<void(const Scope*)> recurse = [&](const Scope* s) {
        if (auto parent = s->parent()) {
            recurse(parent);
        }
        for (const auto& entry : s->values()) {
            stream << " - " << entry.first << ": ";
            print_value(stream, entry.second);
            stream << std::endl;
        }
    };

    recurse(scope);
    std::cout << stream.str() << std::endl;
    return true;
}
}  // namespace builtin

inline void add_builtins(Template& tmpl) {
    // non-constant constants :-)
    tmpl.define("true", true);
    tmpl.define("false", false);

    // built-in functions
    tmpl.define("exists", 1, builtin::exists);
    tmpl.define("concat", -2, builtin::concat);
    tmpl.define("join", -2, builtin::join);
    tmpl.define("join_prefix", 3, builtin::join_prefix);
    tmpl.define("time", 1, builtin::time);
    tmpl.define("include", 1, builtin::include);
    tmpl.define("reverse", 1, builtin::reverse);
    tmpl.define("replace", 3, builtin::replace);
    tmpl.define("value_or", 2, builtin::value_or);
    tmpl.define("error", -1, builtin::error);
    tmpl.define("print", -1, builtin::print);
    tmpl.define("dump", 0, builtin::dump);
    tmpl.define("to_list", -1, builtin::to_list);

    // case-conversion functions
    tmpl.define("to_snake_case", 1, builtin::to_snake_case);
    tmpl.define("to_camel_case", 1, builtin::to_camel_case);
    tmpl.define("to_pascal_case", 1, builtin::to_pascal_case);
    tmpl.define("to_kebab_case", 1, builtin::to_kebab_case);
    tmpl.define("to_upper", 1, builtin::to_upper);
    tmpl.define("to_lower", 1, builtin::to_lower);
}

}  // namespace intercom::icgen
