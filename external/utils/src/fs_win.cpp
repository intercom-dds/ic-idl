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

#include <ctype.h>
#include <shlwapi.h>

#include "InterCOM/detail/filesystem.h"

const intercom::fs::path::value_type intercom::fs::path::preferred_separator = '\\';

namespace {
constexpr static DWORD BUF_SIZE = 256;

std::string last_error() {
    LPTSTR text = nullptr;
    auto err_num = GetLastError();
    FormatMessage(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_IGNORE_INSERTS, nullptr,
                  err_num, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), (LPTSTR)&text, 0, nullptr);

    std::string err{text};
    LocalFree(text);
    return "(E" + std::to_string(err_num) + ") " + err;
}
}  // namespace

bool intercom::fs::path::is_relative() const noexcept {
    // PathIsRelative from the win32 API differs from the requirements of std::filesystem.
    bool has_root = m_path.compare(0, 2, "\\\\") == 0 || m_path.compare(0, 2, "//") == 0;

    if (m_path.size() > 2) {
        if (isalpha(m_path[0]) && m_path[1] == ':') {
            return false;
        }
        if (has_root) {
            return m_path[2] == '/' || m_path[2] == '\\';
        }
    }
    return !has_root;
}

intercom::fs::path intercom::fs::path::root_name() const {
    if (m_path.length() > 1 && isalpha(m_path[0]) && m_path[1] == ':') {
        return m_path.substr(0, 2);
    }
    return {};
}

intercom::fs::path intercom::fs::path::root_directory() const {
    // this behavior is consistent with MSVC
    if (!m_path.empty() && (m_path[0] == '/' || m_path[0] == '\\')) {
        return std::string(1, m_path[0]);
    }
    if (is_absolute() && m_path.length() > 2) {
        return std::string(1, m_path[2]);
    }
    return {};
}

std::vector<intercom::fs::path> intercom::fs::read_dir(const path& a_dir) {
    WIN32_FIND_DATA ffd;
    auto pattern = a_dir.native() + "\\*";
    auto handle = FindFirstFileA(pattern.c_str(), &ffd);

    if (handle == INVALID_HANDLE_VALUE) {
        throw std::runtime_error("Failed to open directory: " + last_error());
    }

    std::vector<path> directories;
    while (FindNextFileA(handle, &ffd) != 0) {
        if (strcmp(ffd.cFileName, ".") != 0 && strcmp(ffd.cFileName, "..") != 0) {
            directories.emplace_back(a_dir / ffd.cFileName);
        }
    }

    if (GetLastError() != ERROR_NO_MORE_FILES) {
        throw std::runtime_error("Failed to read files in directory: " + last_error());
    }
    FindClose(handle);
    return directories;
}

intercom::fs::path intercom::fs::absolute(const path& a_path) {
    TCHAR buffer[BUF_SIZE];
    int res = GetFullPathNameA(a_path.c_str(), BUF_SIZE, buffer, nullptr);
    if (res == 0) {
        throw std::runtime_error("Failed to get absolute path: " + last_error());
    }
    return buffer;
}

intercom::fs::path intercom::fs::canonical(const path& a_path) {
    TCHAR buffer[BUF_SIZE];
    auto handle = CreateFileA(a_path.c_str(), GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING,
                              FILE_ATTRIBUTE_NORMAL, nullptr);

    auto res = GetFinalPathNameByHandleA(handle, buffer, BUF_SIZE, VOLUME_NAME_NT);
    if (res == 0) {
        throw std::runtime_error{"Failed to canonicalize path: " + last_error()};
    }
    CloseHandle(handle);
    return buffer;
}

intercom::fs::path intercom::fs::current_path() {
    TCHAR buffer[BUF_SIZE];
    auto res = GetCurrentDirectoryA(BUF_SIZE, buffer);
    if (res == 0) {
        throw std::runtime_error{"Failed to get current directory: " + last_error()};
    }
    return buffer;
}

intercom::fs::path intercom::fs::get_executable_path() {
    TCHAR buffer[BUF_SIZE];
    auto res = GetModuleFileName(nullptr, buffer, BUF_SIZE);
    if (res == 0) {
        throw std::runtime_error{"Failed to get executable path: " + last_error()};
    }
    std::string dataDir(buffer);
    intercom::fs::path path(dataDir);
    return path.parent_path().string();
}

void intercom::fs::current_path(const path& a_path) {
    if (SetCurrentDirectoryA(a_path.c_str()) == 0) {
        throw std::runtime_error("Failed to change directory: " + last_error());
    }
}

bool intercom::fs::create_directory(const path& a_path) {
    return CreateDirectoryA(a_path.c_str(), nullptr) != 0;
}

bool intercom::fs::exists(const path& a_path) noexcept {
    DWORD attrib = GetFileAttributesA(a_path.c_str());
    return attrib != INVALID_FILE_ATTRIBUTES;
}

bool intercom::fs::is_directory(const path& a_path) {
    DWORD attrib = GetFileAttributesA(a_path.c_str());
    return (attrib & FILE_ATTRIBUTE_DIRECTORY) && (attrib != INVALID_FILE_ATTRIBUTES);
}

bool intercom::fs::is_executable(const path& a_path) {
    DWORD file_type = 0;
    if (!GetBinaryType(a_path.c_str(), &file_type)) {
        return false;
    }
    return file_type == SCS_32BIT_BINARY || file_type == SCS_64BIT_BINARY || file_type == SCS_WOW_BINARY;
}

intercom::fs::path intercom::fs::temp_directory_path() {
    TCHAR buffer[BUF_SIZE];
    auto res = GetTempPathA(BUF_SIZE, buffer);
    if (res == 0) {
        throw std::runtime_error{"Failed to get temp dir: " + last_error()};
    }
    return buffer;
}

bool intercom::fs::remove(const path& a_path) {
    if (!exists(a_path)) {
        return false;
    }
    auto rc = is_directory(a_path) ? RemoveDirectoryA(a_path.c_str()) : DeleteFileA(a_path.c_str());
    if (rc == 0) {
        throw std::runtime_error{"Failed to delete file: " + last_error()};
    }
    return true;
}
