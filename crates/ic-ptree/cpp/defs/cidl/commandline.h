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

#include "cidl/codegen.h"

namespace intercom::cidl {

class CommandLineOption {
  public:
    static bool intercom_build();
    static bool generate_typesupport_only();
    static bool list_only();
    static bool no_rename();
    static bool proxies();
    static bool compatibility();
    static bool no_corba_dependency();
    static bool cpp_gen_cpp11();
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
    static const char* c_file_prefix();
    static const char* dll_exp_sym();
    static const char* cs_target_directory();
    static const char* c_target_directory();
    static const char* java_target_directory();
    static const char* java_package_prefix();
    static const char* json_target_directory();
    static const char* json_schema_target_directory();
    static const char* rust_target_directory();
    static const char* python_target_directory();
    static const char* python_global_postfix();
    static const char* ada_target_directory();
    static const char* idl_target_directory();
    static const char* xml_target_directory();
    static const char* proto_target_directory();
    static const char* toml_target_directory();
    static const char* ada_package_prefix();
    static const char* header_subfolder();
    static const char* copyright_notice();

    static intercom::cidl::Config& get_instance();
};

}  // namespace intercom::cidl
