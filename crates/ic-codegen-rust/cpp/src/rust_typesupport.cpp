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

#ifndef CIDL_BOOTSTRAP

#  include "InterCOM/dds_xtypes_constants.h"
#  include "InterCOM/dds_xtypes_typeobject.h"
#  include "cidl/type_definition.h"

static void emit_type_id(Twine& out, const ptree* node) {
    auto lib = intercom::cidl::get_type_definition(node, [](const ptree*) { return true; });
    const auto& type_id = lib.type_info.complete.typeid_with_size.type_id;

    if (type_id._d() == intercom::dcps::xtypes::EK_COMPLETE) {
        out(prefix, "::xtypes::TypeIdentifier::EkComplete([\n");
        for (auto byte : type_id.equivalence_hash()) {
            out(fmt::format("{:#04x}, ", byte));
        }
        out("\n])");
    } else {
        out(prefix,
            "::xtypes::TypeIdentifier::ScComponentId(",
            prefix,
            "::xtypes::StronglyConnectedComponentId {\n");
        out("sc_component_id: ", prefix, "::xtypes::TypeObjectHashId::EkComplete([\n");
        for (auto byte : type_id.sc_component_id().sc_component_id.hash()) {
            out(fmt::format("{:#04x}, ", byte));
        }
        out("\n]),\n");
        out("scc_index: ", type_id.sc_component_id().scc_index, ",\n");
        out("scc_length: ", type_id.sc_component_id().scc_length, ",\n");
        out("})");
    }
}

static void emit_ts(Twine& out, const ptree* node) {
    if (is_non_serialized(node) || CommandLineOption::no_typesupport()) {
        return;
    }

    auto orig = original_node(node);
    auto lib = intercom::cidl::get_type_definition(orig, no_struct_or_enum_filter);
    const auto& type_id = lib.type_info.complete.typeid_with_size.type_id;

    intercom::dcps::Buffer buf(4000);
    auto flags = intercom::CDR_LITTLE_ENDIAN | intercom::CDR_XCDR_PLAIN;
    const auto& type_info = intercom::TypeTraits<intercom::dcps::xtypes::TypeDefinition>::type_info;
    intercom::dcps::cts::writeEncapsulation(buf, flags, type_info);
    intercom::dcps::cts::CdrWriter writer(buf, flags);
    intercom::dcps::cts::CdrMarshal(writer).io(lib);

    out("impl ", prefix, "::core::TypeTraits for ", node, " {\n");
    out("const TYPE_DEFINITION: &'static [u8] = b", begin("\""));
    for (intercom::ULong i = 0; i < buf.readable_length(); i++) {
        if (i % 22 == 0) {
            out("\\\n");
        }
        out(fmt::format("\\x{:02x}", buf.m_read_pointer[i]));
    }
    out(end("\""), ";\n\n");

    out("const TYPE_IDENTIFIER: ", prefix, "::xtypes::TypeIdentifier =", begin(""), "\n");
    emit_type_id(out, orig);
    out(end(""), ";\n\n");

    out("fn register_type(repo: &", prefix, "::xtypes::TypeRepository)");
    out(" -> ", prefix, "::core::Result<()> {\n");
    auto has_type = type_id._d() == intercom::dcps::xtypes::TI_STRONGLY_CONNECTED_COMPONENT
                        ? "has_type"
                        : "has_complete_type";
    out("if !repo.", has_type, "(&Self::TYPE_IDENTIFIER) {\n");
    out("repo.register_serialized_type(Self::TYPE_DEFINITION)?;\n");
    auto order = ptree_build_order(node);
    for (const auto& group : order) {
        for (auto obj : group) {
            if (obj != node && !no_struct_or_enum_filter(obj) && is_emit(obj, LANG_RUST)) {
                out(rust_type(obj, node), "::register_type(repo)?;\n");
            }
        }
    }
    out("}\n");
    out("Ok(())\n");
    out("}\n");

    out("}\n\n");

    out("impl ", prefix, "::core::TypeSupport for ", node, " {\n");

    out("fn type_identifier(&self) -> ", prefix, "::xtypes::TypeIdentifier {\n");
    out("<Self as ", prefix, "::core::TypeTraits>::TYPE_IDENTIFIER\n");
    out("}\n");

    out("fn type_definition(&self) -> ", prefix, "::xtypes::TypeDefinition{\n");
    out(cts_prefix, "::cdr::from_le_bytes(\n");
    out("&<Self as ", prefix, "::core::TypeTraits>::TYPE_DEFINITION[4..]).unwrap_or_default()\n");
    out("}\n");

    out("fn create_data(&self) -> Self {\n");
    out("Self::default()\n");
    out("}\n\n");

    out("}\n\n");
}

#else

static void emit_ts(Twine&, const ptree*) {}

#endif
