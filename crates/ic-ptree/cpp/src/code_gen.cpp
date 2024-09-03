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

#include <fcntl.h>
#include <fmt/chrono.h>

#include <cassert>
#include <chrono>
#include <cstdarg>
#include <cstring>

#include "cidl/hdrs.h"
#include "cidl/memf.h"
#include "cidl/ptree_builder.h"
#include "cidl/ptree_helpers.h"

static bool is_path_sep(char c) {
#ifdef INTERCOM_PLATFORM_WINDOWS
    return c == '/' || c == '\\';
#else
    return c == '/';
#endif
}

namespace intercom::cidl {

std::string trim_include_name(std::filesystem::path name, bool trim_absolute) {
    auto file = name.replace_extension();
    std::string native = name.string();
    if (trim_absolute && (is_path_sep(native[0]) ||
                          (native[0] != '\0' && native[1] == ':' && is_path_sep(native[2])) ||
                          (native[0] == '.' && native[1] == '.' && is_path_sep(native[2])))) {
        return file.stem().string();
    }
    return file.string();
}

parse_result clone_tree(const parse_result* result) {
    IdlParser parser;
    parser.run([&](auto state) { return duplicate_tree(state, result->tree); });

    auto clone = parser.result();
    clone.error_count = result->error_count;
    clone.warning_count = result->warning_count;
    clone.modules = result->modules;
    clone.msg = result->msg;
    for (auto inc : result->includes) {
        clone.includes.emplace(duplicate_node(clone.state.get(), inc));
    }
    return clone;
}

std::string copyright_header(const std::string& comment_str) {
    constexpr const char* header =
        "{0} KONGSBERG PROPRIETARY - This software, related documentation and its accompanying elements,\n"
        "{0} contain information which is proprietary and confidential to KONGSBERG or its licensors.\n"
        "{0} Any disclosure, copying, distribution or use is prohibited if not otherwise explicitly agreed\n"
        "{0} with KONGSBERG in writing. It is strictly prohibited to modify, reverse engineer, decompile,\n"
        "{0} or disassemble the software, unless such acts are allowed under applicable mandatory law or\n"
        "{0} explicitly agreed with KONGSBERG in writing. Any authorized reproduction, in whole or in part,\n"
        "{0} must include this legend. (C) {1:%Y} KONGSBERG - All rights reserved\n";
    return fmt::format(header, comment_str, std::chrono::system_clock::now());
}

std::string tolower(std::string res) {
    std::transform(res.begin(), res.end(), res.begin(), [](std::string::value_type c) {
        return static_cast<std::string::value_type>(std::tolower(static_cast<int>(c)));
    });
    return res;
}

static void
print_doc_annotation(struct memf* f, const ptree* doc_annotation, const bool& print_as_post_doc) {
    const ptree* elem;
    const char* docs = nullptr;
    for (elem = doc_annotation->members; elem; elem = elem->next) {
        if (elem->name == "text") {
            docs = elem->value.val.str().c_str();
            break;
        }
    }
    MemfIndentScopeLock indent_lk(f);  // comments should not affect indentation
    if (docs) {
        const int post_padding = 2;
        std::string line_start = print_as_post_doc ? "//!" : " * ";
        std::string newline_padding;
        if (print_as_post_doc) {
            int spaces = std::max(f->column + post_padding - f->indent, 0);
            newline_padding += std::string(spaces, ' ');
        }
        int start_of_line = 1;
        const char* pp;
        if (print_as_post_doc) {
            mprintf(f, std::string(post_padding, ' '));
        } else {
            mprintf(f, "/**\n");
        }
        for (pp = docs; *pp; pp++) {
            if (start_of_line) {
                if (pp != docs) {
                    mprintf(f, "{}", newline_padding);
                }
                mprintf(f, "{}", line_start);
            }
            if (*pp) {
                start_of_line = *pp == '\n';
                mprintf(f, "{}", start_of_line ? '\n' : *pp);
            }
        }
        if (!start_of_line) {
            mprintf(f, "\n");
        }
        if (!print_as_post_doc) {
            mprintf(f, " */\n");
        }
    }
}

void emit_docs(struct memf* f, const ptree* obj) {
    if (!f || !obj) {
        return;
    }
    for (const ptree* ann : obj->annotations) {
        if (is_pre_doc(ann)) {
            print_doc_annotation(f, ann, false);
        }
    }
}

void emit_post_docs(struct memf* f, const ptree* obj) {
    if (!f || !obj) {
        return;
    }
    bool no_comments = true;
    for (const ptree* ann : obj->annotations) {
        if (is_post_doc(ann)) {
            print_doc_annotation(f, ann, true);
            no_comments = false;
        }
    }
    if (no_comments) {
        mprintf(f, "\n");
    }
}

}  // namespace intercom::cidl
