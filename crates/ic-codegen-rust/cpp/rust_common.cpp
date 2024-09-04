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

#include "rust_common.h"

#include <string>
#include <string_view>

#include "cidl/commandline.h"
#include "cidl/constants.h"
#include "cidl/hdrs.h"
#include "cidl/ptree.h"
#include "cidl/ptree_helpers.h"
#include "cidl/symbols.h"
#include "icgen/template/casing.h"
#include "utils/string_utils.h"

using namespace intercom::cidl;

namespace intercom::rust {

static void qos_name(const ptree* node, std::string& name) {
    if (node->kind == N_ENUM || node->kind == N_STRUCT) {
        if (name == "PropertyQosPolicy") {
            name = "PropertyQos";
        } else {
            auto pos = name.rfind("QosPolicy");
            if (pos != std::string::npos) {
                name.erase(pos, 9);
            }
        }
    } else if (node->kind == N_CONST && node->super && node->super->kind == N_ENUM) {
        if (string_utils::ends_with(node->name, "_QOS")) {
            size_t pos = name.rfind('_', name.length() - 5);
            name.erase(pos);
        }
    }
}

std::string conv_name(const ptree* node, intercom::icgen::Case casing) {
    std::string name = node->name;

    // Strip 'QosPolicy' and '_XXX_QOS' suffixes from the QoS types
    if (CommandLineOption::intercom_build()) {
        qos_name(node, name);
    }

    // Strip "_t" and "_e" suffixes, then convert the name
    if (!CommandLineOption::no_rename()) {
        std::string lower = tolower(name);
        std::string_view view(lower);
        if (view.length() > 2 &&
            (string_utils::ends_with(view, "_t") || string_utils::ends_with(view, "_e"))) {
            name = name.substr(0, name.length() - 2);
        }
        intercom::icgen::CaseConverter conv(casing);
        name = conv.convert(name);
    }
    return safe_name(node, name, LANG_RUST);
}

std::string const_name(const ptree* node) {
    auto name = conv_name(node, intercom::icgen::Case::Snake);
    if (!CommandLineOption::no_rename()) {
        transform(name.begin(), name.end(), name.begin(), [](char c) {  //
            return std::toupper(c, std::locale());
        });
    }
    return name;
}

std::string fn_name(const ptree* node) {
    return conv_name(node, intercom::icgen::Case::Snake);
}

std::string mod_name(const ptree* node) {
    return conv_name(node, intercom::icgen::Case::Snake);
}

std::string type_name(const ptree* node) {
    return conv_name(node, intercom::icgen::Case::Pascal);
}

std::string member_name(const ptree* node) {
    return conv_name(node, intercom::icgen::Case::Snake);
}

std::string seri_name(const ptree* node) {
    if (auto rename = get_annotation(node, annotation_type_ext_rename)) {
        return string_value(get_annotation_value(rename, "name"));
    }
    return original_node(node)->name;
}

}  // namespace intercom::rust
