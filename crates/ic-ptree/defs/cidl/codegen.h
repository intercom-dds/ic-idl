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

#include <list>

#include "InterCOM/PlatformConfig.h"
#include "cidl/idl_parser.h"

namespace intercom {
namespace cidl {

struct Config {
    bool disable_header_follow = false;
    bool generate_typesupport_only = false;
    bool corba_types = false;
    bool no_rename = false;
    bool list_only = false;
    bool string_encoding_utf8 = false;
    bool proxies = false;
    bool compatibility = false;
    bool no_corba_dependency = false;
    bool cpp_gen_cpp11 = false;
    bool cpp_access_functions = false;
    bool doxy_compatible_output = false;
    bool expand_idl = false;
    bool legacy_idl = false;
    bool generate_default_literals = false;
    bool cpp_no_stream_op = false;
    bool no_defs = false;

    std::string cpp_header_postfix = "h";
    std::string c_file_prefix;
    std::string dll_exp_sym;
    std::string java_package_prefix;
    std::string ada_package_prefix;
    std::string header_subfolder;
};

struct File {
    /// Path of the file to be generated.
    std::string path;

    /// Contents of the file.
    std::string content;

    File(std::string path) : path(std::move(path)) {}

    File(std::string path, std::string&& content) : path(std::move(path)), content(std::move(content)) {}
};

inline bool operator<(const File& lhs, const File& rhs) {
    return lhs.path < rhs.path;
}

INTERCOM_PUBLIC std::list<File> code_gen_cs(const Config& config, parse_result* result);
INTERCOM_PUBLIC std::list<File> code_gen_dds_cplpl(const Config& config, parse_result* result);
INTERCOM_PUBLIC std::list<File> code_gen_java(const Config& config, parse_result* result);
INTERCOM_PUBLIC std::list<File> code_gen_python(const Config& config, parse_result* result);
INTERCOM_PUBLIC std::list<File> code_gen_idl(const Config& config, parse_result* result);
INTERCOM_PUBLIC std::list<File> code_gen_json_schema(const Config& config, parse_result* result);

}  // namespace cidl
}  // namespace intercom
