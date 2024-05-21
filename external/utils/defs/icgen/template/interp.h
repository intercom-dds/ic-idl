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

#include <functional>
#include <stdexcept>
#include <string_view>
#include <utility>

#include "parser.h"

namespace intercom {
namespace icgen {

struct TemplateError : public std::runtime_error {
    explicit TemplateError(const std::string& what) : std::runtime_error(what) {}
};

struct ValueError : public TemplateError {
    explicit ValueError(const std::string& what) : TemplateError(what) {}
};

class CtxError : public TemplateError {
  public:
    static CtxError from_token(const Token& token, const std::string& msg) {
        std::stringstream err;
        err << token.line << ":" << token.col << ": " << msg;
        return {token, err.str()};
    }

  private:
    CtxError(Token context, const std::string& what) : TemplateError(what), m_context(context) {}

    const Token m_context;
};

class Scope;

class Value {
  public:
    enum Kind {
        None,
        Bool,
        List,
        String,
        Map,
    };

    Value() : m_kind(Kind::None) {}

    Value(bool boolean) : m_kind(Kind::Bool) {
        m_storage.boolean = boolean;
    }

    Value(const char* str) : m_kind(Kind::String) {
        new (&m_storage.string) std::string(str);
    }

    Value(std::string str) : m_kind(Kind::String) {
        new (&m_storage.string) std::string(std::move(str));
    }

    Value(std::vector<Value> list) : m_kind(Kind::List) {
        new (&m_storage.list) std::vector<Value>(std::move(list));
    }

    Value(Scope map);

    Value(const Value& other) : m_kind(other.m_kind) {
        construct(other);
    }

    Value& operator=(const Value& other) {
        if (this != &other) {
            clear();
            m_kind = other.m_kind;
            construct(other);
        }
        return *this;
    }

    bool operator==(const Value& other) const {
        if (m_kind != other.m_kind) {
            return false;
        }
        switch (m_kind) {
        case Kind::None:
            return true;
        case Kind::Bool:
            return boolean() == other.boolean();
        case Kind::List:
            return list() == other.list();
        case Kind::String:
            return str() == other.str();
        case Kind::Map:
            return map() == other.map();
        }
        return false;
    }

    bool operator!=(const Value& other) const {
        return !(*this == other);
    }

    explicit operator bool() const {
        return m_kind != Kind::None;
    }

    ~Value() {
        clear();
    }

    Kind kind() const {
        return m_kind;
    }

    std::string kind_str() const {
        switch (kind()) {
        case Kind::None:
            return "none";
        case Kind::Bool:
            return "bool";
        case Kind::List:
            return "list";
        case Kind::String:
            return "string";
        default:
            return "map";
        }
    }

    bool boolean() const {
        if (m_kind != Kind::Bool) {
            throw ValueError("expected boolean, found " + kind_str());
        }
        return m_storage.boolean;
    }

    const std::string& str() const {
        if (m_kind != Kind::String) {
            throw ValueError("expected string, found " + kind_str());
        }
        return m_storage.string;
    }

    const std::vector<Value>& list() const {
        if (m_kind != Kind::List) {
            throw ValueError("expected list, found " + kind_str());
        }
        return m_storage.list;
    }

    Scope* map() const {
        if (m_kind != Kind::Map) {
            throw ValueError("expected map, found " + kind_str());
        }
        return m_storage.map.get();
    }

  private:
    void clear();

    void construct(const Value& other);

    Kind m_kind;
    union Storage {
        Storage() {}
        ~Storage() {}
        bool boolean;
        std::string string;
        std::vector<Value> list;
        std::unique_ptr<Scope> map;
    } m_storage;
};

struct FunctionData {
    using Args = std::vector<Value>;
    using Callback = std::function<Value(class Scope*, const Args&)>;

    enum class Kind {
        Bool,
        List,
        String,
    } kind;

    int n_args;
    Callback callback;
};

class Scope {
  public:
    Scope() = default;

    explicit Scope(Scope* parent) : m_parent(parent) {}

    bool is_strict() {
        return contains("strict") && find("strict")->boolean();
    }

    Value* find(const std::string& var) {
        auto it = m_values.find(var);
        if (it != m_values.end()) {
            return &it->second;
        }
        if (m_parent) {
            return m_parent->find(var);
        }
        return nullptr;
    }

    Value* find(const Token& var) {
        std::string name(var.view);
        if (auto it = find(name)) {
            return it;
        }
        if (is_strict()) {
            throw CtxError::from_token(var, "undefined variable '" + name + '\'');
        }
        return nullptr;
    }

    Value* find_local(const Token& var) {
        std::string name(var.view);
        auto it = m_values.find(name);
        if (it != m_values.end()) {
            return &it->second;
        }
        if (is_strict()) {
            throw CtxError::from_token(var, "undefined variable '" + name + '\'');
        }
        return nullptr;
    }

    const FunctionData& function(const Token& token) {
        std::string name(token.view);
        auto it = m_funcs.find(name);
        if (it != m_funcs.end()) {
            return it->second;
        }
        if (m_parent) {
            return m_parent->function(token);
        }

        std::stringstream stream;
        stream << "undefined function '" << name << "'. list of available functions:\n";
        for (const auto& fn : m_funcs) {
            stream << " - " << fn.first << '\n';
        }
        throw CtxError::from_token(token, stream.str());
    }

    bool contains(const std::string& var) const {
        if (m_values.count(var)) {
            return true;
        }
        if (m_parent) {
            return m_parent->contains(var);
        }
        return false;
    }

    void assign(std::string_view var, const Value& value) {
        std::string name(var);
        if (auto ptr = find(name)) {
            *ptr = value;
        } else {
            m_values[name] = value;
        }
    }

    void define(const std::string& name, const FunctionData& func) {
        m_funcs[name] = func;
    }

    const std::map<std::string, Value>& values() const {
        return m_values;
    }

    const Scope* parent() const {
        return m_parent;
    }

  private:
    Scope* m_parent{};
    // C++11 doesn't support transparent comparators, so we'll have to use
    // strings here.
    std::map<std::string, Value> m_values;
    std::map<std::string, FunctionData> m_funcs;
};

inline Value::Value(Scope scope) : m_kind(Value::Map) {
    new (&m_storage.map) std::unique_ptr<Scope>(new Scope(std::move(scope)));
}

inline void Value::clear() {
    if (m_kind == Kind::String) {
        m_storage.string.~basic_string();
    } else if (m_kind == Kind::List) {
        m_storage.list.~vector<Value>();
    } else if (m_kind == Kind::Map) {
        m_storage.map.~unique_ptr<Scope>();
    }
    m_kind = Kind::None;
}

inline void Value::construct(const Value& other) {
    if (m_kind == Kind::Bool) {
        m_storage.boolean = other.m_storage.boolean;
    } else if (m_kind == Kind::String) {
        new (&m_storage.string) std::string(other.m_storage.string);
    } else if (m_kind == Kind::List) {
        new (&m_storage.list) std::vector<Value>(other.m_storage.list);
    } else if (m_kind == Kind::Map) {
        new (&m_storage.map) std::unique_ptr<Scope>(new Scope(*other.m_storage.map));
    }
}

struct ErrorBuilder {
    static CtxError argument_count(const Function* func, const FunctionData& data) {
        std::stringstream err;
        auto count = abs(data.n_args);

        err << func->name.view << "() takes ";
        if (data.n_args < 0) {
            err << "at least ";
        }
        err << count << " argument" << (count == 1 ? "" : "s");
        err << " but " << func->params.size() << " were given";
        return CtxError::from_token(func->name, err.str());
    }
};

class MemberResolver : public Visitor {
  public:
    static const Value* resolve(Scope* scope, Expr* node) {
        MemberResolver resolver(scope);
        node->accept(&resolver);
        return resolver.m_current;
    }

  private:
    explicit MemberResolver(Scope* scope) : m_scope(scope) {}

    void visit(Member* member) override {
        if (m_current) {
            find_value(member->name, m_current);
        } else {
            m_current = m_scope->find(member->name);
        }
        member->var->accept(this);
    }

    void visit(Variable* var) override {
        if (m_current) {
            find_value(var->name, m_current);
        }
    }

    void find_value(const Token& var, const Value* value) {
        if (value->kind() == Value::List) {
            size_t index = static_cast<size_t>(std::stoull(std::string(var.view)));
            if (index < value->list().size()) {
                m_current = &value->list()[index];
            } else if (m_scope->is_strict()) {
                throw CtxError::from_token(var, "index out of range");
            }
        } else {
            if (auto val = value->map()->find_local(var)) {
                m_current = val;
            }
        }
    }

  private:
    Scope* m_scope = nullptr;
    const Value* m_current = nullptr;
};

class Evaluator : public Visitor {
  public:
    explicit Evaluator(Scope* scope) : m_scope(scope) {}

    static Value evaluate(Scope* scope, Expr* expr) {
        Evaluator eval(scope);
        expr->accept(&eval);
        return eval.m_value;
    }

    static bool is_true(const Value& value) {
        switch (value.kind()) {
        case Value::None:
            return false;
        case Value::Bool:
            return value.boolean();
        case Value::List:
            return !value.list().empty();
        case Value::String:
            return !value.str().empty();
        case Value::Map:
            return !value.map()->values().empty();
        }
        return false;
    }

  private:
    void visit(UnaryExpr* expr) override {
        expr->expr->accept(this);
        m_value = !is_true(m_value);
    }

    void visit(BinaryExpr* expr) override {
        expr->lhs->accept(this);

        if ((is_true(m_value) && expr->oper == TokenKind::And) ||
            (!is_true(m_value) && expr->oper == TokenKind::Or)) {
            expr->rhs->accept(this);
        } else {
            auto temp = m_value;
            expr->rhs->accept(this);

            if (expr->oper == TokenKind::EqEq) {
                m_value = temp == m_value;
            } else {
                m_value = temp != m_value;
            }
        }
    }

    void visit(Function* func) override {
        const auto& fn = m_scope->function(func->name);
        size_t count = abs(fn.n_args);

        std::vector<Value> params;
        for (const auto& param : func->params) {
            if (auto ret = Evaluator::evaluate(m_scope, param.get())) {
                params.emplace_back(std::move(ret));
            }
        }

        if ((params.size() > count && fn.n_args > 0) || params.size() < count) {
            std::stringstream err;
            err << func->name.view << "() takes ";
            if (fn.n_args < 0) {
                err << "at least ";
            }
            err << count << " argument" << (count == 1 ? "" : "s");
            err << " but " << func->params.size() << " were given";
            throw CtxError::from_token(func->name, err.str());
        }

        try {
            m_value = fn.callback(m_scope, params);
        } catch (std::exception& e) {
            std::string msg = std::string(func->name.view) + ": ";
            throw CtxError::from_token(func->name, e.what());
        }
    }

    void visit(Variable* var) override {
        if (auto res = m_scope->find(var->name)) {
            m_value = *res;
        }
    }

    void visit(String* str) override {
        // Since each token is only a pointer to a data buffer we don't own, we can't
        // handle escaped characters in the lexer. Instead, we'll process it here when
        // we write the string, which is the only place where it really matters, anyway.
        m_value = escape_str(str->string.view);
    }

    void visit(Member* member) override {
        if (auto value = MemberResolver::resolve(m_scope, member)) {
            m_value = *value;
        }
    }

  private:
    Scope* m_scope;
    Value m_value{};
};

class Interp : public Visitor {
  public:
    Interp(Scope* scope, std::ostream& output) : m_scope(scope), m_stream(output) {}

    void execute(Node* node) {
        node->accept(this);
    }

  protected:
    // visits a node with the given scope
    void scoped_visit(Node* node, Scope* scope) {
        auto parent = m_scope;
        m_scope = scope;
        node->accept(this);
        m_scope = parent;
    }

    void write_str(std::string_view str) {
        m_text = !str.empty();
        if (m_text) {
            m_stream << m_indent;
        }

        // maintain whitespace for the entire block
        for (size_t i = 0; i < str.length(); i++) {
            m_stream << str[i];

            if (i < str.length() - 1 && str[i] == '\n') {
                m_stream << m_indent;
            }
        }
        m_indent = {};
    }

    void write_value(const Value& value) {
        if (value.kind() == Value::String) {
            write_str(value.str());
        }
    }

    void visit(IfStmt* stmt) override {
        auto value = Evaluator::evaluate(m_scope, stmt->condition.get());
        auto ok = Evaluator::is_true(value);
        const auto& body = ok ? stmt->if_body : stmt->else_body;

        Scope scope(m_scope);
        for (const auto& elem : body) {
            scoped_visit(elem.get(), &scope);
        }
    }

    void visit(ForStmt* stmt) override {
        std::string index;
        auto var = stmt->var->name;
        auto value = Evaluator::evaluate(m_scope, stmt->list.get());
        const auto& list = value.list();

        if (stmt->enumerator) {
            index = std::string(stmt->enumerator->name.view);
        }
        for (size_t i = 0; i < list.size(); i++) {
            Scope scope(m_scope);
            scope.assign(index, std::to_string(i));
            scope.assign(var.view, list[i]);

            for (const auto& node : stmt->body) {
                scoped_visit(node.get(), &scope);
            }
        }
    }

    void visit(AssignExpr* assignment) override {
        auto value = Evaluator::evaluate(m_scope, assignment->value.get());
        m_scope->assign(assignment->var.view, value);
    }

    void visit(Text* text) override {
        if (text->whitespace) {
            if (m_text && text->text.view == "\n") {
                m_stream << text->text.view;
            } else {
                m_indent = text->text.view;
            }
        } else {
            if (m_text) {
                m_stream << m_indent;
            }
            m_stream << text->text.view;
            m_indent = {};
        }
        m_text = !text->whitespace;
    }

    void visit(Function* func) override {
        auto ret = Evaluator::evaluate(m_scope, func);
        write_value(ret);
    }

    void visit(Variable* var) override {
        if (auto value = m_scope->find(var->name)) {
            write_value(*value);
        }
    }

    void visit(Member* member) override {
        auto ret = Evaluator::evaluate(m_scope, member);
        write_value(ret);
    }

  protected:
    Scope* m_scope;
    bool m_text = false;
    std::string_view m_indent;
    std::ostream& m_stream;
};

}  // namespace icgen
}  // namespace intercom
