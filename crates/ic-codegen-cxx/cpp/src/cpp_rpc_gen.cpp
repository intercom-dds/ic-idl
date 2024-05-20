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

#include "cidl/internal/commandline.h"
#include "cidl/internal/hdrs.h"
#include "cidl/internal/idl_rpc_gen.h"
#include "cidl/internal/ptree_builder.h"
#include "cidl/pretty_printer.h"
#include "cidl/ptree_helpers.h"

using namespace intercom::cidl;

namespace {
std::string cpp11_rpc_namespace() {
    if (CommandLineOption::cpp_gen_cpp11()) {
        return "dds::rpc::";
    }
    return "intercom::dcps::rpc::";
}

std::string member_accessor(std::string name) {
    if (CommandLineOption::cpp_access_functions()) {
        name += "()";
    }
    return name;
}

std::string tolower(std::string res) {
    transform(res.begin(), res.end(), res.begin(), ::tolower);
    return res;
}

PrettyPrinter& dll_exp(PrettyPrinter& out) {
    if (CommandLineOption::dll_exp_sym()) {
        out << CommandLineOption::dll_exp_sym() << " ";
    }
    return out;
}

int is_pointer_type(const ptree* elem) {
    return (is_shared(elem) || base_type_of(elem)->kind == N_INTERFACE);
}

int is_pass_by_value(const ptree* elem) {
    return
        // Pass pointers and primitive types by value
        (is_pointer_type(elem) || is_primitive(base_type_of(elem)) ||
         (base_type_of(elem)->kind == N_ENUM) || (base_type_of(elem)->kind == N_BITMASK)) &&
        // ...but not if they are optional templates
        !is_optional(elem);
}

std::string cpp_arg_type(const ptree* elem, const ptree* context, unsigned flag_mask = 0) {
    std::stringstream out;
    unsigned int flags = elem->flags & ~flag_mask;
    if (is_pass_by_value(elem)) {
        out << cpp_type_name(elem->type, context);
        if (flags & OPT_OUT) {
            out << "&";
        }
    } else {
        if ((flags & OPT_OUT) == 0) {
            out << "const ";
        }
        out << cpp_type_name(elem->type, context) << "&";
    }
    return out.str();
}

int argument_count(ptree* args, ptree_opts flags) {
    int count = 0;
    while (args) {
        if (args->flags & flags) {
            ++count;
        }
        args = args->next;
    }
    return count;
}

int is_oneway(const ptree* node) {
    const ptree* ann = get_annotation(node, annotation_type_oneway);
    return ann && integer_value(ann->members->value) != 0 &&
           argument_count(node->members, OPT_OUT) == 0 && node->type == nullptr;
}

void gen_promise(const ptree* a_node, PrettyPrinter& a_head) {
    const ptree* context = a_node->state->lookup_node("DDS::RPC");
    std::vector<rpc_operation> operations = get_rpc_operations_non_recursive(a_node);
    for (auto op : operations) {
        a_head << blank_line;
        a_head << "template <>" << endl;
        a_head << "class promise<" << cpp_type_name(op.out_type, nullptr)
               << "> : public promise_base" << endl;
        a_head.begin("{");
        a_head << endl;

        a_head << unindent << "public:" << endl;
        a_head
            << "explicit promise(const TypeSupport* a_type_support) : m_type_support(a_type_support) {}";
        a_head << blank_line;

        a_head << blank_line;
        a_head << "void set_value(const OctetSeq& a_data, const SampleInfo&) override ";
        a_head << begin("{") << endl;
        a_head << "try " << begin("{") << endl;
        a_head << cpp_type_name(get_rpc_reply_node(a_node), nullptr) << " sample;" << endl;
        a_head << "m_type_support->fr_cdr(&sample, a_data, false);" << endl;
        a_head << "if (sample." << member_accessor("header")
               << ".remote_ex == REMOTE_EX_OK && sample." << member_accessor("reply")
               << "._d() == " << cpp_type_name(op.hash_const, nullptr) << ") " << begin("{")
               << endl;
        a_head << cpp_type_name(op.result_type, nullptr) << "& result = sample."
               << member_accessor("reply") << "." << cpp_name(op.prototype) << "();" << endl;
        a_head << "switch (result._d()) " << begin("{") << endl;
        for (auto result : op.result_type->members) {
            if (result->kind == N_MEMBER) {
                if (result->flags & OPT_DEFAULT) {
                    a_head << unindent << "default:" << endl;
                } else {
                    for (auto cas : result->members) {
                        a_head << unindent << "case "
                               << cpp_type_name(cas->value.val.node(), context) << ":" << endl;
                    }
                }
                if (result->next == nullptr) {
                    a_head << "detail::throw_remote_exception(REMOTE_EX_INVALID_ARGUMENT);" << endl;
                    a_head << "break;" << endl;
                }
                if (result->name == "result") {
                    a_head << "m_promise.set_value(result.result());" << endl;
                } else {
                    a_head << "throw result." << cpp_name(result) << "();" << endl;
                }
                a_head << "break;" << endl;
            }
        }
        a_head << end("}") << endl;
        a_head << end("}") << " else " << begin("{") << endl;
        a_head << "detail::throw_remote_exception(sample." << member_accessor("header")
               << ".remote_ex);" << endl;
        a_head << end("}") << endl;
        a_head << end("}") << " catch (...) " << begin("{") << endl;
        a_head << "try { m_promise.set_exception(std::current_exception()); }" << endl;
        a_head << "catch (...) {}" << endl;
        a_head << end("}") << endl;
        a_head << end("}") << endl;

        a_head << blank_line;
        a_head << "void set_exception(RemoteExceptionCode_t ex) override " << begin("{") << endl;
        a_head << "detail::set_remote_exception(m_promise, ex);" << endl;
        a_head << end("}") << endl;

        a_head << blank_line;
        a_head << "future<" << cpp_type_name(op.out_type, nullptr) << "> get_future() "
               << begin("{") << endl;
        a_head << "return m_promise.get_future();" << endl;
        a_head << end("}") << endl;

        a_head << blank_line;
        a_head << unindent << "private:" << endl;
        a_head << "detail::promise<" << cpp_type_name(op.out_type, nullptr) << "> m_promise;"
               << endl;
        a_head << "const TypeSupport* m_type_support;" << endl;
        a_head << end("}") << ";" << endl;
    }
};

void gen_client_sync(const ptree* a_node, PrettyPrinter& a_head, PrettyPrinter& a_body) {
    std::vector<rpc_operation> operations = get_rpc_operations(a_node);

    a_head << blank_line;
    a_head << "class " << cpp_name(a_node) << " : ";
    a_head << "public ::" << cpp11_rpc_namespace() << "ClientEndpoint<"
           << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
           << cpp_type_name(get_rpc_reply_node(a_node), a_node) << "> ";
    a_head << begin("{") << endl;
    a_head << tab_group;
    a_head << unindent << "public:" << endl;
    a_head << cpp_name(a_node) << "(const ::" << cpp11_rpc_namespace() << "ClientParams& params);"
           << endl;
    a_head << blank_line;
    for (auto op : operations) {
        const ptree* prot = op.prototype;
        a_head << dll_exp << (prot->type ? cpp_type_name(prot->type, a_node) : "void") << " "
               << cpp_name(prot);
        a_head.begin("(");
        for (auto el : prot->members) {
            a_head << list_sep << cpp_arg_type(el, a_node) << " " << cpp_name(el);
        }
        a_head.end(");");
        a_head << endl;
    }
    a_head << tab_group;
    a_head.end("};");
    a_head << endl;

    a_body << blank_line;
    a_body << cpp_type_name(a_node, nullptr) << "::" << cpp_name(a_node)
           << "(const ::" << cpp11_rpc_namespace() << "ClientParams& params) : ClientEndpoint<"
           << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
           << cpp_type_name(get_rpc_reply_node(a_node), a_node) << ">" << "(params)" << endl;
    a_body << "{" << endl;
    a_body << "}" << endl;
    a_body << blank_line;

    for (auto op : operations) {
        const ptree* prot = op.prototype;
        a_body << blank_line;
        a_body << (prot->type ? cpp_type_name(prot->type, nullptr) : "void") << " "
               << cpp_type_name(a_node, nullptr) << "::" << cpp_name(prot);
        a_body.begin("(");
        for (auto el : prot->members) {
            a_body << list_sep << cpp_arg_type(el, nullptr) << " " << cpp_name(el);
        }
        a_body.end(")");
        a_body << endl;
        a_body.begin("{");
        a_body << endl;
        a_body << cpp_type_name(op.in_type, a_node) << " intercom_in_arg_";
        if (argument_count(prot->members, OPT_IN) > 0) {
            a_body.begin("(");
            for (auto el : prot->members) {
                if (el->flags & OPT_IN) {
                    a_body << list_sep << cpp_name(el);
                }
            }
            a_body.end(")");
        }
        a_body << ";" << endl;
        a_body << cpp_type_name(get_rpc_request_node(a_node), a_node) << " intercom_request_arg_;"
               << endl;
        a_body << "intercom_request_arg_." << member_accessor("data") << "."
               << cpp_name(op.prototype) << "(intercom_in_arg_);" << endl;
        if (is_oneway(op.prototype)) {
            a_body << "ClientEndpoint<" << cpp_type_name(get_rpc_request_node(a_node), a_node)
                   << ", " << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                   << " >::m_requester.send_request(intercom_request_arg_);" << endl;
        } else {
            a_body << "::intercom::dcps::rpc::future<" << cpp_type_name(op.out_type, a_node) << "> "
                   << endl
                   << "   intercom_out_future_ = ClientEndpoint<"
                   << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
                   << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                   << " >::m_requester.send_request_async<" << cpp_type_name(op.out_type, a_node)
                   << ">" << "(intercom_request_arg_);" << endl;
            if (CommandLineOption::cpp_gen_cpp11()) {
                a_body << "dds::core::Duration intercom_max_wait_ = ClientEndpoint<"
                       << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
                       << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                       << ">::get_max_service_wait();" << endl;
                a_body << "if (intercom_max_wait_ != dds::core::Duration::infinite() &&" << endl
                       << "     intercom_out_future_.wait_for(intercom_max_wait_.to_duration()) != "
                          "std::future_status::ready)"
                       << endl;
            } else {
                a_body << "::intercom::dcps::Duration_t intercom_max_wait_ = ClientEndpoint<"
                       << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
                       << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                       << " >::get_max_service_wait();" << endl;
                a_body
                    << "if (intercom_max_wait_ != ::intercom::dcps::DURATION_INFINITE &&" << endl
                    << "    intercom_out_future_.wait_for(std::chrono::seconds(intercom_max_wait_.sec) +"
                    << endl
                    << "                                    std::chrono::nanoseconds(intercom_max_wait_.nanosec) "
                       ") != std::future_status::ready)"
                    << endl;
            }

            a_body.begin("{");
            a_body << endl;
            a_body << "throw ::intercom::dcps::rpc::RemoteNoService();" << endl;
            a_body.end("}");
            a_body << endl;
            if (argument_count(prot->members, OPT_OUT) > 0 || prot->type) {
                a_body << cpp_type_name(op.out_type, a_node)
                       << " intercom_out_arg_ = intercom_out_future_.get();" << endl;
                for (auto el : prot->members) {
                    if (el->flags & OPT_OUT) {
                        a_body << cpp_name(el) << " = intercom_out_arg_."
                               << member_accessor(cpp_name(el)) << ";" << endl;
                    }
                }
                if (prot->type) {
                    a_body << "return intercom_out_arg_." << member_accessor("return_") << ";"
                           << endl;
                }
            } else {
                a_body << "intercom_out_future_.get();" << endl;
            }
        }

        a_body.end("}");
        a_body << endl;
    }
}

void gen_client_async(const ptree* a_node, PrettyPrinter& a_head, PrettyPrinter& a_body) {
    std::vector<rpc_operation> operations = get_rpc_operations(a_node);

    a_head << blank_line;
    a_head << "class " << cpp_name(a_node) << "Async : ";
    a_head << "public ::" << cpp11_rpc_namespace() << "ClientEndpoint<"
           << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
           << cpp_type_name(get_rpc_reply_node(a_node), a_node) << "> ";
    a_head << begin("{") << endl;
    a_head << tab_group;
    a_head << unindent << "public:" << endl;
    a_head << cpp_name(a_node) << "Async(const ::" << cpp11_rpc_namespace()
           << "ClientParams& params);" << endl;
    a_head << blank_line;
    for (auto op : operations) {
        const ptree* prot = op.prototype;
        a_head << dll_exp;
        if (is_oneway(prot)) {
            a_head << "void ";
        } else if (argument_count(prot->members, OPT_OUT) == 0 && prot->type == nullptr) {
            a_head << "intercom::dcps::rpc::future<void> ";
        } else {
            a_head << "intercom::dcps::rpc::future<" << cpp_type_name(op.out_type, a_node) << "> ";
        }
        a_head << cpp_name(prot);
        a_head.begin("(");
        for (auto el : prot->members) {
            if (el->flags & OPT_IN) {
                a_head << list_sep << cpp_arg_type(el, a_node, OPT_OUT) << " " << cpp_name(el);
            }
        }
        a_head.end(");");
        a_head << endl;
    }
    a_head.end("};");
    a_head << endl;

    a_body << blank_line;
    a_body << cpp_type_name(a_node, nullptr) << "Async::" << cpp_name(a_node)
           << "Async(const ::" << cpp11_rpc_namespace() << "ClientParams& params) : ClientEndpoint<"
           << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
           << cpp_type_name(get_rpc_reply_node(a_node), a_node) << ">" << "(params)" << "{}"
           << endl;
    a_body << blank_line;

    for (auto op : operations) {
        const ptree* prot = op.prototype;
        a_body << blank_line;
        if (is_oneway(op.prototype)) {
            a_body << "void ";
        } else if (argument_count(op.prototype->members, OPT_OUT) == 0 &&
                   op.prototype->type == nullptr) {
            a_body << "intercom::dcps::rpc::future<void>" << endl;
        } else {
            a_body << "intercom::dcps::rpc::future<" << cpp_type_name(op.out_type, nullptr) << ">"
                   << endl;
        }
        a_body << cpp_type_name(a_node, nullptr) << "Async::" << cpp_name(prot);
        a_body.begin("(");
        for (auto el : prot->members) {
            if (el->flags & OPT_IN) {
                a_body << list_sep << cpp_arg_type(el, nullptr, OPT_OUT) << " " << cpp_name(el);
            }
        }
        a_body.end(")");
        a_body << endl;
        a_body.begin("{");
        a_body << endl;
        a_body << cpp_type_name(op.in_type, a_node) << " intercom_in_arg_";
        if (argument_count(prot->members, OPT_IN) > 0) {
            a_body.begin("(");
            for (auto el : prot->members) {
                if (el->flags & OPT_IN) {
                    a_body << list_sep << cpp_name(el);
                }
            }
            a_body.end(")");
        }
        a_body << ";" << endl;
        a_body << cpp_type_name(get_rpc_request_node(a_node), a_node) << " intercom_request_arg_;"
               << endl;
        a_body << "intercom_request_arg_." << member_accessor("data ") << "."
               << cpp_name(op.prototype) << "(intercom_in_arg_);" << endl;
        if (is_oneway(op.prototype)) {
            a_body << "ClientEndpoint<" << cpp_type_name(get_rpc_request_node(a_node), a_node)
                   << ", " << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                   << " >::m_requester.send_request( intercom_request_arg_);" << endl;
        } else if (argument_count(op.prototype->members, OPT_OUT) == 0 &&
                   op.prototype->type == nullptr) {
            a_body << "return ClientEndpoint<"
                   << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
                   << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                   << " >::m_requester.send_request_async< void >" << "(intercom_request_arg_);"
                   << endl;
        } else {
            a_body << "return ClientEndpoint<"
                   << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
                   << cpp_type_name(get_rpc_reply_node(a_node), a_node)
                   << " >::m_requester.send_request_async< " << cpp_type_name(op.out_type, a_node)
                   << " >" << "(intercom_request_arg_);" << endl;
        }
        a_body.end("}");
        a_body << endl;
    }
}

void parent_request_reply_topics(
    const std::vector<ptree*>& nodes,
    std::vector<std::string>& request,
    std::vector<std::string>& reply
) {
    for (auto node : nodes) {
        parent_request_reply_topics(node->parents, request, reply);
        request.push_back(rpc_request_topic_name(node));
        reply.push_back(rpc_reply_topic_name(node));
    }
}

void gen_service(const ptree* a_node, PrettyPrinter& a_head, PrettyPrinter& a_body) {
    std::vector<rpc_operation> operations = get_rpc_operations(a_node);

    a_head << blank_line;
    a_head << "using " << cpp_name(a_node) << "Service = " << "::" << cpp11_rpc_namespace()
           << "ServiceEndpoint<" << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
           << cpp_type_name(get_rpc_reply_node(a_node), a_node) << ">;" << endl;
    a_head << blank_line;
    a_head << "class " << cpp_name(a_node) << "Support : ";
    a_head << "public ::" << cpp11_rpc_namespace() << "SimpleReplierListener<"
           << cpp_type_name(get_rpc_request_node(a_node), a_node) << ", "
           << cpp_type_name(get_rpc_reply_node(a_node), a_node) << "> ";
    a_head << begin("{") << endl;
    a_head << tab_group;
    a_head << unindent << "public:" << endl;

    if (!a_node->parents.empty()) {
        std::vector<std::string> request;
        std::vector<std::string> reply;
        parent_request_reply_topics(a_node->parents, request, reply);
        a_head << "::" << cpp11_rpc_namespace() << "ServiceParams default_service_params() override"
               << endl;
        a_head.begin("{");
        a_head << endl;
        a_head << tab_group;
        a_head << "::" << cpp11_rpc_namespace()
               << "ServiceParams params = SimpleReplierListener::default_service_params();" << endl;
        for (const auto& req : request) {
            a_head << "params.request_topic_aliases().emplace_back(\"" << req << "\");" << endl;
        }
        for (const auto& rep : reply) {
            a_head << "params.reply_topic_aliases().emplace_back(\"" << rep << "\");" << endl;
        }
        a_head << "return params;" << endl;
        a_head.end("};");
        a_head << endl;
    }

    if (CommandLineOption::cpp_gen_cpp11()) {
        a_head << dll_exp << cpp_type_name(get_rpc_reply_node(a_node), a_node)
               << " process_request(const dds::sub::Sample<"
               << cpp_type_name(get_rpc_request_node(a_node), a_node)
               << ">& request, const intercom::dcps::SampleIdentity_t& request_identity) override;"
               << endl;
    } else {
        a_head << dll_exp << cpp_type_name(get_rpc_reply_node(a_node), a_node)
               << " process_request(const intercom::dcps::rpc::Sample<"
               << cpp_type_name(get_rpc_request_node(a_node), a_node)
               << ">& request, const intercom::dcps::SampleIdentity_t& request_identity) override;"
               << endl;
    }

    for (auto op : operations) {
        const ptree* prot = op.prototype;
        a_head << "virtual " << (prot->type ? cpp_type_name(prot->type, a_node) : "void") << " "
               << cpp_name(prot);
        a_head.begin("(");
        for (auto el : prot->members) {
            a_head << list_sep << cpp_arg_type(el, a_node) << " " << cpp_name(el);
        }
        a_head.end(") = 0;");
        a_head << endl;
    }
    a_head << tab_group;
    a_head << endl;
    a_head.end("};");
    a_head << endl;

    a_body << blank_line;

    if (CommandLineOption::cpp_gen_cpp11()) {
        a_body << cpp_type_name(get_rpc_reply_node(a_node), nullptr) << endl
               << cpp_type_name(a_node, nullptr) << "Support::process_request(" << endl
               << "   const dds::sub::Sample<"
               << cpp_type_name(get_rpc_request_node(a_node), nullptr) << " >& request," << endl
               << "   const ::intercom::dcps::SampleIdentity_t& request_identity)" << endl;

    } else {
        a_body << cpp_type_name(get_rpc_reply_node(a_node), nullptr) << endl
               << cpp_type_name(a_node, nullptr) << "Support::process_request(" << endl
               << "   const ::intercom::dcps::rpc::Sample<"
               << cpp_type_name(get_rpc_request_node(a_node), nullptr) << " >& request," << endl
               << "   const ::intercom::dcps::SampleIdentity_t& request_identity)" << endl;
    }
    a_body.begin("{");
    a_body << endl;
    a_body << cpp_type_name(get_rpc_reply_node(a_node), a_node) << " reply;" << endl;
    a_body << "reply." << member_accessor("header") << ".related_request_id = request_identity;"
           << endl;
    a_body << "reply." << member_accessor("header")
           << ".remote_ex = ::intercom::dcps::rpc::REMOTE_EX_OK;" << endl;
    a_body << "try" << endl;
    a_body.begin("{");
    a_body << endl;
    a_body << "switch (request.data()." << member_accessor("data") << "._d()) " << endl;
    a_body.begin("{");
    a_body << endl;
    for (auto op : operations) {
        a_body << unindent << "case " << cpp_type_name(op.hash_const, a_node) << ":" << endl;
        a_body.begin("{");
        a_body << endl;
        a_body << cpp_type_name(op.result_type, a_node) << " result;" << endl;
        if (!op.prototype->getraises.empty()) {
            a_body << "try" << endl;
            a_body.begin("{");
            a_body << endl;
        }
        if (argument_count(op.prototype->members, OPT_IN) > 0) {
            a_body << "const " << cpp_type_name(op.in_type, a_node) << "& in_arg = request.data()."
                   << member_accessor("data") << "." << cpp_name(op.prototype) << "();" << endl;
        }
        if (!is_oneway(op.prototype) || argument_count(op.prototype->members, OPT_OUT) > 0) {
            a_body << cpp_type_name(op.out_type, a_node) << " out_arg;" << endl;
        }
        for (auto el : op.prototype->members) {
            if ((el->flags & OPT_INOUT) == OPT_INOUT) {
                a_body << "out_arg." << member_accessor(cpp_name(el)) << " = in_arg."
                       << member_accessor(cpp_name(el)) << ";" << endl;
            }
        }
        if (op.prototype->type) {
            a_body << "out_arg." << member_accessor("return_") << " = ";
        }
        a_body << cpp_name(op.prototype);
        a_body.begin("(");
        for (auto el : op.prototype->members) {
            if (el->flags & OPT_OUT) {
                a_body << list_sep << "out_arg." << member_accessor(cpp_name(el));
            } else {
                a_body << list_sep << "in_arg." << member_accessor(cpp_name(el));
            }
        }
        a_body.end(");");
        a_body << endl;
        if (is_oneway(op.prototype)) {
            a_body.end("}");
            a_body << endl;
            a_body << "reply = " << cpp_type_name(get_rpc_reply_node(a_node), a_node) << "();"
                   << endl;
            a_body << "break;" << endl;
        } else {
            a_body << "result.result(out_arg);" << endl;

            if (!op.prototype->getraises.empty()) {
                a_body.end("}");
                a_body << endl;
                for (auto getraise : op.prototype->getraises) {
                    std::string name = tolower(cpp_name(getraise)) + "_ex";
                    a_body << "catch (const " << cpp_type_name(getraise, a_node)
                           << "& ex) { result." << name << "(ex); }" << endl;
                }
            }
            a_body << "reply." << member_accessor("reply") << "." << cpp_name(op.prototype)
                   << "(result);" << endl;
            a_body.end("}");
            a_body << endl;
            a_body << "break;" << endl;
        }
    }
    a_body << unindent << "default:" << endl;
    a_body << "throw ::intercom::dcps::rpc::RemoteUnknownOperationError();" << endl;
    a_body.end("}");
    a_body << endl;
    a_body.end("}");
    a_body << endl;
    a_body << "catch (const ::intercom::dcps::rpc::RemoteUnsupportedError&) { reply."
           << member_accessor("header")
           << ".remote_ex = ::intercom::dcps::rpc::REMOTE_EX_UNSUPPORTED; }" << endl;
    a_body << "catch (const ::intercom::dcps::rpc::RemoteInvalidArgumentError&) { reply."
           << member_accessor("header")
           << ".remote_ex = ::intercom::dcps::rpc::REMOTE_EX_INVALID_ARGUMENT; }" << endl;
    a_body << "catch (const ::intercom::dcps::rpc::RemoteOutOfResourcesError&) { reply."
           << member_accessor("header")
           << ".remote_ex = ::intercom::dcps::rpc::REMOTE_EX_OUT_OF_RESOURCES; }" << endl;
    a_body << "catch (const ::intercom::dcps::rpc::RemoteUnknownOperationError&) { reply."
           << member_accessor("header")
           << ".remote_ex = ::intercom::dcps::rpc::REMOTE_EX_UNKNOWN_OPERATION; }" << endl;
    a_body << "catch (...) { reply." << member_accessor("header")
           << ".remote_ex = ::intercom::dcps::rpc::REMOTE_EX_UNKNOWN_EXCEPTION; }" << endl;
    a_body << "return reply;" << endl;
    a_body.end("}");
    a_body << endl;
}

bool has_rpc_service(const ptree* module, const ptree* current_include) {
    while (module) {
        if (module->included_from == current_include && is_rpc_service(module)) {
            return true;
        }
        if (has_rpc_service(module->members, current_include)) {
            return true;
        }
        module = module->next;
    }
    return false;
}

void gen_promises(const ptree* module, PrettyPrinter& head, const ptree* current_include) {
    while (module) {
        if (module->included_from == current_include && is_rpc_service(module) &&
            !is_oneway(module)) {
            gen_promise(module, head);
        }
        gen_promises(module->members, head, current_include);
        module = module->next;
    }
}

void gen_services(
    const ptree* module,
    PrettyPrinter& head,
    PrettyPrinter& body,
    const ptree* current_include
) {
    while (module) {
        if (module->kind == N_MODULE && has_rpc_service(module, current_include)) {
            if (idl_scoped_name(module, nullptr) == "DDS") {
                head << "namespace intercom {" << endl;
                head << "namespace dcps {" << endl;
                gen_services(module->members, head, body, current_include);
                head << "}" << endl;
                head << "}" << endl;
            } else if (idl_scoped_name(module, nullptr) == "DDS::XTypes") {
                head << "namespace xtypes {" << endl;
                gen_services(module->members, head, body, current_include);
                head << "}" << endl;
            } else {
                head << "namespace " << cpp_name(module) << " {" << endl;
                gen_services(module->members, head, body, current_include);
                head << "}" << endl;
            }
        } else if (module->included_from == current_include && is_rpc_service(module)) {
            gen_client_sync(module, head, body);
            gen_client_async(module, head, body);
            gen_service(module, head, body);
        }
        module = module->next;
    }
}
}  // namespace

void cpl_rpc_service_gen(
    const ptree* a_node,
    struct memf* a_memf_head,
    struct memf* a_memf_body,
    const ptree* current_include
) {
    PrettyPrinter head;
    PrettyPrinter body;
    if (has_rpc_service(a_node, current_include)) {
        head << "namespace intercom {" << endl;
        head << "namespace dcps {" << endl;
        head << "namespace rpc {" << endl;

        gen_promises(a_node, head, current_include);

        head << "} // namespace rpc" << endl;
        head << "} // namespace dcps" << endl;
        head << "} // namespace intercom" << endl;

        gen_services(a_node, head, body, current_include);

        std::stringstream head_out;
        head.print(head_out);
        memfcat_str(a_memf_head, head_out.str().c_str());

        std::stringstream body_out;
        body.print(body_out);
        memfcat_str(a_memf_body, body_out.str().c_str());
    }
}
