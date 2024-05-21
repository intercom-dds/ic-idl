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

#include "InterCOM/detail/filesystem.h"

#include <fstream>
#include <iostream>
#include <iterator>
#include <vector>

#include "InterCOM/PlatformConfig.h"

#ifdef INTERCOM_PLATFORM_WINDOWS
#  include "fs_win.cpp"  // NOLINT
#else
#  include <pwd.h>

#  include "fs_unix.cpp"  // NOLINT
#endif

namespace {
bool is_sep(char c) {
    return c == '/' || c == '\\';
}

std::string to_string(const std::vector<intercom::fs::path>& a_list) {
    std::string path_str;
    for (auto it = a_list.begin(); it != a_list.end(); ++it) {
        path_str += it->native();
        if (std::next(it) != a_list.end() && !is_sep(*it->c_str())) {
            path_str.push_back(intercom::fs::path::preferred_separator);
        }
    }
    return path_str;
}
}  // namespace

class intercom::fs::path::Lexer {
  public:
    static std::vector<intercom::fs::path> split_all(const std::string& a_path) {
        Lexer lexer{a_path};
        return lexer.scan();
    }

  private:
    explicit Lexer(const std::string& a_path) : m_path(a_path) {}

    std::vector<intercom::fs::path> scan() {
        std::vector<intercom::fs::path> paths;
        static auto psep = std::string(1, intercom::fs::path::preferred_separator);
        static auto sep = new_seg(psep);

        if (is_sep(next())) {
            paths.emplace_back(sep);
            ++m_index;
        }

        while (!is_end()) {
            if (is_sep(next())) {
                ++m_index;
            } else {
                paths.emplace_back(new_seg(consume_ident()));
            }
        }
        return paths;
    }

    bool is_end() const { return m_index == m_path.size(); }

    char next() const { return m_path[m_index]; }

    std::string consume_ident() {
        size_t start = m_index;
        while (!is_end() && !is_sep(next())) {
            ++m_index;
        }
        return m_path.substr(start, m_index - start);
    }

    static path new_seg(std::string a_str) {
        path path;
        path.m_path = std::move(a_str);
        return path;
    }

  private:
    size_t m_index = 0;
    const std::string& m_path;
};

intercom::fs::path::path(string_type a_path) : m_path(std::move(a_path)), m_segments(Lexer::split_all(m_path)) {}

intercom::fs::path::path(const value_type* a_path) : m_path(a_path), m_segments(Lexer::split_all(m_path)) {}

intercom::fs::path::path(std::vector<path> a_segments)
        : m_path(to_string(a_segments)), m_segments(std::move(a_segments)) {}

bool intercom::fs::path::is_absolute() const noexcept {
    return !is_relative();
}

intercom::fs::path intercom::fs::path::lexically_normal() const {
    decltype(m_segments) normal;

    for (const auto& seg : m_segments) {
        if (seg.native() == ".." && !normal.empty()) {
            normal.pop_back();
        } else if (seg.native() != ".") {
            normal.emplace_back(seg);
        }
    }
    if (normal.empty()) {
        normal.emplace_back(".");
    }
    return normal;
}

intercom::fs::path intercom::fs::path::parent_path() const {
    path path;
    auto parent = this->lexically_normal();

    for (auto it = parent.begin(); it != parent.end(); ++it) {
        if (std::next(it) != parent.end()) {
            path /= *it;
        }
    }
    return path;
}

intercom::fs::path intercom::fs::path::filename() const {
    return m_segments.empty() ? path{m_path} : *--end();
}

intercom::fs::path intercom::fs::path::root_path() const {
    return root_name() / root_directory();
}

intercom::fs::path intercom::fs::path::stem() const {
    auto file = filename();
    if (!file.empty()) {
        const auto& native = file.native();
        if (native == "." || native == "..") {
            return file;
        }

        auto pos = native.rfind('.');
        if (pos != std::string::npos) {
            return native.substr(0, pos);
        }
    }
    return file;
}

intercom::fs::path intercom::fs::path::extension() const noexcept {
    auto file = filename();
    if (!file.empty()) {
        const auto& native = file.native();
        auto pos = native.rfind('.');
        if (pos != std::string::npos) {
            return native.substr(pos);
        }
    }
    return {};
}

intercom::fs::path& intercom::fs::path::replace_extension(const path& replacement) {
    const auto& file = filename().string();

    if (!file.empty()) {
        if (!replacement.empty() && replacement.native().compare(0, 1, ".") != 0) {
            m_path.push_back('.');
        }

        auto pos = file.rfind('.');
        if (pos == std::string::npos) {
            m_path += replacement.m_path;
        } else {
            auto path_pos = m_path.size() - file.size() + pos;
            m_path = m_path.substr(0, path_pos) + replacement.filename().native();
        }
        // this forces the lexer to update the segments
        *this = m_path;
    } else {
        *this = replacement;
    }
    return *this;
}

bool intercom::fs::path::empty() const noexcept {
    return m_path.empty();
}

void intercom::fs::path::clear() noexcept {
    m_path.clear();
    m_segments.clear();
}

int intercom::fs::path::compare(const path& a_path) const noexcept {
    if (m_segments.empty() || a_path.m_segments.empty()) {
        return native().compare(a_path.native());
    }

    auto it1 = begin();
    auto it2 = a_path.begin();

    // Iterate over both paths and compare each segment lexicographically.
    for (; it1 != end() && it2 != a_path.end(); ++it1, ++it2) {
        if (it1->native() != it2->native()) {
            return it1->native().compare(it2->native());
        }
    }
    if (it1 == end()) {
        return it2 == a_path.end() ? 0 : -1;
    }
    return 1;
}

int intercom::fs::path::compare(const string_type& a_path) const {
    return compare(path{a_path});
}

int intercom::fs::path::compare(const value_type* a_path) const {
    return compare(path{a_path});
}

const intercom::fs::path::string_type& intercom::fs::path::native() const noexcept {
    return m_path;
}

intercom::fs::path::operator intercom::fs::path::string_type() const {
    return native();
}

std::string intercom::fs::path::string() const {
    return {m_path.begin(), m_path.end()};
}

const char* intercom::fs::path::c_str() const noexcept {
    return native().c_str();
}

intercom::fs::path intercom::fs::path::operator/(const path& a_path) const {
    if (empty() || a_path.is_absolute()) {
        return a_path;
    }
    path new_path{*this};
    return new_path /= a_path;
}

intercom::fs::path& intercom::fs::path::operator/=(const path& a_path) {
    if (empty() || a_path.is_absolute()) {
        *this = a_path.m_path;
    } else {
        if (m_path.back() != preferred_separator) {
            m_path.push_back(preferred_separator);
        }
        m_path.insert(m_path.end(), a_path.native().begin(), a_path.native().end());

        auto segments = Lexer::split_all(a_path.native());
        m_segments.insert(m_segments.end(), segments.begin(), segments.end());
    }
    return *this;
}

intercom::fs::path& intercom::fs::path::operator+=(const path& a_path) {
    m_path += a_path.native();
    *this = m_path;
    return *this;
}

std::ostream& intercom::fs::operator<<(std::ostream& a_stream, const path& a_path) {
    a_stream << '"' << a_path.native() << '"';
    return a_stream;
}

std::istream& intercom::fs::operator>>(std::istream& a_stream, path& a_path) {
    std::string path;
    a_stream >> path;
    a_path = path;
    return a_stream;
}

bool intercom::fs::create_directories(const path& a_path) {
    path temp;
    for (const auto& seg : a_path) {
        temp /= seg;
        if (!exists(temp) && !create_directory(temp)) {
            return false;
        }
    }
    return true;
}

std::vector<intercom::fs::path::value_type> intercom::fs::read(const intercom::fs::path& a_path) {
    std::ifstream stream{a_path.native()};
    if (!stream.good()) {
        throw std::runtime_error("failed to open file: " + a_path.native());
    }
    return {std::istreambuf_iterator<path::value_type>{stream}, std::istreambuf_iterator<path::value_type>{}};
}

std::string intercom::fs::read_to_string(const path& a_path) {
    auto bytes = fs::read(a_path);
    return {bytes.begin(), bytes.end()};
}

void intercom::fs::write(const path& a_path, string_view data) {
    std::ofstream file(a_path, std::ios_base::out | std::ios_base::binary);
    if (file.is_open()) {
        file << data;
    } else {
        throw std::runtime_error("writing file failed: " + a_path.native());
    }
}

bool intercom::fs::remove_all(const path& a_path) {
    bool ok = true;

    if (is_directory(a_path)) {
        auto files = read_dir(a_path);
        for (const auto& file : files) {
            ok &= remove_all(file);
        }
    }
    ok &= remove(a_path);
    return ok;
}

intercom::fs::path intercom::fs::relative(const path& a_path) {
    auto cwd = current_path();
    return relative(a_path, cwd);
}

intercom::fs::path intercom::fs::relative(const path& a_path, const path& a_base) {
    if (a_path.root_path() != a_base.root_path()) {
        return a_path;
    }

    auto it1 = a_path.begin();
    auto it2 = a_base.begin();
    while (it1 != a_path.end() && it2 != a_base.end() && *it1 == *it2) {
        ++it1;
        ++it2;
    }

    path result;
    for (; it2 != a_base.end(); ++it2) {
        result /= "..";
    }
    for (; it1 != a_path.end(); ++it1) {
        result /= *it1;
    }

    if (result.empty()) {
        result /= ".";
    }
    return result;
}

std::string intercom::fs::tilde_expand_path(const std::string& path, std::string& error) {
    if (path.empty() || path.front() != '~') {
        return "";
    }
#ifdef INTERCOM_PLATFORM_WINDOWS
    const char* home = std::getenv("USERPROFILE");
    const char* homeless_error = "No \"USERPROFILE\" environment variable";
#else
    const char* home = std::getenv("HOME");
    if (!home) {
        auto pwd = getpwuid(getuid());
        if (pwd) {
            home = pwd->pw_dir;
        }
    }
    const char* homeless_error = "No \"HOME\" environment variable";
#endif

    const char second_letter = path.c_str()[1];
    switch (second_letter) {
    case '\0':
        if (!home) {
            error = homeless_error;
            return "";
        }
        return {home};
    case '/':
    case '\\':
        if (!home) {
            error = homeless_error;
            return "";
        }
        return std::string(home) + path.substr(1);
    case '+':
        error = "\"~+\" not supported";
        return "";
    case '-':
        error = "\"~-\" not supported";
        return "";
    case 'N':
        error = "\"~N\" not supported";
        return "";
    default:
#ifdef INTERCOM_PLATFORM_WINDOWS
        error = "\"~{$USER}\" not supported on windows";
        return "";
#else
        auto sep = path.find('/');
        std::string user_name = path.substr(1, sep - 1);
        auto pwd = getpwnam(user_name.c_str());
        if (!pwd) {
            error = std::string("No user named ") + user_name;
            return "";
        }
        home = pwd->pw_dir;
        if (!home) {
            error = "User " + user_name + " has no home working directory";
            return "";
        }
        std::string remaining_path = (sep == std::string::npos) ? "" : path.substr(sep);
        return std::string(home) + remaining_path;
#endif
    }
}

intercom::fs::path::iterator intercom::fs::path::begin() const {
    return m_segments.begin();
}

intercom::fs::path::iterator intercom::fs::path::end() const {
    return m_segments.end();
}

bool intercom::fs::path::operator==(const path& a_rhs) const noexcept {
    return !(*this < a_rhs) && !(a_rhs < *this);
}

bool intercom::fs::path::operator!=(const path& a_rhs) const noexcept {
    return !(*this == a_rhs);
}

bool intercom::fs::path::operator<(const path& a_rhs) const noexcept {
    return compare(a_rhs) < 0;
}

bool intercom::fs::path::operator<=(const path& a_rhs) const noexcept {
    return !(a_rhs < *this);
}

bool intercom::fs::path::operator>(const path& a_rhs) const noexcept {
    return a_rhs < *this;
}

bool intercom::fs::path::operator>=(const path& a_rhs) const noexcept {
    return !(*this < a_rhs);
}
