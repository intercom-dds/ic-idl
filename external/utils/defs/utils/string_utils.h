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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
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

#include <cctype>
#include <string_view>
#include <vector>

/// Contains internal string utilities
namespace string_utils {

/// Tokenizes a string exactly like strtok_r(3C)
///
/// This methods maps to the correct method on different platforms.
/// See 'man strtok_r for further information.
char* strtok_r(char* s1, const char* s2, char** lasts);

/// Split a string like strtok, returning result as a sequence of non-empty substrings.
///
/// \param result A sequence of substrings
/// \param str The string to split
/// \param delim A set of zero or more delimiters
/// \param trim_whitespace If true, remove whitespace (isblank()) from start and end of all returned
/// substrings
void split_string(
    std::vector<std::string>& result,
    std::string_view str,
    const char* delim,
    bool trim_whitespace
);

std::pair<std::string, std::string> split_at(std::string_view str, char delim);

/// Takes a string and attempts to parse a double from it.
/// This function forces the locale to be "classic", meaning the text 3.14 will be parsed
/// to a floating number with the value 3.14, while the text 3,14 will be parsed to 3.
/// Note that this function will return false if it receives a string sequence like 3,14,
/// but the value that is passed in as a reference will contain 3.
///
/// @param text The input text.
/// @param value Reference to the resulting value.
/// @return true if it has successfully parsed the entire text.
bool string_to_double(std::string_view text, double& value);

/// Takes a string, which may include one or more numbers and attempts to parse a double from it.
/// This function forces the locale to be "classic", meaning the text 3.14 will be parsed
/// to a floating number with the value 3.14, while the text 3,14 will be parsed to 3. Since the
/// function does not take a stand on if this is the only number in the string it will return true
/// even if it receives 3,14, but the value of the passed reference will be 3.
///
/// @param text The input text.
/// @param value Reference to the resulting value.
/// @param endPtr A pointer to an int which holds the position of the next character after the
/// numerical value. If eof or failure the value will be -1.
/// @return true If it parses the entire text OR a substring of that text.
bool string_to_double_in_string_sequence(
    std::string_view text,
    double& value,
    int* end_ptr = nullptr
);

/// Takes a string and attempts to parse a float from it.
/// This function forces the locale to be "classic", meaning the text 3.14 will be parsed
/// to a floating number with the value 3.14, while the text 3,14 will be parsed to 3.
///
/// @param text The input text.
/// @param value Reference to the resulting value.
/// @return true if it has successfully parsed the entire text.
bool string_to_float(std::string_view text, float& value);

/// Takes a string, which may include one or more numbers and attempts to parse a float from it.
/// This function forces the locale to be "classic", meaning the text 3.14 will be parsed
/// to a floating number with the value 3.14, while the text 3,14 will be parsed to 3. Since the
/// function does not take a stand on if this is the only number in the string it will return true
/// even if it receives 3,14, but the value of the passed reference will be 3.
///
/// @param text The input text.
/// @param value Reference to the resulting value.
/// @param endPtr A pointer to an int which holds the position of the next character after the
/// numerical value. If eof or failure the value will be -1.
/// @return true If it parses the entire text OR a substring of that text.
bool string_to_float_in_string_sequence(
    std::string_view text,
    float& value,
    int* end_ptr = nullptr
);

/// Converts the input string to upper case.
///
/// @param a_str The input string
/// @return A copy of the input string with all lower case characters converted to upper case
std::string to_upper_case(std::string a_str);

/// Converts the input string to lower case.
///
/// @param a_str The input string
/// @return A copy of the input string with all upper case characters converted to lower case
std::string to_lower_case(std::string a_str);

/// Removes all leading and trailing whitespaces from the input string
///
/// @param a_str The input string
/// @return A copy of the input string with all leading and trailing whitespace trimmed
std::string trim_string(const std::string& a_str);

/// Removes all leading and trailing whitespaces from the input string
///
/// @param a_str The input string
/// @return A copy of the input string with all leading and trailing whitespace trimmed
bool compare_ignore_case(const std::string& a_str1, const std::string& a_str2);

/// Converts the input string to a boolean value
///
/// @param a_str The input string
/// @param a_default Default value to be returned if a_str is not recognized as representing a bool
/// @return true if a_str is the string "true", false if a_str is the string "false", a_default
/// otherwise
bool to_boolean(const std::string& a_str, const bool& a_default = false);

/// Converts the input string to a base 8 integer
///
/// @param a_str The input string
/// @param a_default Default value to be returned if a_str is not recognized as representing a base
/// 8 integer
/// @return an integer parsed from a_str if a_str is a base 8 integer, a_default otherwise
int to_octal(const std::string& a_str, const int& a_default = 0);

/// Converts the input string to an unsigned long integer
///
/// @param a_str The input string
/// @param a_default Default value to be returned if a_str is not recognized as representing an
/// unsigned long integer
/// @return an unsigned long integer parsed from a_str if a_str is an unsigned long integer,
/// a_default otherwise
unsigned long to_ulong(const std::string& a_str, const unsigned long& a_default = 0);

/// Converts the input string to an unsigned integer
///
/// @param a_str The input string
/// @param a_default Default value to be returned if a_str is not recognized as representing an
/// unsigned integer
/// @return an unsigned integer parsed from a_str if a_str is an unsigned integer, a_default
/// otherwise
unsigned int to_uint(const std::string& a_str, const unsigned int& a_default = 0);

/// Converts the input string to a signed integer
///
/// @param a_str The input string
/// @param a_default Default value to be returned if a_str is not recognized as representing a
/// signed integer
/// @return a signed integer parsed from a_str if a_str is a signed integer, a_default otherwise
int to_int(const std::string& a_str, const int& a_default = 0);

/// Converts the input string to an unsigned short integer
///
/// @param a_str The input string
/// @param a_default Default value to be returned if a_str is not recognized as representing an
/// unsigned short integer
/// @return an unsigned short integer parsed from a_str if a_str is an unsigned short integer,
/// a_default otherwise
unsigned short to_ushort(const std::string& a_str);

/// Converts the input string to a string formatted for display in html.
/// I.e. special characters like the ampersand is replaced with &amp; so that it will display as an
/// ampersand during display of an html document.
///
/// @param a_src Input string.
/// @return An html representation of a_src.
std::string to_html_encoding(const std::string& a_src);

bool starts_with(std::string_view str, std::string_view prefix);

bool ends_with(std::string_view str, std::string_view suffix);

}  // namespace string_utils

#include "string_utils.ic"  // IWYU pragma: export
