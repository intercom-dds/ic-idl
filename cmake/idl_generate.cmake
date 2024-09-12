# Copyright 2024 KONGSBERG
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
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
# ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
# WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

# User Documentation:

# Passing LANGUAGE <CPP|PYTHON|RUST|IDL|PROTOBUf> to cidl_generate() will cause
# that language to be generated. If no LANGUAGE is specified CPP will be
# generated. Multiple languages in the same statement is not supported.
#
# DESTINATION should be a path to the output directory.
# If no DESTINATION is specified, CMAKE_CURRENT_BINARY_DIR will be used.
# If the DESTINATION is a relative path, it will be explicitly relative
# to CMAKE_CURRENT_BINARY_DIR.
#
# INPUT_IDL should be a list of relative or absolute paths, with or with file
# extensions. The base directory for each INPUT_IDL is added to
# INCLUDE_DIRECTORIES automatically.
#
# INCLUDE_DIRECTORIES are added as preprocessor include directories when
# parsing the INPUT_IDL files. The idl-directory bundled with the InterCOM
# redistributables are automatically included.
#
# FLAGS will be passed as options to CIDL when generating.
# If CIDL_GENERATE_DEFAULT_FLAGS is defined, this will always be prepended to
# all calls in the current scope.
#
# Any unparsed parameters will be added to the INPUT_IDL list.
# "cidl_generate( myType )" will set ${CMAKE_CURRENT_SOURCE_DIR}/myType.idl
# as your input.
#
# The OUTPUT_VAR list will be populated with ${DESTINATION}/<idl-file> and set
# in PARENT_SCOPE. OUTPUT_VAR will default to being named CIDL_GENERATE_OUTPUTS,
# and is assigned in the parent scope. Note that OUTPUT_VAR will be overwritten
# with each call to cidl_generate().
#
# The OUTPUT_ACCUMULATED list will append to the variable, rather than
# overwriting the entire list each time. This is useful for cases where
# switches may change, but you would still like to give multiple outputs
# to the same target. Avoid using the same variable for OUTPUT_VAR (default
# CIDL_GENERATE_OUTPUTS) and OUTPUT_ACCUMULATED, as the former will overwrite
# the latter.
#
# If you know your generated code will not match the output expected by this
# script, use OVERRIDE_OUTPUTS to specify the output manually.

function(IDL_GENERATE)
    set(options IC_BUILD)
    set(oneValueArgs
        LANGUAGE
        DESTINATION
        OUTPUT_VAR
        OUTPUT_ACCUMULATED
    )
    set(multiValueArgs
        INPUT_IDL
        INCLUDE_DIRECTORIES
        FLAGS
        OVERRIDE_OUTPUTS
    )

    cmake_parse_arguments(_IC_GENERATE "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN})

    if(NOT IC_EXE)
        if(TARGET InterCOM::ic_idl)
            set(IC_EXE $<TARGET_FILE:InterCOM::ic_idl>)
        else()
            message(SEND_ERROR "idl_generate could not locate InterCOM::ic_idl target exectuable")
        endif()
    endif()

    if( NOT _IC_GENERATE_DESTINATION )
        set( _IC_GENERATE_DESTINATION ${CMAKE_CURRENT_BINARY_DIR} )
    endif()

    if( NOT IS_ABSOLUTE "${_IC_GENERATE_DESTINATION}" )
        set( _IC_GENERATE_DESTINATION "${CMAKE_CURRENT_BINARY_DIR}/${_IC_GENERATE_DESTINATION}" )
    endif()

    if( InterCOM_DIR )
        list( APPEND _IC_GENERATE_INCLUDE_DIRECTORIES ${InterCOM_DIR}/../idl )
    endif()

    foreach( _DIR ${_IC_GENERATE_INCLUDE_DIRECTORIES} )
        get_filename_component( _DIR "${_DIR}" REALPATH )
        list( APPEND _INCLUDES -I${_DIR} )
    endforeach()

    get_filename_component( _ABS_DESTINATION "${_IC_GENERATE_DESTINATION}" REALPATH )

    if( NOT _IC_GENERATE_LANGUAGE )
        set( _IC_GENERATE_LANGUAGE CPP )
    else()
        if( _IC_GENERATE_LANGUAGE MATCHES ";" )
            message( SEND_ERROR "Multiple output languages not supported." )
        endif()
    endif()

    string( TOUPPER ${_IC_GENERATE_LANGUAGE} _IC_GENERATE_LANGUAGE )

    if( CIDL_GENERATE_DEFAULT_FLAGS )
        list( APPEND _IC_GENERATE_FLAGS ${CIDL_GENERATE_DEFAULT_FLAGS} )
    endif()

    if( _IC_GENERATE_LANGUAGE STREQUAL "PYTHON" )
        list( APPEND _IC_GENERATE_FLAGS --python-out ${_ABS_DESTINATION} )
        list( APPEND _OUTPUT_SUFFIXES .py )
    endif()

    if( _IC_GENERATE_LANGUAGE STREQUAL "RUST" )
        list( APPEND _IC_GENERATE_FLAGS --rust-out ${_ABS_DESTINATION} )
        list( APPEND _OUTPUT_SUFFIXES .rs )
    endif()

    if( _IC_GENERATE_LANGUAGE STREQUAL "CPP" )
        list( APPEND _IC_GENERATE_FLAGS --cpp-out ${_ABS_DESTINATION} )
        list( APPEND _OUTPUT_SUFFIXES .cpp .h )
    endif()

    if( _IC_GENERATE_LANGUAGE STREQUAL "IDL" )
         list( APPEND _IC_GENERATE_FLAGS --idl-destination ${_ABS_DESTINATION} )
         list( APPEND _OUTPUT_SUFFIXES .idl )
    endif()

    if( _IC_GENERATE_LANGUAGE STREQUAL "PROTOBUF" )
         list( APPEND _IC_GENERATE_FLAGS --idl-destination ${_ABS_DESTINATION} )
         list( APPEND _OUTPUT_SUFFIXES .idl )
    endif()

    if( DEFINED ENV{LD_LIBRARY_PATH} )
        list( APPEND _IC_RPATH $ENV{LD_LIBRARY_PATH} )
    endif()

    if( CMAKE_BUILD_RPATH )
        list( APPEND _IC_RPATH ${CMAKE_BUILD_RPATH} )
    endif()

    if( _IC_RPATH )
        string( REPLACE ";" ":"  _IC_RPATH "${_IC_RPATH}" )
        list( APPEND _ENV_CMD_ARGS LD_LIBRARY_PATH=${_IC_RPATH} )
    endif()

    if( _IC_GENERATE_INTERCOM_BUILD OR _IC_RPATH )
        set( _ENV_CMD ${CMAKE_COMMAND} -E env ${_ENV_CMD_ARGS} )
    endif()

    if( _IC_GENERATE_DOC_IDL )
        get_filename_component( _DOC_FILE_NAME "${_IC_GENERATE_DOC_IDL}" NAME )
        get_filename_component( _DOC_FILE_DIRECTORY "${_IC_GENERATE_DOC_IDL}" DIRECTORY )
        get_filename_component( _DOC_FILE_DIRECTORY "${_DOC_FILE_DIRECTORY}" REALPATH )

        if( "${_DOC_FILE_NAME}" MATCHES "\\.idl$" )
            get_filename_component( _DOC_FILE_BASENAME "${_IC_GENERATE_DOC_IDL}" NAME_WE )
      elseif(NOT IS_DIRECTORY "${_IC_GENERATE_DOC_IDL}")
            set( _DOC_FILE_BASENAME ${_DOC_FILE_NAME} )
            set( _DOC_FILE_NAME ${_DOC_FILE_BASENAME}.idl )
        endif()
    endif()

    if( NOT CIDL_L_EXE AND TARGET InterCOM::cidl )
        get_property( CIDL_L_EXE TARGET InterCOM::cidl PROPERTY LOCATION )
    endif()

    foreach( _INPUT ${_IC_GENERATE_INPUT_IDL} )

        get_filename_component( _FILE_NAME "${_INPUT}" NAME )
        get_filename_component( _ABS_DIRECTORY "${_INPUT}" DIRECTORY )
        get_filename_component( _ABS_DIRECTORY "${_ABS_DIRECTORY}" REALPATH )

        set( _INPUT_INCLUDES ${_INCLUDES} -I ${_ABS_DIRECTORY} )

        if( "${_FILE_NAME}" MATCHES "\\.idl$" )
            get_filename_component( _FILE_BASENAME "${_INPUT}" NAME_WE )
        elseif(NOT IS_DIRECTORY "${_INPUT}")
            set( _FILE_BASENAME ${_FILE_NAME} )
            set( _FILE_NAME ${_FILE_BASENAME}.idl )
        endif()

        set( _ABS_INPUT ${_ABS_DIRECTORY}/${_FILE_NAME} )

        set( _IC_ARGS ${_INPUT_INCLUDES} ${_IC_GENERATE_FLAGS} ${_ABS_INPUT} )

        unset( _ABS_OUTPUT )

        string( REPLACE ";" " " _ENV_CMD_PRETTY "${_ENV_CMD}" )
        string( REPLACE ";" " " _IC_ARGS_PRETTY "${_IC_ARGS}" )

        if( _IC_GENERATE_OVERRIDE_OUTPUTS )
            set( _ABS_OUTPUT ${_IC_GENERATE_OVERRIDE_OUTPUTS} )
        elseif(CIDL_L_EXE)
            execute_process(
                COMMAND
                    ${_ENV_CMD} ${CIDL_L_EXE} ${_IC_ARGS} -l
                OUTPUT_VARIABLE
                    _IC_FILE_LIST
                ERROR_VARIABLE
                    _IC_ERROR
                RESULT_VARIABLE
                    _IC_RESULT
                OUTPUT_STRIP_TRAILING_WHITESPACE )
            if( NOT _IC_RESULT EQUAL 0 )
                message( FATAL_ERROR "[${CIDL_L_EXE} -l ${_IC_ARGS_PRETTY}] returned ${_IC_RESULT}: ${_IC_ERROR}" )
            endif()

            if( _IC_FILE_LIST STREQUAL "" )
                message( FATAL_ERROR "[${CIDL_L_EXE} -l ${_IC_ARGS_PRETTY}] returned no output" )
            endif()

            string( REGEX REPLACE "\n" ";" _IC_FILE_LIST "${_IC_FILE_LIST}")

            foreach( _OUTPUT_FILE ${_IC_FILE_LIST} )
                list( APPEND _ABS_OUTPUT ${_OUTPUT_FILE} )
            endforeach()
        else()
            message(FATAL_ERROR "Unable to locate the ic-idl executable ${CIDL_L_EXE}")
        endif()

        list( APPEND _OUTPUT_LIST ${_ABS_OUTPUT} )

        if( CIDL_GENERATE_TRACE )
            message( "" )
            message( "INPUT: ${_INPUT}" )
            message( "INPUT_INCLUDES: ${_INPUT_INCLUDES}" )
            message( "FILE_BASENAME: ${_FILE_BASENAME}" )
            message( "FILE_NAME: ${_FILE_NAME}" )
            message( "ABS_DIRECTORY: ${_ABS_DIRECTORY}" )
            message( "ABS_INPUT: ${_ABS_INPUT}" )
            message( "ABS_OUTPUT: ${_ABS_OUTPUT}" )

            message( "COMMAND: ${_ENV_CMD_PRETTY} ${IC_EXE} ${_IC_ARGS_PRETTY}" )
            message( "DEPENDS: ${IC_EXE} ${_ABS_INPUT}" )
        endif()

        # Messing with a variable we don't need anymore (_FILE_BASENAME) for prettier comments
        if( _DOC_FILE_BASENAME )
            set( _FILE_BASENAME "${_FILE_BASENAME} + ${_DOC_FILE_BASENAME}" )
        endif()

        add_custom_command( OUTPUT ${_ABS_OUTPUT}
            COMMAND
                ${CMAKE_COMMAND} -E make_directory ${_ABS_DESTINATION}
            COMMAND
                ${_ENV_CMD} ${IC_EXE} ${_IC_ARGS}
            MAIN_DEPENDENCY
                ${_ABS_INPUT}
            DEPENDS
                ${IC_EXE}
            WORKING_DIRECTORY
                ${_ABS_DIRECTORY}
            COMMENT
                "[CIDL][${_FILE_BASENAME}] -> ${_ABS_DESTINATION}" )
    endforeach()

    list( REMOVE_DUPLICATES _OUTPUT_LIST )

    if( NOT _IC_GENERATE_OUTPUT_VAR )
        set( _IC_GENERATE_OUTPUT_VAR CIDL_GENERATE_OUTPUTS )
    endif()

    # NB: CIDL_GENERATE_OUTPUTS is only defined for the PARENT_SCOPE
    set( ${_IC_GENERATE_OUTPUT_VAR} ${_OUTPUT_LIST} PARENT_SCOPE )

    if( _IC_GENERATE_OUTPUT_ACCUMULATED )
        list(APPEND ${_IC_GENERATE_OUTPUT_ACCUMULATED} ${_OUTPUT_LIST})
        set(${_IC_GENERATE_OUTPUT_ACCUMULATED} ${${_IC_GENERATE_OUTPUT_ACCUMULATED}} PARENT_SCOPE)
    endif()
endfunction()
