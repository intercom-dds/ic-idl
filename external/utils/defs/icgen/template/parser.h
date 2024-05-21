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

#include <algorithm>
#include <array>
#include <cassert>
#include <cctype>
#include <cstdio>
#include <iostream>
#include <map>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "lexer.h"

namespace intercom {
namespace icgen {

template <typename T, typename... Args>
std::unique_ptr<T> make_unique(Args&&... args) {
    return std::unique_ptr<T>(new T(std::forward<Args>(args)...));
}

class Visitor {
  public:
    virtual ~Visitor() = default;

    virtual void visit(class Stmt*) {}
    virtual void visit(class IfStmt*) {}
    virtual void visit(class ForStmt*) {}
    virtual void visit(class Text*) {}

    virtual void visit(class Expr*) {}
    virtual void visit(class UnaryExpr*) {}
    virtual void visit(class BinaryExpr*) {}
    virtual void visit(class AssignExpr*) {}
    virtual void visit(class Function*) {}
    virtual void visit(class Variable*) {}
    virtual void visit(class String*) {}
    virtual void visit(class Member*) {}
};

class Node {
  public:
    enum class NodeKind {
        Undef,
        If,
        For,
        Text,
        String,
        Member,
        Unary,
        Binary,
        Assign,
        Function,
        Variable
    };

    explicit Node(NodeKind kind) : kind(kind) {}

    virtual ~Node() = default;

    virtual void accept(Visitor*) = 0;

    const NodeKind kind;
};

using NodePtr = std::unique_ptr<Node>;
using ExprPtr = std::unique_ptr<Expr>;

class Stmt : public Node {
  public:
    explicit Stmt(NodeKind kind) : Node(kind) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }
};

class Expr : public Node {
  public:
    explicit Expr(NodeKind kind) : Node(kind) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }
};

/// Represents a variable to be substituted.
class Variable : public Expr {
  public:
    explicit Variable(Token var) : Expr(NodeKind::Variable), name(var) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token name;
};

/// Represents a function call.
/// Functions are defined in C++, but can be invoked from the template code.
class Function : public Expr {
  public:
    explicit Function(Token func) : Expr(NodeKind::Function), name(func) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token name;

    std::vector<ExprPtr> params;
};

class UnaryExpr : public Expr {
  public:
    UnaryExpr(Token token, std::unique_ptr<Expr> expr)
        : Expr(NodeKind::Unary), oper(token), expr(std::move(expr)) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token oper;
    std::unique_ptr<Expr> expr;
};

class BinaryExpr : public Expr {
  public:
    BinaryExpr(std::unique_ptr<Expr> lhs, Token token, std::unique_ptr<Expr> rhs)
        : Expr(NodeKind::Binary), lhs(std::move(lhs)), oper(token), rhs(std::move(rhs)) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    std::unique_ptr<Expr> lhs;
    Token oper;
    std::unique_ptr<Expr> rhs;
};

class AssignExpr : public Expr {
  public:
    AssignExpr(Token var, std::unique_ptr<Expr> value)
        : Expr(NodeKind::Assign), var(var), value(std::move(value)) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token var;
    std::unique_ptr<Expr> value;
};

/// A string. Only allowed as function parameters.
class String : public Expr {
  public:
    explicit String(Token string) : Expr(NodeKind::String), string(string) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token string;
};

/// Member access into a variable, e.g. `my_var.my_member`
class Member : public Expr {
  public:
    explicit Member(Token name) : Expr(NodeKind::Member), name(name) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token name;
    ExprPtr var;
};

/// Represents an `if` statement.
class IfStmt : public Stmt {
  public:
    explicit IfStmt(std::unique_ptr<Expr> condition)
        : Stmt(NodeKind::If), condition(std::move(condition)) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

  public:
    std::unique_ptr<Expr> condition;
    std::vector<NodePtr> if_body;
    std::vector<NodePtr> else_body;
};

class ForStmt : public Stmt {
  public:
    ForStmt(std::unique_ptr<Variable> var, std::unique_ptr<Expr> list)
        : Stmt(NodeKind::For), var(std::move(var)), list(std::move(list)) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

  public:
    std::unique_ptr<Variable> var;
    std::unique_ptr<Variable> enumerator;
    std::unique_ptr<Expr> list;
    std::vector<NodePtr> body;
};

/// Represents a chunk of raw text.
/// Does not contain any code or comment blocks.
class Text : public Stmt {
  public:
    explicit Text(Token text, bool whitespace)
        : Stmt(NodeKind::Text), text(text), whitespace(whitespace) {}

    void accept(Visitor* visitor) override {
        visitor->visit(this);
    }

    Token text;
    bool whitespace;
};

class Parser {
  public:
    explicit Parser(const std::vector<Token>& tokens) : m_tokens(tokens) {}

    const Token& get() const {
        return m_idx < m_tokens.size() ? m_tokens[m_idx] : Eof;
    }

    const Token& peek() const {
        return (m_idx + 1) < m_tokens.size() ? m_tokens[m_idx + 1] : Eof;
    }

    std::runtime_error fmt_error(std::string_view msg) const {
        auto token = get();
        if (token == TokenKind::Eof) {
            token = m_tokens.back();
        }
        std::string pos = std::to_string(token.line) + ":" + std::to_string(token.col);
        return std::runtime_error(pos + ": " + std::string(msg));
    }

    Token expect(TokenKind kind) {
        if (get().kind != kind) {
            std::string msg = "expected " + TOKEN_NAMES[size_t(kind)] + ", found ";

            if (get().kind == TokenKind::Ident) {
                msg += "identifier '" + std::string(get().view) + '\'';
            } else if (get().kind == TokenKind::Invalid) {
                msg += "invalid identifier '" + std::string(get().view) + '\'';
            } else {
                msg += '\'' + TOKEN_NAMES[size_t(get().kind)] + '\'';
            }
            throw fmt_error(msg);
        }
        return take();
    }

    Token take() {
        auto token = get();
        m_idx++;
        return token;
    }

    void block_end() {
        if (get() == TokenKind::RBracket || get() == TokenKind::Semi) {
            take();
        }
    }

    void block_end(TokenKind kind) {
        expect(kind);
        block_end();
    }

    std::unique_ptr<Variable> parse_ident() {
        auto ident = expect(TokenKind::Ident);
        return icgen::make_unique<Variable>(ident);
    }

    std::unique_ptr<Member> member() {
        auto ident = expect(TokenKind::Ident);
        expect(TokenKind::Dot);
        auto expr = icgen::make_unique<Member>(ident);

        if (peek() == TokenKind::Dot) {
            expr->var = member();
        } else {
            expr->var = parse_ident();
        }
        return expr;
    }

    ExprPtr parse_var() {
        ExprPtr expr;
        if (peek().kind == TokenKind::Eq) {
            expr = assignment();
        } else {
            expr = parse_ident();
        }
        block_end();
        return expr;
    }

    /// Parameters are ordinary expressions.
    /// They can be strings, variables, function calls, or expression groups.
    std::vector<ExprPtr> param_list() {
        std::vector<ExprPtr> params;
        while (get().kind != TokenKind::RParen && get().kind != TokenKind::Eof) {
            params.emplace_back(expr());

            // we allow trailing commas, because why not
            if (get().kind != TokenKind::Comma) {
                break;
            }
            take();
        }
        return params;
    }

    std::unique_ptr<Function> function() {
        auto func = icgen::make_unique<Function>(take());
        expect(TokenKind::LParen);
        func->params = param_list();
        expect(TokenKind::RParen);
        return func;
    }

    std::unique_ptr<AssignExpr> assignment() {
        assert(get().kind == TokenKind::Ident);
        auto var = take();
        expect(TokenKind::Eq);
        return icgen::make_unique<AssignExpr>(var, expr());
    }

    ExprPtr parse_group() {
        expect(TokenKind::LParen);
        auto node = expr();
        expect(TokenKind::RParen);
        return node;
    }

    /// Operator precedence is defined as follows, from highest to lowest:
    ///   1. `or`
    ///   2. `and`
    ///   3. `==` and `!=`
    ///   4. `not`
    ExprPtr expr() {
        auto lhs = parse_and();
        while (get().kind == TokenKind::Or) {
            auto op = take();
            lhs = icgen::make_unique<BinaryExpr>(std::move(lhs), op, parse_and());
        }

        // skip superfluous semicolons
        block_end();
        while (get() == TokenKind::Semi) {
            take();
        }
        return lhs;
    }

    ExprPtr parse_and() {
        auto lhs = parse_eq();
        while (get().kind == TokenKind::And) {
            auto op = take();
            return icgen::make_unique<BinaryExpr>(std::move(lhs), op, parse_eq());
        }
        return lhs;
    }

    ExprPtr parse_eq() {
        auto lhs = parse_unary();
        while (get().kind == TokenKind::EqEq || get().kind == TokenKind::NotEq) {
            auto op = take();
            return icgen::make_unique<BinaryExpr>(std::move(lhs), op, parse_unary());
        }
        return lhs;
    }

    ExprPtr parse_unary() {
        // only supported unary operator
        if (get().kind == TokenKind::Not) {
            auto op = take();
            return icgen::make_unique<UnaryExpr>(op, parse_unary());
        }
        return parse_call();
    }

    ExprPtr parse_call() {
        if (get() == TokenKind::String) {
            return icgen::make_unique<String>(take());
        }
        if (get() == TokenKind::LParen) {
            return parse_group();
        }
        if (peek() == TokenKind::LParen) {
            return function();
        }
        if (peek() == TokenKind::Dot) {
            return member();
        }
        return parse_var();
    }

    std::unique_ptr<IfStmt> parse_if() {
        // consume the leading if/elif
        take();
        auto cond = expr();
        // Conditions must either be followed by a block end, or optionally a semicolon.
        block_end();

        auto stmt = icgen::make_unique<IfStmt>(std::move(cond));
        while (get() != TokenKind::Else && get() != TokenKind::Elif && get() != TokenKind::EndIf) {
            stmt->if_body.emplace_back(any());
        }

        if (get() == TokenKind::Elif) {
            block_end();
            stmt->else_body.emplace_back(parse_if());
            return stmt;
        }
        if (get() == TokenKind::Else) {
            block_end(TokenKind::Else);
            while (get() != TokenKind::EndIf) {
                stmt->else_body.emplace_back(any());
            }
        }

        block_end(TokenKind::EndIf);
        return stmt;
    }

    std::unique_ptr<ForStmt> parse_for() {
        expect(TokenKind::For);
        std::unique_ptr<Variable> enumerator;

        if (peek().kind == TokenKind::Comma) {
            enumerator = parse_ident();
            take();
        }

        auto var = parse_ident();
        expect(TokenKind::In);
        auto stmt = icgen::make_unique<ForStmt>(std::move(var), parse_call());
        stmt->enumerator = std::move(enumerator);
        block_end();

        while (get() != TokenKind::Eof && get() != TokenKind::EndFor) {
            stmt->body.emplace_back(any());
        }
        block_end(TokenKind::EndFor);
        return stmt;
    }

    NodePtr parse_text() {
        auto kind = get().kind;
        assert(kind == TokenKind::Text || kind == TokenKind::Whitespace);
        return icgen::make_unique<Text>(take(), kind == TokenKind::Whitespace);
    }

    NodePtr any() {
        switch (get().kind) {
        case TokenKind::For:
            return parse_for();
        case TokenKind::If:
            return parse_if();
        case TokenKind::Ident:
            return expr();
        case TokenKind::Text:
        case TokenKind::Whitespace:
            return parse_text();
        case TokenKind::Eof:
            throw fmt_error("expected statement or expression, found EOF");
        default:
            throw fmt_error(
                "expected statement or expression, found '" + std::string(get().view) + '\''
            );
        }
    }

    std::vector<NodePtr> parse() {
        std::vector<NodePtr> nodes;
        while (get() != TokenKind::Eof) {
            if (get() == TokenKind::RBracket) {
                // empty code blocks are permitted but are not included in the AST
                take();
            } else {
                nodes.emplace_back(any());
            }
        }
        return nodes;
    }

  private:
    size_t m_idx = 0;
    const std::vector<Token>& m_tokens;
};

}  // namespace icgen
}  // namespace intercom
