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

#include <optional>
#include <stdexcept>

#include "InterCOM/cidl_json.h"
#include "cidl/json.h"
#include "cidl/ptree_builder.h"
#include "cidl/symbols.h"

using namespace intercom::cidl;

namespace {
struct Serialize {
    json::NodeId next_id = 0;
    std::vector<json::Node> nodes;
    std::map<const ptree*, json::NodeId> alloc;
};

struct Deserialize {
    std::map<json::NodeId, json::Node> nodes;
    std::map<json::NodeId, ptree*> alloc;
};

using NodeMap = std::unordered_map<json::NodeId, json::Node>;
}  // namespace

void init_parser_state(const intercom::RefPointer<parser>& state);

static json::Numeric json_numeric(const numeric& numeric, Serialize& context);

static numeric ptree_numeric(const json::Numeric&, Deserialize&);

static json::NodeKind json_kind(node_kind kind) {
    return static_cast<json::NodeKind>(kind);
}

static node_kind json_kind(json::NodeKind kind) {
    return static_cast<node_kind>(kind);
}

static numeric_kind numeric_kind(json::NumericKind kind) {
    return static_cast<enum numeric_kind>(kind);
}

static json::NumericKind json_numeric_kind(enum numeric_kind kind) {
    return static_cast<json::NumericKind>(kind);
}

static std::optional<json::NodeId> serialize(const ptree* node, Serialize& context) {
    if (!node) {
        return intercom::nullopt;
    }

    auto it = context.alloc.find(node);
    if (it != context.alloc.end()) {
        return it->second;
    }

    json::Node obj;
    obj.id = context.next_id++;
    obj.name = node->name;
    obj.kind = json_kind(node->kind);
    obj.flags = node->flags;
    obj.file_name = node->file_name;
    obj.pos.line = node->pos.line;
    obj.pos.column = node->pos.column;
    obj.pos_end.line = node->pos_end.line;
    obj.pos_end.column = node->pos_end.column;
    context.alloc.emplace(node, obj.id);

    // Reserve a spot so that IDs map directly to indices
    context.nodes.emplace_back();

    obj.next = serialize(node->next, context);
    obj.super = serialize(node->super, context);
    obj.scope = serialize(node->scope, context);
    obj.members = serialize(node->members, context);
    obj.annotations = serialize(node->annotations, context);
    obj.included_from = serialize(node->included_from, context);
    obj.value = json_numeric(node->value, context);

    auto append = [&](const ptree* ty, std::optional<json::Type>& out) {
        if (ty) {
            json::Type type;
            if (ty->flags & OPT_BUILTIN) {
                type.qualified_name(idl_scoped_name(ty, nullptr));
            } else {
                type.id(*serialize(ty, context));
            }
            out = type;
        }
    };

    append(node->type, obj.type);
    append(node->element_type, obj.element_type);
    append(node->key_type, obj.key_type);
    append(node->discriminator, obj.discriminator);

    for (auto parent : node->parents) {
        obj.parents.emplace_back(*serialize(parent, context));
    }
    for (auto raise : node->getraises) {
        obj.getraises.emplace_back(*serialize(raise, context));
    }
    for (auto raise : node->setraises) {
        obj.setraises.emplace_back(*serialize(raise, context));
    }
    for (const auto& bound : node->bounds) {
        obj.bounds.emplace_back(json_numeric(bound, context));
    }
    context.nodes[obj.id] = std::move(obj);
    return obj.id;
}

static json::Numeric json_numeric(const numeric& numeric, Serialize& context) {
    json::Numeric value;
    switch (numeric.val._d()) {
    case BOOLEAN_KIND:
        value.b(numeric->b());
        break;
    case INT8_KIND:
        value.i8(numeric->i8());
        break;
    case OCTET_KIND:
        value.o(numeric->o());
        break;
    case SHORT_KIND:
        value.s(numeric->s());
        break;
    case USHORT_KIND:
        value.us(numeric->us());
        break;
    case LONG_KIND:
        value.l(numeric->l());
        break;
    case ULONG_KIND:
        value.ul(numeric->ul());
        break;
    case LONGLONG_KIND:
        value.ll(numeric->ll());
        break;
    case ULONGLONG_KIND:
        value.ull(numeric->ull());
        break;
    case FLOAT_KIND:
        value.f(numeric->f());
        break;
    case DOUBLE_KIND:
        value.d(numeric->d());
        break;
    case CHAR_KIND:
        value.c(numeric->c());
        break;
    case STRING_KIND:
        value.str(numeric->str());
        break;
    case PTREE_KIND:
        value.node(serialize(numeric->node(), context).value());
        break;
    case UNDEF_KIND:
        value._d(json::UNDEF_KIND);
        break;
    }
    return value;
}

static ptree* builtin_type(const std::string& name) {
    if (auto type = lookup_node(create_identifier(name.c_str()))) {
        return type;
    }
    return g_state->type_map.at(name);
}

static ptree* deserialize(json::NodeId id, Deserialize& context) {
    auto it = context.alloc.find(id);
    if (it != context.alloc.end()) {
        return it->second;
    }

    const auto& node = context.nodes.at(id);
    auto kind = json_kind(node.kind);
    auto obj = create_node(kind, create_identifier(node.name.c_str()));
    obj->flags = node.flags;
    obj->file_name = node.file_name;
    obj->pos.line = static_cast<int>(node.pos.line);
    obj->pos.column = static_cast<int>(node.pos.column);
    obj->pos_end.line = static_cast<int>(node.pos_end.line);
    obj->pos_end.column = static_cast<int>(node.pos_end.column);
    context.alloc.emplace(id, obj);

    auto append = [&](const std::optional<json::NodeId>& ty, ptree*& list) {
        if (ty) {
            list = append_node(list, deserialize(*ty, context));
        }
    };

    append(node.next, obj->next);
    append(node.super, obj->super);
    append(node.scope, obj->scope);
    append(node.members, obj->members);
    append(node.annotations, obj->annotations);
    append(node.included_from, obj->included_from);

    auto builtin = [&](const std::optional<json::Type>& ty, ptree*& out) {
        if (ty) {
            if (ty->_d() == json::BUILTIN_TYPE) {
                out = builtin_type(ty->qualified_name());
            } else {
                append(ty->id(), out);
            }
        }
    };

    builtin(node.type, obj->type);
    builtin(node.element_type, obj->element_type);
    builtin(node.key_type, obj->key_type);
    builtin(node.discriminator, obj->discriminator);

    if (node.value) {
        obj->value = ptree_numeric(*node.value, context);
    }
    for (auto parent : node.parents) {
        obj->parents.emplace_back(deserialize(parent, context));
    }
    for (auto raise : node.getraises) {
        obj->getraises.emplace_back(deserialize(raise, context));
    }
    for (auto raise : node.setraises) {
        obj->setraises.emplace_back(deserialize(raise, context));
    }
    for (const auto& bound : node.bounds) {
        obj->bounds.emplace_back(ptree_numeric(bound, context));
    }
    return obj;
}

static numeric ptree_numeric(const json::Numeric& numeric, Deserialize& nodes) {
    struct numeric value;
    switch (numeric._d()) {
    case json::BOOLEAN_KIND:
        value.val.b(numeric.b());
        break;
    case json::INT8_KIND:
        value.val.i8(numeric.i8());
        break;
    case json::OCTET_KIND:
        value.val.o(numeric.o());
        break;
    case json::SHORT_KIND:
        value.val.s(numeric.s());
        break;
    case json::USHORT_KIND:
        value.val.us(numeric.us());
        break;
    case json::LONG_KIND:
        value.val.l(numeric.l());
        break;
    case json::ULONG_KIND:
        value.val.ul(numeric.ul());
        break;
    case json::LONGLONG_KIND:
        value.val.ll(numeric.ll());
        break;
    case json::ULONGLONG_KIND:
        value.val.ull(numeric.ull());
        break;
    case json::FLOAT_KIND:
        value.val.f(numeric.f());
        break;
    case json::DOUBLE_KIND:
        value.val.d(numeric.d());
        break;
    case json::CHAR_KIND:
        value.val.c(numeric.c());
        break;
    case json::STRING_KIND:
        value.val.str(numeric.str());
        break;
    case json::PTREE_KIND:
        value.val.node(deserialize(numeric.node(), nodes));
        break;
    case json::UNDEF_KIND:
        value.val._d(UNDEF_KIND);
        break;
    }
    return value;
}

static void register_primitives() {
    g_state->type_map["any"] = &any_type;
    g_state->type_map["Object"] = &object_type;
    g_state->type_map["boolean"] = &boolean_type;
    g_state->type_map["int8"] = &int8_type;
    g_state->type_map["uint8"] = &octet_type;
    g_state->type_map["char"] = &char_type;
    g_state->type_map["wchar"] = &wchar_type;
    g_state->type_map["int16"] = &short_type;
    g_state->type_map["uint16"] = &ushort_type;
    g_state->type_map["int32"] = &long_type;
    g_state->type_map["uint32"] = &ulong_type;
    g_state->type_map["int64"] = &longlong_type;
    g_state->type_map["uint64"] = &ulonglong_type;
    g_state->type_map["float"] = &float_type;
    g_state->type_map["double"] = &double_type;
    g_state->type_map["long double"] = &ldouble_type;
    g_state->type_map["fixed"] = &fixed_type;
    g_state->type_map["string"] = &unbounded_string_type;
    g_state->type_map["wstring"] = &unbounded_wstring_type;
}

void intercom::cidl::to_json(std::ostream& out, const ptree* node) {
    Serialize context;
    serialize(node, context);

    json::Tree tree;
    tree.root = 0;
    tree.definitions = std::move(context.nodes);
    intercom::marshal_json(out, tree);
}

parse_result intercom::cidl::from_json(std::istream& stream) {
    json::Tree tree;
    intercom::unmarshal_json(stream, tree);

    Deserialize context;
    for (auto node : tree.definitions) {
        auto it = context.nodes.emplace(node.id, std::move(node));
        if (!it.second) {
            throw std::runtime_error(fmt::format("Duplicate node ID '{}'", node.id));
        }
    }

    IdlParser parser;
    parser.run([&] {
        register_primitives();
        return deserialize(tree.root, context);
    });
    return parser.result();
}
