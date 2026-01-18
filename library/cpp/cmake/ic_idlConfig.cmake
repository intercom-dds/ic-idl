# Copyright 2026 KONGSBERG
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# 1. Redistributions of source code must retain the above copyright notice,
#    this list of conditions and the following disclaimer.
#
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
#
# 3. Neither the name of the copyright holder nor the names of its contributors
#    may be used to endorse or promote products derived from this software
#    without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

find_program(IC_IDL_EXECUTABLE
    NAMES ic-idl
    PATHS ${CMAKE_CURRENT_LIST_DIR}/../bin
    NO_DEFAULT_PATH
    DOC "Path to the ic-idl compiler"
)

if(IC_IDL_EXECUTABLE)
    add_executable(ic_idl::ic-idl IMPORTED)
    set_target_properties(ic_idl::ic-idl PROPERTIES
        IMPORTED_LOCATION "${IC_IDL_EXECUTABLE}"
    )
    message(STATUS "Found ic-idl: ${IC_IDL_EXECUTABLE}")
else()
    message(FATAL_ERROR "ic-idl compiler not found. Set IC_IDL_EXECUTABLE manually.")
endif()

add_library(ic_idl::ic_cts INTERFACE IMPORTED)

set_target_properties(ic_idl::ic_cts PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_LIST_DIR}/../include"
    INTERFACE_COMPILE_FEATURES cxx_std_17
)

include("${CMAKE_CURRENT_LIST_DIR}/idl_generate.cmake")
