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

# Passing LANGUAGE <CPP|CSHARP|JAVA|ADA> to cidl_generate() will cause that language to be generated.
# There is no need to add +c, +cs, +j or +a to the FLAGS parameter.
# If no LANGUAGE is specified CPP will be generated.
# Multiple languages in the same statement is not supported.

# DESTINATION should be a path to the output directory.
# If no DESTINATION is specified, CMAKE_CURRENT_BINARY_DIR will be used.
# If the DESTINATION is a relative path, it will be explicitly relative to CMAKE_CURRENT_BINARY_DIR.

# INPUT_IDL should be a list of relative or absolute paths, with or with file extensions.
# The base directory for each INPUT_IDL is added to INCLUDE_DIRECTORIES automatically.

# INCLUDE_DIRECTORIES are added as preprocessor include directories when parsing the INPUT_IDL files.
# The idl-directory bundled with the InterCOM redistributables are automatically included.

# FLAGS will be passed as options to CIDL when generating.
# If CIDL_GENERATE_DEFAULT_FLAGS is defined, this will always be prepended to all calls in the current scope.

# Any unparsed parameters will be added to the INPUT_IDL list.
# "cidl_generate( myType )" will set ${CMAKE_CURRENT_SOURCE_DIR}/myType.idl as your input.

# The OUTPUT_VAR list will be populated with ${DESTINATION}/<idl-basename>.<ext> and set in PARENT_SCOPE.
# OUTPUT_VAR will default to being named CIDL_GENERATE_OUTPUTS, and is assigned in the parent scope.
# Note that OUTPUT_VAR will be overwritten with each call to cidl_generate().

# The OUTPUT_ACCUMULATED list will append to the variable, rather than overwriting the entire list each time.
# This is useful for cases where switches may change, but you would still like to give multiple outputs to the same target.
# Avoid using the same variable for OUTPUT_VAR (default CIDL_GENERATE_OUTPUTS) and OUTPUT_ACCUMULATED, as the former will overwrite the latter.

# If you know your generated code will not match the output expected by this script, use OVERRIDE_OUTPUTS to specify the output manually.

# Setting CIDL_GENERATE_TRACE=TRUE before calling cidl_generate() will print debugging info, including inputs, destinations and full commands per input file.

# cmake_minimum_required( VERSION 3.5 ) # cmake_parse_arguments

function( CIDL_GENERATE )
   set( options INTERCOM_BUILD )
   set( oneValueArgs
         LANGUAGE
         DESTINATION
         OUTPUT_VAR
         OUTPUT_ACCUMULATED
         ALLOW_REDECLARE
         DOC_IDL )
   set( multiValueArgs
         INPUT_IDL
         INCLUDE_DIRECTORIES
         FLAGS
         OVERRIDE_OUTPUTS )

   cmake_parse_arguments( _CIDL_GENERATE "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN} )

   if( NOT CIDL_EXE )
      if( TARGET InterCOM::cidl )
         set( CIDL_EXE $<TARGET_FILE:InterCOM::cidl> )
      else()
         message( SEND_ERROR "cidl_generate could not locate InterCOM::cidl target exectuable" )
      endif()
   endif()

   foreach( ARG ${_CIDL_GENERATE_UNPARSED_ARGUMENTS} )

      if(    ARG STREQUAL "CPP"
          OR ARG STREQUAL "CSHARP"
          OR ARG STREQUAL "JAVA"
          OR ARG STREQUAL "ADA"
          OR ARG STREQUAL "PYTHON"
          OR ARG STREQUAL "RUST")

         message( WARNING "Language option arguments to cidl_generate have been deprecated. Please use LANGUAGE <lang> instead." )

         if( ARG STREQUAL "CPP" )
            list( APPEND _CIDL_GENERATE_LANGUAGE CPP )
         endif()
         if( ARG STREQUAL "CSHARP" )
            list( APPEND _CIDL_GENERATE_LANGUAGE CSHARP )
         endif()
         if( ARG STREQUAL "JAVA" )
            list( APPEND _CIDL_GENERATE_LANGUAGE JAVA )
         endif()
         if( ARG STREQUAL "ADA" )
            list( APPEND _CIDL_GENERATE_LANGUAGE ADA )
         endif()
         if( ARG STREQUAL "RUST" )
            list( APPEND _CIDL_GENERATE_LANGUAGE RUST )
         endif()
         if( ARG STREQUAL "PYTHON" )
            list( APPEND _CIDL_GENERATE_LANGUAGE PYTHON )
         endif()
      else()
         list( APPEND _CIDL_GENERATE_INPUT_IDL ${ARG} )
      endif()

   endforeach()

   if( NOT _CIDL_GENERATE_DESTINATION )
      set( _CIDL_GENERATE_DESTINATION ${CMAKE_CURRENT_BINARY_DIR} )
   endif()

   if( NOT IS_ABSOLUTE "${_CIDL_GENERATE_DESTINATION}" )
      set( _CIDL_GENERATE_DESTINATION "${CMAKE_CURRENT_BINARY_DIR}/${_CIDL_GENERATE_DESTINATION}" )
   endif()

   if( InterCOM_DIR )
      list( APPEND _CIDL_GENERATE_INCLUDE_DIRECTORIES ${InterCOM_DIR}/../idl )
   endif()

   foreach( _DIR ${_CIDL_GENERATE_INCLUDE_DIRECTORIES} )
      get_filename_component( _DIR "${_DIR}" REALPATH )
      list( APPEND _INCLUDES -I${_DIR} )
   endforeach()

   if( CIDL_GENERATE_TRACE )
      message( "" )
      message( "Call to cidl_generate:" )
      message( "CIDL_EXE: ${CIDL_EXE}" )
      message( "INPUT_IDL: ${_CIDL_GENERATE_INPUT_IDL}" )
      message( "INCLUDES: ${_INCLUDES}" )
      message( "DESTINATION: ${_CIDL_GENERATE_DESTINATION}" )
   endif()

   get_filename_component( _ABS_DESTINATION "${_CIDL_GENERATE_DESTINATION}" REALPATH )

   if( NOT _CIDL_GENERATE_LANGUAGE )
      set( _CIDL_GENERATE_LANGUAGE CPP )
   else()
      if( _CIDL_GENERATE_LANGUAGE MATCHES ";" )
         message( SEND_ERROR "Multiple output languages not supported." )
      endif()
   endif()

   string( TOUPPER ${_CIDL_GENERATE_LANGUAGE} _CIDL_GENERATE_LANGUAGE )

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "JAVA" )
      list( APPEND _CIDL_GENERATE_FLAGS --java-destination ${_ABS_DESTINATION} )
      # Java will generate code in subfolders for each module level, so this guesswork only works for global module types
      list( APPEND _OUTPUT_SUFFIXES .java )
      list( APPEND _OUTPUT_SUFFIXES Helper.java )
      list( APPEND _OUTPUT_SUFFIXES Holder.java )
   endif()

   if( CIDL_GENERATE_DEFAULT_FLAGS )
      list( APPEND _CIDL_GENERATE_FLAGS ${CIDL_GENERATE_DEFAULT_FLAGS} )
   endif()

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "ADA" )
      list( APPEND _CIDL_GENERATE_FLAGS --ada-destination ${_ABS_DESTINATION} )
      list( APPEND _OUTPUT_SUFFIXES .adb .ads )
   endif()

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "CSHARP" )
      list( APPEND _CIDL_GENERATE_FLAGS --csharp-destination ${_ABS_DESTINATION} )
      list( APPEND _OUTPUT_SUFFIXES .cs )
   endif()

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "PYTHON" )
      list( APPEND _CIDL_GENERATE_FLAGS --python-destination ${_ABS_DESTINATION} )
      list( APPEND _OUTPUT_SUFFIXES .py )
   endif()

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "RUST" )
      list( APPEND _CIDL_GENERATE_FLAGS --rust-destination ${_ABS_DESTINATION} )
      list( APPEND _OUTPUT_SUFFIXES .rs )
   endif()

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "CPP" )
      list( APPEND _CIDL_GENERATE_FLAGS --cpp-destination ${_ABS_DESTINATION} )
      list( APPEND _OUTPUT_SUFFIXES .cpp .h )
   endif()

   if( _CIDL_GENERATE_LANGUAGE STREQUAL "IDL" )
       list( APPEND _CIDL_GENERATE_FLAGS --idl-destination ${_ABS_DESTINATION} )
       list( APPEND _OUTPUT_SUFFIXES .idl )
   endif()

   if( _CIDL_GENERATE_INTERCOM_BUILD )
      list( APPEND _ENV_CMD_ARGS INTERCOM_BUILD=${_CIDL_GENERATE_INTERCOM_BUILD} )
   endif()

   if( _CIDL_GENERATE_ALLOW_REDECLARE )
      list( APPEND _ENV_CMD_ARGS CIDL_ALLOW_REDECLARE=${_CIDL_GENERATE_ALLOW_REDECLARE} )
   endif()

   if( DEFINED ENV{LD_LIBRARY_PATH} )
      list( APPEND _CIDL_RPATH $ENV{LD_LIBRARY_PATH} )
   endif()

   if( CMAKE_BUILD_RPATH )
      list( APPEND _CIDL_RPATH ${CMAKE_BUILD_RPATH} )
   endif()

   if( _CIDL_RPATH )
      string( REPLACE ";" ":"  _CIDL_RPATH "${_CIDL_RPATH}" )
      list( APPEND _ENV_CMD_ARGS LD_LIBRARY_PATH=${_CIDL_RPATH} )
   endif()

   if( _CIDL_GENERATE_INTERCOM_BUILD OR _CIDL_GENERATE_ALLOW_REDECLARE OR _CIDL_RPATH )
      set( _ENV_CMD ${CMAKE_COMMAND} -E env ${_ENV_CMD_ARGS} )
   endif()

   if( CIDL_GENERATE_TRACE )
      message( "FLAGS: ${_CIDL_GENERATE_FLAGS}" )
      message( "ABS_DESTINATION: ${_ABS_DESTINATION}" )
      message( "ENV_CMD: ${_ENV_CMD}" )
   endif()

   if( _CIDL_GENERATE_DOC_IDL )
      get_filename_component( _DOC_FILE_NAME "${_CIDL_GENERATE_DOC_IDL}" NAME )
      get_filename_component( _DOC_FILE_DIRECTORY "${_CIDL_GENERATE_DOC_IDL}" DIRECTORY )
      get_filename_component( _DOC_FILE_DIRECTORY "${_DOC_FILE_DIRECTORY}" REALPATH )

      if( "${_DOC_FILE_NAME}" MATCHES "\\.idl$" )
         get_filename_component( _DOC_FILE_BASENAME "${_CIDL_GENERATE_DOC_IDL}" NAME_WE )
     elseif(NOT IS_DIRECTORY "${_CIDL_GENERATE_DOC_IDL}")
         set( _DOC_FILE_BASENAME ${_DOC_FILE_NAME} )
         set( _DOC_FILE_NAME ${_DOC_FILE_BASENAME}.idl )
      endif()

      set( _DOC_ABS_INPUT ${_DOC_FILE_DIRECTORY}/${_DOC_FILE_NAME} )

      if( NOT EXISTS ${_DOC_ABS_INPUT} )
         message( SEND_ERROR "DOC_IDL \"${_DOC_ABS_INPUT}\" could not be found"  )
      endif()

      if( CIDL_GENERATE_TRACE )
         message( "DOC_IDL: ${_CIDL_GENERATE_DOC_IDL}" )
         message( "DOC_FILE_DIRECTORY: ${_DOC_FILE_DIRECTORY}" )
         message( "DOC_FILE_BASENAME: ${_DOC_FILE_BASENAME}" )
         message( "DOC_FILE_NAME: ${_DOC_FILE_NAME}" )
         message( "DOC_ABS_INPUT: ${_DOC_ABS_INPUT}" )
      endif()
   endif()

   if( NOT CIDL_L_EXE AND TARGET InterCOM::cidl )
      get_property( CIDL_L_EXE TARGET InterCOM::cidl PROPERTY LOCATION )
   endif()

   foreach( _INPUT ${_CIDL_GENERATE_INPUT_IDL} )

      get_filename_component( _FILE_NAME "${_INPUT}" NAME )
      get_filename_component( _ABS_DIRECTORY "${_INPUT}" DIRECTORY )
      get_filename_component( _ABS_DIRECTORY "${_ABS_DIRECTORY}" REALPATH )

      set( _INPUT_INCLUDES ${_INCLUDES} -I${_ABS_DIRECTORY} )

      if( "${_FILE_NAME}" MATCHES "\\.idl$" )
         get_filename_component( _FILE_BASENAME "${_INPUT}" NAME_WE )
     elseif(NOT IS_DIRECTORY "${_INPUT}")
         set( _FILE_BASENAME ${_FILE_NAME} )
         set( _FILE_NAME ${_FILE_BASENAME}.idl )
      endif()

      set( _ABS_INPUT ${_ABS_DIRECTORY}/${_FILE_NAME} )

      set( _CIDL_ARGS ${_INPUT_INCLUDES} ${_CIDL_GENERATE_FLAGS} ${_ABS_INPUT} ${_DOC_ABS_INPUT} )

      unset( _ABS_OUTPUT )

      string( REPLACE ";" " " _ENV_CMD_PRETTY "${_ENV_CMD}" )
      string( REPLACE ";" " " _CIDL_ARGS_PRETTY "${_CIDL_ARGS}" )

      if( _CIDL_GENERATE_OVERRIDE_OUTPUTS )
         set( _ABS_OUTPUT ${_CIDL_GENERATE_OVERRIDE_OUTPUTS} )
      elseif(CIDL_L_EXE)
         execute_process(
            COMMAND
               ${_ENV_CMD} ${CIDL_L_EXE} ${_CIDL_ARGS} -l
            OUTPUT_VARIABLE
               _CIDL_FILE_LIST
            ERROR_VARIABLE
               _CIDL_ERROR
            RESULT_VARIABLE
               _CIDL_RESULT
            OUTPUT_STRIP_TRAILING_WHITESPACE )
         if( NOT _CIDL_RESULT EQUAL 0 )
            message( FATAL_ERROR "[${CIDL_L_EXE} -l ${_CIDL_ARGS_PRETTY}] returned ${_CIDL_RESULT}: ${_CIDL_ERROR}" )
         endif()

         if( _CIDL_FILE_LIST STREQUAL "" )
            message( FATAL_ERROR "[${CIDL_L_EXE} -l ${_CIDL_ARGS_PRETTY}] returned no output" )
         endif()

         string( REGEX REPLACE "\n" ";" _CIDL_FILE_LIST "${_CIDL_FILE_LIST}")

         foreach( _OUTPUT_FILE ${_CIDL_FILE_LIST} )
            list( APPEND _ABS_OUTPUT ${_ABS_DESTINATION}/${_OUTPUT_FILE} )
         endforeach()
      else()
         # Fallback to an educated guess removed as this masks cidl installation problems. This is really
         # only necessary when building cidl the first time during repository bootstrap.
         if (NOT INTERCOM_BUILD_HELPER_TOOLS)
            message(FATAL_ERROR "Unable to locate cidl executable ${CIDL_L_EXE} - IDL dependencies must be specified with OVERRIDE_OUTPUTS")
         else()
            foreach( _SUFFIX ${_OUTPUT_SUFFIXES} )
               list( APPEND _ABS_OUTPUT ${_ABS_DESTINATION}/${_FILE_BASENAME}${_SUFFIX} )
            endforeach()
         endif()
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

         message( "COMMAND: ${_ENV_CMD_PRETTY} ${CIDL_EXE} ${_CIDL_ARGS_PRETTY}" )
         message( "DEPENDS: ${CIDL_EXE} ${_ABS_INPUT} ${_DOC_ABS_INPUT}" )
      endif()

      # Messing with a variable we don't need anymore (_FILE_BASENAME) for prettier comments
      if( _DOC_FILE_BASENAME )
         set( _FILE_BASENAME "${_FILE_BASENAME} + ${_DOC_FILE_BASENAME}" )
      endif()

      add_custom_command( OUTPUT ${_ABS_OUTPUT}
         COMMAND
            ${CMAKE_COMMAND} -E make_directory ${_ABS_DESTINATION}
         COMMAND
            ${_ENV_CMD} ${CIDL_EXE} ${_CIDL_ARGS}
         MAIN_DEPENDENCY
            ${_ABS_INPUT}
         DEPENDS
            ${CIDL_EXE}
            ${_DOC_ABS_INPUT}
         WORKING_DIRECTORY
            ${_ABS_DIRECTORY}
         COMMENT
            "[CIDL][${_FILE_BASENAME}] -> ${_ABS_DESTINATION}" )
   endforeach()

   list( REMOVE_DUPLICATES _OUTPUT_LIST )

   if( NOT _CIDL_GENERATE_OUTPUT_VAR )
      set( _CIDL_GENERATE_OUTPUT_VAR CIDL_GENERATE_OUTPUTS )
   endif()

   # NB: CIDL_GENERATE_OUTPUTS is only defined for the PARENT_SCOPE
   set( ${_CIDL_GENERATE_OUTPUT_VAR} ${_OUTPUT_LIST} PARENT_SCOPE )

   if( _CIDL_GENERATE_OUTPUT_ACCUMULATED )
      list( APPEND ${_CIDL_GENERATE_OUTPUT_ACCUMULATED} ${_OUTPUT_LIST} )
      set( ${_CIDL_GENERATE_OUTPUT_ACCUMULATED} ${${_CIDL_GENERATE_OUTPUT_ACCUMULATED}} PARENT_SCOPE )
   endif()

   if( CIDL_GENERATE_TRACE )
      message( "" )
      message( "${_CIDL_GENERATE_OUTPUT_VAR}: ${_OUTPUT_LIST}" )
   endif()

endfunction()
