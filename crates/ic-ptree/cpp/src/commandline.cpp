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

#include "cidl/commandline.h"

#include <fmt/format.h>

#include <cstdlib>
#include <cstring>
#include <vector>

#include "utils/string_utils.h"

using namespace intercom::cidl;

namespace {
struct CurrentOptionsT : public intercom::cidl::Config {
    bool intercom_build = false;
    bool do_suppress_deprecated = false;
    bool use_fmtlib = false;
    bool use_wstring = false;
    bool no_typesupport = false;

    std::string copyright_notice;
    std::string json_target_directory;
    std::string json_schema_target_directory;
    std::string rust_target_directory;
    std::string python_global_postfix;
    std::string ada_target_directory;
    std::string idl_target_directory;
    std::string xml_target_directory;
    std::string toml_target_directory;
    std::vector<std::string> input_list;
    std::vector<std::string> include_directories{"."};
    std::vector<std::string> pp_parameters;
    static constexpr unsigned long DEFAULT_ERRORS =
        1UL << CommandLineOption::WARNING_UNKNOWN_ANNOTATION |
        1UL << CommandLineOption::WARNING_UNCATEGORIZED_ERROR;
    static constexpr unsigned long DEFAULT_WARNINGS =
        DEFAULT_ERRORS | 1UL << CommandLineOption::WARNING_UNCATEGORIZED_WARNING;
    unsigned long enabled_errors = DEFAULT_ERRORS;
    unsigned long enabled_warnings = DEFAULT_WARNINGS;
} g_CurrentOptions;

struct WarningDetails {
    const char* warningName;
    CommandLineOption::WarningType type;
    const char* documentation;
};
}  // namespace

static bool g_silent = false;

static WarningDetails g_all_warnings[] = {
    {"all", CommandLineOption::WARNING_ALL, "Switch for all warnings"},
    {"deprecated",
     CommandLineOption::WARNING_DEPRECATED,
     "Warn on deprecated constructs (that may be discontinued in the future)"},
    {"annotation",
     CommandLineOption::WARNING_ANNOTATION,
     "Warn on suspicious annotation usage such as conflicting annotations."},
    {"unknown-annotation",
     CommandLineOption::WARNING_UNKNOWN_ANNOTATION,
     "Warn on use of unknown annotation"},
    {"pedantic",
     CommandLineOption::WARNING_PEDANTIC,
     "Warn on use of language extensions or implementation-defined behavior"},
    {"error", CommandLineOption::WARNING_ERROR, "Treat warnings like errors"},
};

static CommandLineOption::WarningType find_warning(const char* argv, bool& enable) {
    for (const auto& current : g_all_warnings) {
        if (strcmp(argv, current.warningName) == 0) {
            enable = true;
            return current.type;
        }
        if (strncmp(argv, "no-", 3) == 0 && strcmp(argv + 3, current.warningName) == 0) {
            enable = false;
            return current.type;
        }
    }
    if (!g_silent) {
        fmt::print(stderr, "Warning '{}' not recognized\n", argv);
    }
    return CommandLineOption::WARNING_NONE;
}

static void update_warning_flags(
    const CommandLineOption::WarningType& type,
    unsigned long& warning_mask = g_CurrentOptions.enabled_warnings,
    bool enable = true
) {
    if (type == CommandLineOption::WARNING_ALL) {
        // WARNING_ALL means all warnings except for WARNING_ERROR (as it's not really a warning)
        warning_mask = enable ? ~(1 << CommandLineOption::WARNING_ERROR) : 0;
    } else if (type != CommandLineOption::WARNING_NONE) {
        if (enable) {
            warning_mask |= 1 << int(type);
        } else {
            warning_mask &= ~(1 << int(type));
        }
    }
}

static void update_warning(const char* argv, bool option_select) {
    const auto split_argv = string_utils::split_at(argv, '=');
    const bool has_target = split_argv.first != split_argv.second;
    bool enable = false;
    CommandLineOption::WarningType type = find_warning(split_argv.first.c_str(), enable);
    if (!option_select) {
        enable = !enable;
    }

    // get argument of -W[no-]error=...
    const bool werror = type == CommandLineOption::WARNING_ERROR;
    if (werror) {
        bool enable_target = true;
        type = has_target ? find_warning(split_argv.second.c_str(), enable_target)
                          : CommandLineOption::WARNING_ALL;
        if (type == CommandLineOption::WARNING_ERROR || !enable_target) {
            fmt::print(stderr, "Warning '{}' was given unsupported type after '='\n", argv);
            return;
        }
    } else if (has_target) {
        fmt::print(stderr, "Warning '{}' had unexpected '='\n", argv);
        return;
    }

    if (werror) {
        update_warning_flags(type, g_CurrentOptions.enabled_errors, enable);
    } else {
        update_warning_flags(type, g_CurrentOptions.enabled_warnings, enable);
    }
}

static const char* strptr_or_null(const std::string& value) {
    if (value.empty()) {
        return nullptr;
    }
    return value.c_str();
}

// Special gloabl to instruct lexer not to emit doxy comments
extern "C" bool g_ignore_doxy_comments;

CommandLineOption::ScopeDefaultWarnings::ScopeDefaultWarnings()
    : m_prev_errors(CurrentOptionsT::DEFAULT_ERRORS),
      m_prev_warnings(CurrentOptionsT::DEFAULT_WARNINGS) {
    std::swap(m_prev_errors, g_CurrentOptions.enabled_errors);
    std::swap(m_prev_warnings, g_CurrentOptions.enabled_warnings);
}
CommandLineOption::ScopeDefaultWarnings::~ScopeDefaultWarnings() {
    std::swap(m_prev_errors, g_CurrentOptions.enabled_errors);
    std::swap(m_prev_warnings, g_CurrentOptions.enabled_warnings);
}

void CommandLineOption::set_warning(const CommandLineOption::WarningType& type, bool enable) {
    update_warning_flags(type, g_CurrentOptions.enabled_warnings, enable);
}
bool CommandLineOption::intercom_build() {
    return g_CurrentOptions.intercom_build;
}
bool CommandLineOption::generate_typesupport_only() {
    return g_CurrentOptions.generate_typesupport_only;
}
bool CommandLineOption::list_only() {
    return g_CurrentOptions.list_only;
}
bool CommandLineOption::no_rename() {
    return g_CurrentOptions.no_rename;
}
bool CommandLineOption::proxies() {
    return g_CurrentOptions.proxies;
}
bool CommandLineOption::compatibility() {
    return g_CurrentOptions.compatibility;
}
bool CommandLineOption::no_corba_dependency() {
    return g_CurrentOptions.no_corba_dependency;
}
bool CommandLineOption::cpp_gen_cpp11() {
    return g_CurrentOptions.cpp_gen_cpp11;
}
bool CommandLineOption::cpp_no_stream_op() {
    return g_CurrentOptions.cpp_no_stream_op;
}
bool CommandLineOption::cpp_access_functions() {
    return g_CurrentOptions.cpp_access_functions;
}
bool CommandLineOption::doxy_compatible_output() {
    return g_CurrentOptions.doxy_compatible_output;
}
bool CommandLineOption::expand_idl() {
    return g_CurrentOptions.expand_idl;
}
bool CommandLineOption::legacy_idl() {
    return g_CurrentOptions.legacy_idl;
}
bool CommandLineOption::generate_default_literals() {
    return g_CurrentOptions.generate_default_literals;
}
bool CommandLineOption::use_fmtlib() {
    return g_CurrentOptions.use_fmtlib;
}
bool CommandLineOption::no_typesupport() {
    return g_CurrentOptions.no_typesupport;
}
bool CommandLineOption::corba_types() {
    return g_CurrentOptions.corba_types;
}
bool CommandLineOption::use_wstring() {
    return g_CurrentOptions.use_wstring;
}
const char* CommandLineOption::cpp_header_postfix() {
    return strptr_or_null(g_CurrentOptions.cpp_header_postfix);
}
const char* CommandLineOption::c_file_prefix() {
    return strptr_or_null(g_CurrentOptions.c_file_prefix);
}
const char* CommandLineOption::dll_exp_sym() {
    return strptr_or_null(g_CurrentOptions.dll_exp_sym);
}
const char* CommandLineOption::cs_target_directory() {
    return strptr_or_null(g_CurrentOptions.cs_target_directory);
}
const char* CommandLineOption::c_target_directory() {
    return strptr_or_null(g_CurrentOptions.c_target_directory);
}
const char* CommandLineOption::java_target_directory() {
    return strptr_or_null(g_CurrentOptions.java_target_directory);
}
const char* CommandLineOption::java_package_prefix() {
    return strptr_or_null(g_CurrentOptions.java_package_prefix);
}
const char* CommandLineOption::json_target_directory() {
    return strptr_or_null(g_CurrentOptions.json_target_directory);
}
const char* CommandLineOption::json_schema_target_directory() {
    return strptr_or_null(g_CurrentOptions.json_schema_target_directory);
}
const char* CommandLineOption::rust_target_directory() {
    return strptr_or_null(g_CurrentOptions.rust_target_directory);
}
const char* CommandLineOption::python_target_directory() {
    return strptr_or_null(g_CurrentOptions.python_target_directory);
}
const char* CommandLineOption::python_global_postfix() {
    return strptr_or_null(g_CurrentOptions.python_global_postfix);
}
const char* CommandLineOption::ada_target_directory() {
    return strptr_or_null(g_CurrentOptions.ada_target_directory);
}
const char* CommandLineOption::idl_target_directory() {
    return strptr_or_null(g_CurrentOptions.idl_target_directory);
}
const char* CommandLineOption::xml_target_directory() {
    return strptr_or_null(g_CurrentOptions.xml_target_directory);
}
const char* CommandLineOption::proto_target_directory() {
    return strptr_or_null(g_CurrentOptions.proto_target_directory);
}
const char* CommandLineOption::toml_target_directory() {
    return strptr_or_null(g_CurrentOptions.toml_target_directory);
}
const char* CommandLineOption::ada_package_prefix() {
    return strptr_or_null(g_CurrentOptions.ada_package_prefix);
}
const char* CommandLineOption::header_subfolder() {
    return strptr_or_null(g_CurrentOptions.header_subfolder);
}
const char* CommandLineOption::copyright_notice() {
    return strptr_or_null(g_CurrentOptions.copyright_notice);
}

intercom::cidl::Config& CommandLineOption::get_instance() {
    return g_CurrentOptions;
}

bool CommandLineOption::suppress_warning(WarningType warning) {
    unsigned long mask_value = 1 << int(warning);
    return ((g_CurrentOptions.enabled_warnings & mask_value) == 0);
}

bool CommandLineOption::suppress_error(WarningType type) {
    unsigned long mask_value = 1 << int(type);
    return ((g_CurrentOptions.enabled_errors & mask_value) == 0);
}

bool CommandLineOption::suppress_alert(WarningType type) {
    return suppress_warning(type) && suppress_error(type);
}

const std::vector<std::string>& CommandLineOption::get_input_list() {
    return g_CurrentOptions.input_list;
}

const std::vector<std::string>& CommandLineOption::get_parameters() {
    return g_CurrentOptions.pp_parameters;
}

const std::vector<std::string>& CommandLineOption::include_directories() {
    return g_CurrentOptions.include_directories;
}
