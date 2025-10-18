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

#include <fmt/ostream.h>

#include <list>
#include <map>
#include <memory>
#include <set>
#include <sstream>
#include <string>
#include <vector>

#include "cidl/numeric.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"

extern "C" struct parse_result {
    parse_result() = default;
    const ptree* tree{nullptr};
    std::set<const ptree*> includes;
    std::set<std::string> modules;
    size_t error_count{0};
    std::string msg;
    std::shared_ptr<parser_state> state;
};

extern "C" struct parser_state {
    struct error_stream;

    ptree* lookup_node(const char* name) const;
    error_stream error();

    long long enum_counter{0};
    std::vector<std::vector<ptree*>> context;
    std::vector<std::pair<std::string, ptree*>> include_context;
    std::map<std::string, ptree*> type_map;
    std::map<std::string, ptree*> type_dcl_map;
    std::vector<std::shared_ptr<ptree>> allocated_nodes;
    std::vector<std::shared_ptr<declarator>> allocated_decl;
    std::list<numeric> numeric_map;
    std::vector<std::string> errors;
    ptree top_level;
};

struct parser_state::error_stream {
    explicit error_stream(parser_state* parent) : m_parent(parent) {
        m_index = parent->errors.size();
        parent->errors.emplace_back();
    }

    ~error_stream() {
        m_parent->errors[m_index] = m_stream.str();
    }

    template <typename T>
    error_stream& operator<<(const T& src) {
        m_stream << src;
        return *this;
    }

    error_stream& operator<<(ptree* node) {
        m_stream << intercom::cidl::idl_scoped_name(node, nullptr);
        return *this;
    }

    error_stream& operator<<(const ptree* node) {
        m_stream << intercom::cidl::idl_scoped_name(node, nullptr);
        return *this;
    }

    parser_state* m_parent;
    size_t m_index;
    std::stringstream m_stream;
};

namespace intercom::cidl {

enum class JsonValueFlags { FLAG_ESCAPED = 1, FLAG_NUMERICAL_VALUE = 2 };

std::string json_value(const numeric& value, const ptree* context = nullptr, int flags = 0);

}  // namespace intercom::cidl

template <>
struct fmt::formatter<numeric> : public fmt::formatter<std::string> {
    template <typename FormatContext>
    auto format(const numeric& num, FormatContext& ctx) const -> decltype(ctx.out()) {
        return formatter<std::string>::format(intercom::cidl::string_value(num), ctx);
    }
};
