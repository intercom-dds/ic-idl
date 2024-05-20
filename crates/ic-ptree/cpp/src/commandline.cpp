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

#include "InterCOM/version.h"
#include "cidl/hdrs.h"
#include "icgen/cli/CommandLine.h"

namespace cli = intercom::cli;
using namespace intercom::cidl;

namespace {
bool g_silent = false;

struct CurrentOptionsT : public intercom::cidl::Config {
    bool version = false;
    bool intercom_build = false;
    bool do_suppress_deprecated = false;
    bool pp_preprocess_only = false;
    bool pp_preprocess_skip = false;
    bool use_fmtlib = false;
    bool generate_header_timestamp = false;
    bool use_wstring = false;
    bool ast_dump = false;
    bool purge_destination_directories = false;
    bool no_typesupport = false;

    std::string cs_target_directory;
    std::string c_target_directory;
    std::string copyright_notice;
    std::string json_target_directory;
    std::string json_schema_target_directory;
    std::string rust_target_directory;
    std::string python_target_directory;
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
} g_AllWarnings[] = {
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

CommandLineOption::WarningType find_warning(const char* argv, bool& enable) {
    for (const auto& current : g_AllWarnings) {
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

void update_warning_flags(
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

void update_warning(const char* argv, bool option_select) {
    const auto split_argv = intercom::cli::detail::split_at(argv, '=');
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
            return fmt::print(stderr, "Warning '{}' was given unsupported type after '='\n", argv);
        }
    } else if (has_target) {
        return fmt::print(stderr, "Warning '{}' had unexpected '='\n", argv);
    }

    if (werror) {
        update_warning_flags(type, g_CurrentOptions.enabled_errors, enable);
    } else {
        update_warning_flags(type, g_CurrentOptions.enabled_warnings, enable);
    }
}

const char* strptr_or_null(const std::string& value) {
    if (value.empty()) {
        return nullptr;
    }
    return value.c_str();
}
}  // namespace

// Special gloabl to instruct lexer not to emit doxy comments
extern "C" bool g_ignore_doxy_comments;

namespace config {
auto build_commandline() -> cli::CommandLine {
    auto version = cli::detail::replace(INTERCOM_VERSION_S, '_', '.');

    // clang-format off
    auto cli = cli::CommandLine("cidl")
        .version(version)
        .chaining(false)
        .positionals(true, "files")
        .opts(
            // Flags
            cli::Option("--silent")
                .desc("Do not issue any CIDL parameter messages")
                .var(g_silent),

            cli::Option("--cpp11")
                .desc("Use C++11 support")
                .callback([&](bool enabled) {
                    g_CurrentOptions.corba_types = false;
                    g_CurrentOptions.cpp_gen_cpp11 = enabled;
                    g_CurrentOptions.cpp_access_functions = enabled;
                }),

            cli::Option("--no-stream-op")
                .desc("Do not generate stream output operators in C++")
                .var(g_CurrentOptions.cpp_no_stream_op),

            cli::Option("--access-functions")
                .desc("Use access functions instead of direct member access")
                .var(g_CurrentOptions.cpp_access_functions),

            cli::Option("--ignore-comments")
                .desc("Do not attempt to parse doxy-like comments to include them in generated code")
                .var(g_ignore_doxy_comments),

            cli::Option("--vendor-compatibility")
                .desc("Generate include files compatible with other vendors")
                .var(g_CurrentOptions.compatibility),

            cli::Option("--typesupport-only")
                .desc("Only generate typesupport")
                .var(g_CurrentOptions.generate_typesupport_only),

            cli::Option("--no-typesupport")
                .desc("Do not generate typesupport")
                .var(g_CurrentOptions.no_typesupport),

            cli::Option("--ada-no-corba")
                .desc("Do not use CORBA namespace")
                .var(g_CurrentOptions.no_corba_dependency),

            cli::Option("--no-rename")
                .desc("Do not rename generated types")
                .deprecated("--csharp-no-rename")
                .var(g_CurrentOptions.no_rename),

            cli::Option("--string-utf8")
                .desc("Use UTF-8 for string types")
                .var(g_CurrentOptions.string_encoding_utf8),

            cli::Option("--idl-doxygen")
                .desc("Output Doxygen-compatible IDL files")
                .var(g_CurrentOptions.doxy_compatible_output),

            cli::Option("--idl-expand")
                .desc("Expand @DDSService to topics in IDL files")
                .var(g_CurrentOptions.expand_idl),

            cli::Option("--idl-legacy")
                .desc("Attempt to emit IDL that is more friendly for older parsers")
                .var(g_CurrentOptions.legacy_idl),

            cli::Option("--do-proxies")
                .hidden(true)
                .var(g_CurrentOptions.proxies),

            cli::Option("--generate-default-literals")
                .desc("Generate constants for default values")
                .var(g_CurrentOptions.generate_default_literals),

            cli::Option("--std-types")
                .desc("Use types from the standard library instead of CORBA types")
                .hidden(true)
                .deprecated(true)
                .callback(cli::neg(g_CurrentOptions.corba_types)),

            cli::Option("--corba-types")
                .desc("Use CORBA types instead of types from the standard library")
                .var(g_CurrentOptions.corba_types),

            cli::Option("--use-wstring")
                .desc("Use std::wstring for wide-character strings")
                .var(g_CurrentOptions.use_wstring),

            cli::Option("--use-fmt")
                .desc("Generate formatting specializations for fmtlib")
                .var(g_CurrentOptions.use_fmtlib),

            cli::Option("--ast-dump")
                .hidden(true)
                .var(g_CurrentOptions.ast_dump),

            cli::Option("--generate-header-timestamp")
                .desc("Output build timestamp at the top of each generated file")
                .var(g_CurrentOptions.generate_header_timestamp),

            cli::Option("--purge-destination-directories")
                .desc("Empties all destination directories before emitting result")
                .var(g_CurrentOptions.purge_destination_directories),

            cli::Option("-l", "--list")
                .desc("Output list of files to be generated")
                .var(g_CurrentOptions.list_only),

            cli::Option("-H", "--no-header-follow")
                .desc("Do not generate code for included modules")
                .var(g_CurrentOptions.disable_header_follow),

            cli::Option("-E", "--preprocessor-only")
                .desc("Run preprocessor only")
                .var(g_CurrentOptions.pp_preprocess_only),

            cli::Option("-X", "--preprocessor-skip")
                .desc("Skip preprocessor")
                .var(g_CurrentOptions.pp_preprocess_skip),

            // Options
            cli::Option("-D", "--preprocessor-define")
                .desc("Preprocessor define")
                .value(cli::MULTIPLE, "def")
                .multiple(false),

            cli::Option("-I", "--include")
                .desc("Include directory")
                .value(cli::MULTIPLE, "dir")
                .multiple(false)
                .var(g_CurrentOptions.include_directories),

            cli::Option("-W", "--warning")
                .desc("Turn on or off individual warnings")
                .value(cli::MULTIPLE, "warn")
                .multiple(false)
                .callback([](const cli::Args& args) {
                    for (const std::string& warn : args) {
                        update_warning(warn.c_str(), true);
                    }
                }),

            cli::Option("-a", "--ada-destination")
                .desc("Generate Ada files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.ada_target_directory),

            cli::Option("-j", "--java-destination")
                .desc("Generate Java files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.java_target_directory),

            cli::Option("-c", "--cpp-destination")
                .desc("Generate C++ files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.c_target_directory),

            cli::Option("-s", "--csharp-destination")
                .desc("Generate C# files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.cs_target_directory),

            cli::Option("-i", "--idl-destination")
                .desc("Generate IDL files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.idl_target_directory),

            cli::Option("-r", "--rust-destination")
                .desc("Generate Rust files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.rust_target_directory),

            cli::Option("--json-destination")
                .desc("Generate JSON files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.json_target_directory),

            cli::Option("--json-schema-destination")
                .desc("Generate JSON Schema files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.json_schema_target_directory),

            // Only for internal use
            cli::Option("--toml-destination")
                .hidden(true)
                .value(cli::SINGLE)
                .var(g_CurrentOptions.toml_target_directory),

            cli::Option("--xml-destination")
                .desc("Generate XML files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.xml_target_directory),

            cli::Option("--proto-destination")
                .desc("Generate Protobuf files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.proto_target_directory),

            cli::Option("--python-destination")
                .desc("Generate Python files in the specified directory")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.python_target_directory),

            cli::Option("--python-global-postfix")
                .desc("Specify the postix for global modules")
                .value(cli::SINGLE, "postfix")
                .var(g_CurrentOptions.python_global_postfix),

            cli::Option("--java-package-prefix")
                .desc("Use Java package prefix")
                .value(cli::SINGLE, "prefix")
                .var(g_CurrentOptions.java_package_prefix),

            cli::Option("--cpp-header-postfix")
                .desc("Use postfix for C++ headers")
                .value(cli::SINGLE, "postfix")
                .var(g_CurrentOptions.cpp_header_postfix),

            cli::Option("--cpp-file-prefix")
                .desc("Append file prefix for C++ files")
                .value(cli::SINGLE, "prefix")
                .var(g_CurrentOptions.c_file_prefix),

            cli::Option("--dll-export-symbol")
                .desc("Use dllexp symbol")
                .value(cli::SINGLE, "symbol")
                .var(g_CurrentOptions.dll_exp_sym),

            cli::Option("--header-subfolder")
                .desc("Store header files inside a subfolder instead of with the source files")
                .value(cli::SINGLE, "dir")
                .var(g_CurrentOptions.header_subfolder)
       );

    cli.after_help("To disable a warning, add 'no-' before the warning text (e.g. -Wno-all)");
    cli.section("warnings");

    for (const auto& warning : g_AllWarnings) {
        auto name = fmt::format("-W{}", warning.warningName);
        cli.opts(
            cli::Option(name)
                .desc(warning.documentation)
                .callback([=](const cli::Args&){
                    update_warning(warning.warningName, true);
                })
        );
    }
    // clang-format on
    return cli;
}

void parse_options(const cli::ParseResult& result) {
    // Fix special stuff that intercom uses to build internally.
    // We need to do this first, because command line arguments should override these.
    if (getenv("INTERCOM_BUILD")) {
        g_CurrentOptions.intercom_build = true;
        g_CurrentOptions.use_fmtlib = true;
        g_CurrentOptions.header_subfolder = "InterCOM";
        g_CurrentOptions.copyright_notice = copyright_header();
    }
    // end fix

    auto includes = g_CurrentOptions.include_directories;
    for (const auto& inc : includes) {
        g_CurrentOptions.pp_parameters.push_back("-I" + inc);
    }

    auto defines = result.get_vec<std::string>("--preprocessor-define");
    for (const auto& def : defines) {
        g_CurrentOptions.pp_parameters.push_back("-D" + def);
    }

    g_CurrentOptions.input_list = result.positionals();
}
}  // namespace config

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
bool CommandLineOption::version() {
    return g_CurrentOptions.version;
}
bool CommandLineOption::intercom_build() {
    return g_CurrentOptions.intercom_build;
}
bool CommandLineOption::preprocessor_only() {
    return g_CurrentOptions.pp_preprocess_only;
}
bool CommandLineOption::preprocessor_skip() {
    return g_CurrentOptions.pp_preprocess_skip;
}
bool CommandLineOption::disable_header_follow() {
    return g_CurrentOptions.disable_header_follow;
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
bool CommandLineOption::string_encoding_utf8() {
    return g_CurrentOptions.string_encoding_utf8;
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
bool CommandLineOption::generate_header_timestamp() {
    return g_CurrentOptions.generate_header_timestamp;
}
bool CommandLineOption::corba_types() {
    return g_CurrentOptions.corba_types;
}
bool CommandLineOption::use_wstring() {
    return g_CurrentOptions.use_wstring;
}
bool CommandLineOption::ast_dump() {
    return g_CurrentOptions.ast_dump;
}
bool CommandLineOption::purge_destination_directories() {
    return g_CurrentOptions.purge_destination_directories;
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
    unsigned long maskValue = 1 << int(warning);
    return ((g_CurrentOptions.enabled_warnings & maskValue) == 0);
}

bool CommandLineOption::suppress_error(WarningType type) {
    unsigned long maskValue = 1 << int(type);
    return ((g_CurrentOptions.enabled_errors & maskValue) == 0);
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
