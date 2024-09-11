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

#include <string>
#include <vector>

namespace intercom::cidl {

struct Config {
    bool generate_typesupport_only = false;
    bool corba_types = false;
    bool no_rename = false;
    bool proxies = false;
    bool compatibility = false;
    bool no_corba_dependency = false;
    bool cpp_access_functions = false;
    bool cpp_scoped_enums = false;
    bool doxy_compatible_output = false;
    bool expand_idl = false;
    bool legacy_idl = false;
    bool generate_default_literals = false;
    bool cpp_no_stream_op = false;
    bool intercom_build = false;
    bool do_suppress_deprecated = false;
    bool use_fmtlib = false;
    bool use_wstring = false;
    bool no_typesupport = false;

    std::string cpp_header_postfix = "h";
    std::string dll_exp_sym;
    std::string java_package_prefix;
    std::string ada_package_prefix;
    std::string header_subfolder;
    std::string copyright_notice;
    std::string python_global_postfix;
    std::vector<std::string> input_list;
    std::vector<std::string> include_directories{"."};
    std::vector<std::string> pp_parameters;
};

class CommandLineOption {
  public:
    static bool intercom_build();
    static bool generate_typesupport_only();
    static bool no_rename();
    static bool proxies();
    static bool compatibility();
    static bool no_corba_dependency();
    static bool cpp_scoped_enums();
    static bool cpp_no_stream_op();
    static bool cpp_access_functions();
    static bool doxy_compatible_output();
    static bool expand_idl();
    static bool legacy_idl();
    static bool generate_default_literals();
    static bool use_fmtlib();
    static bool corba_types();
    static bool use_wstring();
    static bool no_typesupport();

    static const char* cpp_header_postfix();
    static const char* dll_exp_sym();
    static const char* java_package_prefix();
    static const char* python_global_postfix();
    static const char* ada_package_prefix();
    static const char* header_subfolder();
    static const char* copyright_notice();

    static Config& get_instance();
};

}  // namespace intercom::cidl
