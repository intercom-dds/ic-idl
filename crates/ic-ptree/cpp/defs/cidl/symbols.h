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

#include "cidl/constants.h"

namespace intercom::cidl {

enum Language {
    LANG_CPP,
    LANG_CS,
    LANG_JAVA,
    LANG_ADA,
    LANG_PYTHON,
    LANG_IDL,
    LANG_RUST,
    LANG_PROTO,
    LANG_NONE,
};

INTERCOM_PUBLIC std::string safe_name(const ptree* node, const std::string& name, Language lang);
INTERCOM_PUBLIC std::string cpp_name(const ptree* node);
INTERCOM_PUBLIC std::string java_name(const ptree* node);
INTERCOM_PUBLIC std::string ada_name(const ptree* node);
INTERCOM_PUBLIC std::string cs_name(const ptree* node);
INTERCOM_PUBLIC std::string python_name(const ptree* node);
INTERCOM_PUBLIC std::string idl_name(const ptree* node);
INTERCOM_PUBLIC std::string rust_name(const ptree*);
INTERCOM_PUBLIC std::string proto_name(const ptree*);

INTERCOM_PUBLIC const ptree* common_scope(const ptree* node, const ptree* context);
INTERCOM_PUBLIC const ptree* namespace_of(const ptree* node);
INTERCOM_PUBLIC std::string module_name(const ptree* node);

INTERCOM_PUBLIC uint32_t member_name_hash_id(const std::string& name);

/// \note skips first enum or bitmask scope
/// \details Cidl understands mod::EnumType::VALUE, but should not use enum scopes in emitted idl.
/// i.e., mod::VALUE should be emitted instead of mod::EnumType::VALUE. (OMG IDL-4.2 is limited to
/// C's enum paradigm)
INTERCOM_PUBLIC std::string idl_scoped_name(const ptree* node, const ptree* context);
/// same as idl_scoped_name, but does not skip scopes
/// \note do not use in idl output
INTERCOM_PUBLIC std::string idl_internal_scoped_name(const ptree* node, const ptree* context);
/// \note do not use in idl output
INTERCOM_PUBLIC std::string lc_scoped_name(const ptree* p);

}  // namespace intercom::cidl
