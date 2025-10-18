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

#pragma once

#include <fstream>
#include <stdexcept>
#include <string_view>

#include "interp.h"
#include "marshal.h"
#include "parser.h"

namespace intercom::icgen {

void add_builtins(class Template& tmpl);

class Template {
  public:
    /// Creates a new template instance with an external scope.
    explicit Template(Scope* scope) : m_owned(false), m_scope(scope) {}

    /// Creates a new template instance with its own scope.
    Template() {
        add_builtins(*this);
    }

    ~Template() {
        if (!m_owned) {
            m_scope.release();  // NOLINT
        }
    }

    /// Defines and initializes a variable with the specified value.
    void define(const std::string& var, bool value) {
        m_scope->assign(var, value);
    }

    /// Defines and initializes a variable with the specified value.
    void define(const std::string& var, const char* value) {
        m_scope->assign(var, value);
    }

    /// Defines and initializes a variable with the specified value.
    void define(const std::string& var, const std::string& value) {
        m_scope->assign(var, value);
    }

    /// Defines and initializes a list with the given values.
    void define(const std::string& var, const std::vector<Value>& list) {
        m_scope->assign(var, list);
    }

    /// Defines and initializes a map/structure with the given values.
    /// Can be accessed in the template using the dot operator: `my_map.my_key` yields `my_value`.
    void define(const std::string& var, Scope* map) {
        m_scope->assign(var, Value(*map));
    }

    /// Defines a callback that will be triggered if the function is invoked
    /// in the template, e.g.:
    ///   `[[ my_func(param1, "my string") ]]`
    ///
    /// See `builtin.h` for the built-in functions.
    ///
    /// `n_args` is the number of arguments the function expects.
    /// For variadic functions, negative integers can be used to represent
    /// the number of required arguments. For example, -2 means the function requires
    /// at least two parameters, but accepts more.
    void define(const std::string& name, int n_args, FunctionData::Callback callback) {
        FunctionData data;
        data.kind = FunctionData::Kind::String;
        data.n_args = n_args;
        data.callback = std::move(callback);
        m_scope->define(name, data);
    }

    /// Takes an arbitrary IDL-defined type as parameter and converts it into
    /// a template variable with the specified name. This works for complex,
    /// nested data structures. Member variables can be accessed using the dot
    /// operator: `my_struct.my_member.my_nested_member`.
    template <typename T>
    void define_var(const std::string& var, const T& value) {
        ValueMarshal marshal;
        marshal.io(value);
        m_scope->assign(var, marshal.value());
    }

    /// If strict mode is enabled, the parser will throw an exception if it
    /// encounters an unknown identifier. Disabled by default.
    ///
    /// This can be overridden in the template itself by assigning the `strict` variable:
    ///   `[% strict = true %]`
    ///
    void strict(bool strict) {
        define("strict", strict);
    }

    /// Parses the input and performs the relevant replacements.
    void process(std::string_view input, std::ostream& stream) {
        auto tokens = tokenize(input);
        Parser parser(tokens);
        auto ast = parser.parse();

        Interp interp(m_scope.get(), stream);
        for (const auto& node : ast) {
            interp.execute(node.get());
        }
    }

    /// Parses the input and performs the relevant replacements.
    /// Helper function for dealing with streams.
    void process(std::istream& input, std::ostream& stream) {
        std::stringstream buf;
        buf << input.rdbuf();
        process(buf.str(), stream);
    }

    /// Parses the input and performs the relevant replacements.
    /// Helper function for dealing with streams.
    void from_file(const std::string& filename, std::ostream& stream) {
        std::ifstream buf(filename);
        if (buf.fail()) {
            throw std::runtime_error("opening template '" + filename + "' failed");
        }
        try {
            process(filename, stream);
        } catch (std::exception& e) {
            throw std::runtime_error(filename + ":" + e.what());
        }
    }

    /// Sets the search path for other templates.
    /// Only relevant for the `include` function. Can be overridden on a
    /// per-template basis by setting the "data_dir" variable in the template.
    void include_dir(const std::string& path) {
        define("data_dir", path);
    }

    /// Returns the interpreter's global scope.
    Scope* scope() const {
        return m_scope.get();
    }

  private:
    bool m_owned{true};
    std::unique_ptr<Scope> m_scope{new Scope()};
};
}  // namespace intercom::icgen

#include "builtin.h"
