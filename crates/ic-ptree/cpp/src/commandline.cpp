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

using namespace intercom::cidl;

static thread_local Config g_current_options;

static const char* strptr_or_null(const std::string& value) {
    if (value.empty()) {
        return nullptr;
    }
    return value.c_str();
}

bool CommandLineOption::intercom_build() {
    return g_current_options.intercom_build;
}
bool CommandLineOption::generate_typesupport_only() {
    return g_current_options.generate_typesupport_only;
}
bool CommandLineOption::list_only() {
    return g_current_options.list_only;
}
bool CommandLineOption::no_rename() {
    return g_current_options.no_rename;
}
bool CommandLineOption::proxies() {
    return g_current_options.proxies;
}
bool CommandLineOption::compatibility() {
    return g_current_options.compatibility;
}
bool CommandLineOption::no_corba_dependency() {
    return g_current_options.no_corba_dependency;
}
bool CommandLineOption::cpp_scoped_enums() {
    return g_current_options.cpp_scoped_enums;
}
bool CommandLineOption::cpp_no_stream_op() {
    return g_current_options.cpp_no_stream_op;
}
bool CommandLineOption::cpp_access_functions() {
    return g_current_options.cpp_access_functions;
}
bool CommandLineOption::doxy_compatible_output() {
    return g_current_options.doxy_compatible_output;
}
bool CommandLineOption::expand_idl() {
    return g_current_options.expand_idl;
}
bool CommandLineOption::legacy_idl() {
    return g_current_options.legacy_idl;
}
bool CommandLineOption::generate_default_literals() {
    return g_current_options.generate_default_literals;
}
bool CommandLineOption::use_fmtlib() {
    return g_current_options.use_fmtlib;
}
bool CommandLineOption::no_typesupport() {
    return g_current_options.no_typesupport;
}
bool CommandLineOption::corba_types() {
    return g_current_options.corba_types;
}
bool CommandLineOption::use_wstring() {
    return g_current_options.use_wstring;
}
const char* CommandLineOption::cpp_header_postfix() {
    return strptr_or_null(g_current_options.cpp_header_postfix);
}
const char* CommandLineOption::c_file_prefix() {
    return strptr_or_null(g_current_options.c_file_prefix);
}
const char* CommandLineOption::dll_exp_sym() {
    return strptr_or_null(g_current_options.dll_exp_sym);
}
const char* CommandLineOption::cs_target_directory() {
    return strptr_or_null(g_current_options.cs_target_directory);
}
const char* CommandLineOption::c_target_directory() {
    return strptr_or_null(g_current_options.c_target_directory);
}
const char* CommandLineOption::java_target_directory() {
    return strptr_or_null(g_current_options.java_target_directory);
}
const char* CommandLineOption::java_package_prefix() {
    return strptr_or_null(g_current_options.java_package_prefix);
}
const char* CommandLineOption::json_target_directory() {
    return strptr_or_null(g_current_options.json_target_directory);
}
const char* CommandLineOption::json_schema_target_directory() {
    return strptr_or_null(g_current_options.json_schema_target_directory);
}
const char* CommandLineOption::rust_target_directory() {
    return strptr_or_null(g_current_options.rust_target_directory);
}
const char* CommandLineOption::python_target_directory() {
    return strptr_or_null(g_current_options.python_target_directory);
}
const char* CommandLineOption::python_global_postfix() {
    return strptr_or_null(g_current_options.python_global_postfix);
}
const char* CommandLineOption::ada_target_directory() {
    return strptr_or_null(g_current_options.ada_target_directory);
}
const char* CommandLineOption::idl_target_directory() {
    return strptr_or_null(g_current_options.idl_target_directory);
}
const char* CommandLineOption::xml_target_directory() {
    return strptr_or_null(g_current_options.xml_target_directory);
}
const char* CommandLineOption::proto_target_directory() {
    return strptr_or_null(g_current_options.proto_target_directory);
}
const char* CommandLineOption::toml_target_directory() {
    return strptr_or_null(g_current_options.toml_target_directory);
}
const char* CommandLineOption::ada_package_prefix() {
    return strptr_or_null(g_current_options.ada_package_prefix);
}
const char* CommandLineOption::header_subfolder() {
    return strptr_or_null(g_current_options.header_subfolder);
}
const char* CommandLineOption::copyright_notice() {
    return strptr_or_null(g_current_options.copyright_notice);
}

intercom::cidl::Config& CommandLineOption::get_instance() {
    return g_current_options;
}
