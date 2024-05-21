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

#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include <climits>
#include <cstring>
#include <stdexcept>

#include "InterCOM/CORBA/Array.h"
#include "InterCOM/detail/filesystem.h"

const intercom::fs::path::value_type intercom::fs::path::preferred_separator = '/';

namespace {
constexpr mode_t ALL_PERMS = 0777;

mode_t get_umask() {
    mode_t mask = umask(0);
    umask(mask);
    return mask;
}

std::string last_error() {
    std::string err{strerror(errno)};
    return "(E" + std::to_string(errno) + ") " + err;
}
}  // namespace

bool intercom::fs::path::is_relative() const noexcept {
    auto str = native();
    return !(str.length() > 0 && str[0] == '/');
}

intercom::fs::path intercom::fs::path::root_name() const {
    // no-op on linux
    return {};
}

intercom::fs::path intercom::fs::path::root_directory() const {
    if (is_absolute()) {
        return std::string(1, preferred_separator);
    }
    return {};
}

intercom::fs::path intercom::fs::absolute(const path& a_path) {
    if (a_path.empty() || a_path.is_absolute()) {
        return a_path;
    }
    return current_path() / a_path;
}

intercom::fs::path intercom::fs::canonical(const path& a_path) {
    auto resolved = realpath(a_path.c_str(), nullptr);
    if (!resolved) {
        throw std::runtime_error{"Failed to canonicalize path: " + last_error()};
    }

    std::string str{resolved};
    free(resolved);
    return str;
}

intercom::fs::path intercom::fs::current_path() {
    char buffer[PATH_MAX];
    auto cwd = getcwd(buffer, sizeof(buffer));
    if (!cwd) {
        throw std::runtime_error{"Failed to get current directory: " + last_error()};
    }
    path result{cwd};
    return result;
}

intercom::fs::path intercom::fs::get_executable_path() {
    corba::Array<char, PATH_MAX> buffer;
    auto len = readlink("/proc/self/exe", buffer.data(), PATH_MAX);
    if (len < 0) {
        throw std::runtime_error{"Failed to get executable path from /proc/self/exe: " + last_error()};
    }
    std::string dataDir(buffer.data(), len);
    intercom::fs::path path(dataDir);
    return path.parent_path().string();
}

void intercom::fs::current_path(const path& a_dir) {
    if (chdir(a_dir.c_str()) != 0) {
        throw std::runtime_error{"Failed to change directory: " + last_error()};
    }
}

bool intercom::fs::create_directory(const path& a_path) {
    mode_t mask = get_umask();
    auto mode = ALL_PERMS & ~mask;
    return mkdir(a_path.c_str(), mode) == 0;
}

bool intercom::fs::exists(const path& a_path) noexcept {
    struct stat buf;
    return stat(a_path.c_str(), &buf) == 0;
}

bool intercom::fs::is_directory(const path& a_path) {
    struct stat buf;
    if (stat(a_path.c_str(), &buf) == 0) {
        return (buf.st_mode & S_IFDIR) != 0;
    }
    return false;
}

bool intercom::fs::is_executable(const path& a_path) {
    return !access(a_path.c_str(), X_OK);
}

intercom::fs::path intercom::fs::temp_directory_path() {
    auto tmpdir = getenv("TMPDIR");
    return tmpdir ? tmpdir : "/tmp";
}

std::vector<intercom::fs::path> intercom::fs::read_dir(const path& a_dir) {
    struct dirent* entry;
    DIR* dir = opendir(a_dir.c_str());

    if (!dir) {
        throw std::runtime_error("Failed to open directory: " + last_error());
    }

    std::vector<path> directories;
    while ((entry = readdir(dir))) {
        if (strcmp(entry->d_name, ".") != 0 && strcmp(entry->d_name, "..") != 0) {
            directories.emplace_back(a_dir / entry->d_name);
        }
    }
    closedir(dir);
    return directories;
}

bool intercom::fs::remove(const path& a_path) {
    if (!exists(a_path)) {
        return false;
    }
    if (::remove(a_path.c_str()) != 0) {
        throw std::runtime_error{"Failed to remove directory: " + last_error()};
    }
    return true;
}
