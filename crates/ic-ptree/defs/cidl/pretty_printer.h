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
#include <cassert>
#include <functional>
#include <list>
#include <numeric>
#include <sstream>
#include <string>
#include <vector>

#include "cidl/ptree.h"
#include "cidl/symbols.h"

namespace intercom::cidl {

class PrettyPrinter {
  public:
    enum token_kind {
        TEXT,
        BLOCK_START,
        INDENT_TO_COLUMN_BLOCK_START,  // lock indentation to current column pos (until BLOCK_END)
        BLOCK_END,
        PARENT_INDENT,
        TAB,
        TAB_GROUP,
        NEWLINE,
        SOFT_NEWLINE
    };

    struct token {
        token_kind kind;
        std::string text;
    };

    using tokens = std::list<token>;

    struct Context {
        Context(PrettyPrinter& parent, const ptree* node) : parent(parent) {
            parent.push(node);
        }

        ~Context() {
            parent.pop();
        }

        PrettyPrinter& parent;
    };

    PrettyPrinter() : p(new impl) {}

    template <typename T>
    PrettyPrinter& operator<<(T arg) {
        std::stringstream stream;
        stream << arg;
        std::string input = stream.str();
        if (input.find('\n') != std::string::npos) {
            std::string line;
            std::istringstream istream(stream.str());
            bool first = true;
            while (std::getline(istream, line)) {
                if (!first) {
                    endl();
                }
                first = false;
                add(TEXT, line);
            }
        } else {
            add(TEXT, input);
        }
        return *this;
    }

    PrettyPrinter& operator<<(const ptree* node) {
        operator<<(idl_scoped_name(node, context()));
        return *this;
    }

    PrettyPrinter& operator<<(const PrettyPrinter& other) {
        for (const auto& token : other.p->values) {
            add(token.kind, token.text);
        }
        return *this;
    }

    PrettyPrinter& operator<<(PrettyPrinter& (*pf)(PrettyPrinter&)) {
        return pf(*this);
    }

    PrettyPrinter& operator<<(const std::function<PrettyPrinter&(PrettyPrinter&)>& pf) {
        return pf(*this);
    }

    bool has_text_content() const {
        return std::any_of(p->values.begin(), p->values.end(), [](token& t) {
            return !t.text.empty();
        });
    }

    PrettyPrinter& endl(int count = 1) {
        int found = 0;
        auto prev_newline = p->values.end();
        if (has_text_content()) {
            // count preceding empty (or whitespace only) lines
            for (auto it = p->values.rbegin(); it != p->values.rend(); ++it) {
                if (it->kind == NEWLINE || (count > 1 && it->kind == BLOCK_START)) {
                    if (++found == 1) {
                        auto it_cpy = it;
                        prev_newline = (++it_cpy).base();
                    }
                }
                if (!it->text.empty()) {
                    break;
                }
            }
            // rm preceding whitespace on last line [in case endl() does not add a newline]
            while (prev_newline != p->values.end()) {
                auto it = prev_newline++;
                if (it->kind == TAB) {
                    p->values.erase(it);
                }
            }
            // ad newline(s)
            while (found < count) {
                add(PrettyPrinter::NEWLINE, std::string());
                ++found;
            }
        }
        return *this;
    }

    void get_tab_stops(
        const tokens::iterator& first,
        const tokens::iterator& last,
        std::vector<unsigned int>& tab_stops
    ) const {
        tab_stops.clear();
        tab_stops.push_back(0);
        if (first != last) {
            bool changed = true;
            while (changed) {
                changed = false;
                auto it = first;
                ++it;
                int tab = 0;
                int column = 0;
                int level = 0;
                bool ignore_line = false;
                while (it != last) {
                    if (it->kind == BLOCK_START || it->kind == INDENT_TO_COLUMN_BLOCK_START) {
                        ++level;
                        column += static_cast<int>(it->text.size());
                    } else if (it->kind == BLOCK_END) {
                        if (level == 0) {
                            break;
                        }
                        --level;
                        column += static_cast<int>(it->text.size());
                    } else if (it->kind == PARENT_INDENT) {
                        column -= static_cast<int>(p->indent_size);
                        ignore_line = true;
                    } else if (it->kind == NEWLINE) {
                        column = 0;
                        tab = 0;
                        ignore_line = false;
                    } else if (it->kind == TAB_GROUP) {
                        if (level == 0) {
                            break;
                        }
                    } else if (it->kind == TAB) {
                        if (column != 0) {
                            ++column;
                        }
                        if (level == 0 && !ignore_line) {
                            ++tab;
                            if (tab_stops.size() <= static_cast<unsigned>(tab)) {
                                tab_stops.push_back(column);
                                changed = true;
                            } else if (column > 0 &&
                                       static_cast<unsigned>(column) > tab_stops[tab]) {
                                tab_stops[tab] = column;
                                changed = true;
                            }
                            column = static_cast<int>(tab_stops[tab]);
                        }
                    } else {
                        column += static_cast<int>(it->text.size());
                    }
                    ++it;
                }
            }
        }
    }

    void print(std::ostream& stream, unsigned int indent_base = 0) const {
        unsigned int level = 0;
        unsigned int indent_count = 0;
        unsigned int tab = 0;
        unsigned int column = 0;  // ignores indentation
        std::vector<unsigned int> level_indent{indent_base};
        std::vector<std::vector<unsigned int>> tab_stops;
        tab_stops.resize(1);
        get_tab_stops(p->values.begin(), p->values.end(), tab_stops[0]);
        for (auto it = p->values.begin(); it != p->values.end(); ++it) {
            if ((it->kind == BLOCK_END || it->kind == PARENT_INDENT) && indent_count > 0) {
                --indent_count;
            }
            token& t = *it;
            if (!t.text.empty()) {
                stream << std::string(
                    std::accumulate(
                        level_indent.begin(), level_indent.begin() + indent_count + 1U, 0U
                    ),
                    ' '
                );
                if (tab_stops[level].size() > tab && column < tab_stops[level][tab]) {
                    stream << std::string(tab_stops[level][tab] - column, ' ');
                    column = tab_stops[level][tab];
                }
                stream << t.text;
                column += static_cast<unsigned int>(t.text.size());
                indent_count = 0;
            }
            switch (t.kind) {
            case TEXT:
            case PARENT_INDENT:
                break;
            case BLOCK_START:
            case INDENT_TO_COLUMN_BLOCK_START:
                ++level;
                tab_stops.resize(level + 1);
                level_indent.resize(tab_stops.size());
                if (t.kind == INDENT_TO_COLUMN_BLOCK_START) {
                    level_indent[level] = column;
                } else {
                    level_indent[level] = p->indent_size;
                }
                break;
            case BLOCK_END:
                assert(level > 0);
                --level;
                break;
            case TAB:
                ++tab;
                if (tab_stops[level].size() <= tab || column >= tab_stops[level][tab]) {
                    stream << " ";
                    ++column;
                }
                break;
            case TAB_GROUP:
                get_tab_stops(it, p->values.end(), tab_stops[level]);
                break;
            case NEWLINE:
                stream << std::endl;
                indent_count = level;
                tab = 0;
                column = 0;
                break;
            case SOFT_NEWLINE:
                if (column > 80) {
                    stream << std::endl;
                    indent_count = level;
                    tab = 0;
                    column = 0;
                } else {
                    stream << " ";
                    ++column;
                }
                break;
            }
        }
    }

    std::string str() const {
        std::stringstream stream;
        print(stream);
        return stream.str();
    }

    void push(const ptree* context) {
        p->context.push_back(context);
    }

    void pop() {
        p->context.pop_back();
    }

    const ptree* context() {
        return p->context.empty() ? nullptr : p->context[p->context.size() - 1];
    }

    void add(token_kind kind, const std::string& text) {
        token t = {kind, text};
        p->values.push_back(t);
    }

    PrettyPrinter& begin(const std::string& brace) {
        add(PrettyPrinter::BLOCK_START, brace);
        return *this;
    }

    PrettyPrinter& indent_to_column_begin(const std::string& brace) {
        add(PrettyPrinter::INDENT_TO_COLUMN_BLOCK_START, brace);
        return *this;
    }

    PrettyPrinter& end(const std::string& brace) {
        add(PrettyPrinter::BLOCK_END, brace);
        return *this;
    }

    bool first_in_block() const {
        return p->values.empty() || p->values.rbegin()->kind == BLOCK_START;
    }

    void set_indent_size(unsigned int size) {
        p->indent_size = size;
    }

    bool empty() const {
        return p->values.empty();
    }

  private:
    struct impl {
        impl() = default;

        tokens values;
        std::vector<const ptree*> context;
        unsigned int indent_size = 4;
    };
    std::shared_ptr<impl> p;
};

inline PrettyPrinter& endl(PrettyPrinter& out) {
    return out.endl();
}

inline std::function<PrettyPrinter&(PrettyPrinter&)> begin(const std::string& brace) {
    return [=](PrettyPrinter& out) -> PrettyPrinter& { return out.begin(brace); };
}

inline std::function<PrettyPrinter&(PrettyPrinter&)> indent_to_column_begin(const std::string& brace
) {
    return [=](PrettyPrinter& out) -> PrettyPrinter& { return out.indent_to_column_begin(brace); };
}

inline std::function<PrettyPrinter&(PrettyPrinter&)> end(const std::string& brace) {
    return [=](PrettyPrinter& out) -> PrettyPrinter& { return out.end(brace); };
}

inline PrettyPrinter& begin_curly(PrettyPrinter& out) {
    return out.begin("{");
}

inline PrettyPrinter& end_curly(PrettyPrinter& out) {
    return out.end("}");
}

inline PrettyPrinter& begin_paren(PrettyPrinter& out) {
    return out.begin("(");
}

inline PrettyPrinter& end_paren(PrettyPrinter& out) {
    return out.end(")");
}

inline PrettyPrinter& begin(PrettyPrinter& out) {
    return out.begin("");
}

inline PrettyPrinter& end(PrettyPrinter& out) {
    return out.end("");
}

inline PrettyPrinter& blank_line(PrettyPrinter& out) {
    return out.endl(2);
}

inline PrettyPrinter& double_blank_line(PrettyPrinter& out) {
    return out.endl(3);
}

inline PrettyPrinter& list_sep(PrettyPrinter& out) {
    if (!out.first_in_block()) {
        out << ", ";
    }
    return out;
}

inline PrettyPrinter& unindent(PrettyPrinter& out) {
    out.add(PrettyPrinter::PARENT_INDENT, std::string());
    return out;
}

inline PrettyPrinter& tab(PrettyPrinter& out) {
    out.add(PrettyPrinter::TAB, std::string());
    return out;
}

inline PrettyPrinter& tab_group(PrettyPrinter& out) {
    out.add(PrettyPrinter::TAB_GROUP, std::string());
    return out;
}

inline PrettyPrinter& sp(PrettyPrinter& out) {
    out.add(PrettyPrinter::TEXT, " ");
    return out;
}

}  // namespace intercom::cidl
