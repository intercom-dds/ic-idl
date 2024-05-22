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

%{

#include <stddef.h>
#include <string.h>
#include <stdio.h>
#include <cidl/ptree_builder.h>

int idllex(void);
int idlerror( const char * s );

extern int idl_subtype_count;

int parser_has_error();
void parse_warning( const char* msg, const char* file_name, int line );
void parse_error( const char* msg, const char* file_name, int line );
static void pedantic_enum(const struct ptree* node);
static void pedantic_bitmask(const struct ptree* node);

struct identifier create_identifier( const char *name );

extern struct ptree g_top_level;

%}

%debug

%union {
    const struct numeric* num;
    struct declarator* decl;
    struct identifier ident;
    struct ptree* node;
    struct position pos;
}
                                               
%token                  NULL_LITERAL MODULE SWITCH UNION ENUM CASE DEFAULT STRUCT NATIVE TYPEDEF ANNOTATION
%token                  EXCEPTION LOCAL INTERFACE IN OUT INOUT RAISES GETRAISES SETRAISES ATTRIBUTE READONLY
%token                  VALUETYPE SUPPORTS PUBLIC PRIVATE FACTORY
%token                  CONST DOUBLECOLON LEFT_SHIFT RIGHT_SHIFT
%token                  IDL_UNSIGNED IDL_OCTET IDL_INT8 IDL_BOOLEAN IDL_CHAR IDL_WCHAR IDL_SHORT IDL_LONG IDL_FLOAT IDL_DOUBLE
%token                  IDL_ANY IDL_VOID IDL_OBJECT IDL_LONGLONG IDL_USHORT IDL_ULONG IDL_ULONGLONG
%token                  SEQUENCE STRING WSTRING FIXED MAP BITSET BITFIELD BITMASK
%token  <ident>         DOXY_COMMENT 
%token  <ident>         DOXY_COMMENT_POST
%token  <pos>           BRACE_BEGIN BRACE_END
%token  <ident>         IDENTIFIER
%token  <ident>         ANNOTATION_IDENT
%token  <ident>         ANNOTATION_IDENT_POST
%token  <ident>         ISOLATED_ANNOTATION_IDENT_POST
%token  <ident>         INCLUDE_BEGIN
%token                  INCLUDE_END
%token  <num>           STRING_LITERAL
%token  <num>           CHAR_LITERAL
%token  <num>           CONSTANT

%type   <ident>         name_or_anon
%type   <ident>         scoped_name
%type   <ident>         interface_name
%type   <ident>         value_name

%type   <node>          include_dcl

%type   <node>          doxy_comment 
%type   <node>          doxy_comments
%type   <node>          specification
%type   <node>          definitions
%type   <node>          definition
%type   <node>          module_dcl
%type   <node>          const_dcl
%type   <node>          complex_const_list_elem
%type   <node>          const_expr_list
%type   <node>          type_dcl
%type   <node>          const_type
%type   <node>          constr_type_dcl
%type   <node>          native_dcl
%type   <node>          typedef_dcl
%type   <node>          type_declarator
%type   <node>          type_spec
%type   <node>          simple_type_spec
%type   <node>          base_type_spec
%type   <node>          floating_pt_type
%type   <node>          integer_type
%type   <node>          signed_int
%type   <node>          unsigned_int
%type   <node>          char_type
%type   <node>          wide_char_type
%type   <node>          boolean_type
%type   <node>          octet_type
%type   <node>          any_const_type
%type   <node>          any_type
%type   <node>          object_type
%type   <node>          fixed_pt_const_type
%type   <node>          destination_type

%type   <node>          template_type_spec
%type   <node>          sequence_type
%type   <node>          map_type
%type   <node>          bitset_dcl
%type   <node>          bitmask_dcl
%type   <node>          string_type
%type   <node>          wide_string_type
%type   <node>          fixed_pt_type
%type   <node>          struct_dcl
%type   <node>          struct_def
%type   <node>          struct_forward_dcl
%type   <node>          members
%type   <node>          member
%type   <node>          union_dcl
%type   <node>          union_def
%type   <node>          switch_type_spec
%type   <node>          switch_body
%type   <node>          case_
%type   <node>          case_labels
%type   <node>          case_label
%type   <node>          element_spec
%type   <node>          union_forward_dcl
%type   <node>          enum_dcl
%type   <node>          enumerators
%type   <node>          enumerator
%type   <node>          bitfields
%type   <node>          bitfield
%type   <node>          bit_values
%type   <node>          bit_value

%type   <ident>         annotation_appl_comment_ident

%type   <node>          annotation_dcl
%type   <node>          annotation_body
%type   <node>          annotation_inner
%type   <node>          annotation_member
%type   <node>          annotation_member_type
%type   <node>          annotations
%type   <node>          annotation_appl
%type   <node>          annotation_appl_comment
%type   <node>          annotation_appl_params
%type   <node>          annotation_appl_param_list
%type   <node>          annotation_appl_param

%type   <node>          except_dcl
%type   <node>          interface_dcl
%type   <node>          interface_def
%type   <node>          interface_forward_dcl
%type   <node>          interface_body
%type   <node>          export
%type   <node>          op_dcl
%type   <node>          op_type_spec
%type   <node>          parameter_dcls
%type   <node>          param_dcl_doxy
%type   <node>          param_dcl
%type   <node>          attr_dcl

%type   <node>          value_dcl
%type   <node>          value_def
%type   <node>          value_header
%type   <node>          value_element
%type   <node>          value_elements
%type   <node>          state_member
%type   <node>          init_dcl
%type   <node>          init_param_dcls
%type   <node>          init_param_dcl
%type   <node>          value_forward_dcl
                                                
%type   <decl>          interface_inheritance_spec
%type   <decl>          raises_expr_or_empty
%type   <decl>          raises_expr
%type   <decl>          get_excep_expr
%type   <decl>          set_excep_expr
%type   <decl>          declarator
%type   <decl>          declarators
%type   <decl>          any_declarator
%type   <decl>          any_declarators
%type   <decl>          simple_declarator
%type   <decl>          simple_declarators
%type   <decl>          fixed_array_sizes
%type   <decl>          array_declarator
%type   <decl>          identifiers
%type   <decl>          interface_names

%type   <num>           const_expr
%type   <num>           complex_const_expr
%type   <num>           positive_int_const
%type   <num>           or_expr
%type   <num>           xor_expr
%type   <num>           and_expr
%type   <num>           shift_expr
%type   <num>           add_expr
%type   <num>           mult_expr
%type   <num>           unary_expr
%type   <num>           primary_expr
%type   <num>           fixed_array_size

%start specification
%%

/* Rule 1 */
specification: definitions { append_node( &g_top_level, $1 ); };

/* Rule 2 */
definitions: /* empty */ { $$ = NULL; }
        |       definitions definition { $$ = append_node( $1, $2 ); if ( parser_has_error() ) { YYERROR; } }
        |       definitions annotation_appl_comment { $$ = annotate_last( $1, $2 ); };

/* Rule 2, 71, 98 and 208 */
definition:     module_dcl ';'
        |       const_dcl ';'
        |       type_dcl ';'
        |       annotation_dcl ';'
        |       except_dcl ';'
        |       value_dcl ';'
        |       interface_dcl ';'
        |       include_dcl
        |       doxy_comment
        |       annotation_appl;

include_dcl:    INCLUDE_BEGIN { create_include_start( $1 ); } definitions INCLUDE_END { $$ = create_include_finish( $3 ); };

/* Rule 3 */
module_dcl:     MODULE IDENTIFIER { create_module_start( $2 ); } BRACE_BEGIN definitions BRACE_END { $$ = create_module_finish( $5, $6 ); };

/* Rule 4 */
scoped_name:    IDENTIFIER
        |       DOUBLECOLON IDENTIFIER { $$ = build_scoped_name( create_identifier(""), $2 ); }
        |       scoped_name DOUBLECOLON IDENTIFIER { $$ = build_scoped_name( $1, $3 ); };

/* Rule 5 */
const_dcl:      CONST const_type declarator '=' complex_const_expr { $$ = create_const_node( $3, $2, $5 ); }
        |       CONST const_type declarator { $$ = create_const_node( $3, $2, &num_undef ); }

complex_const_expr:
                const_expr { $$ = create_value_node( $1, NULL ); }
        |       BRACE_BEGIN BRACE_END { $$ = create_value_node( &num_undef, NULL ); }
        |       BRACE_BEGIN const_expr_list BRACE_END
        { $$ = create_value_node( &num_undef, $2 ); parse_pedantic(NULL, "Complex constants are an InterCOM extension"); };

complex_const_list_elem:
                complex_const_expr { $$ = create_const_node( NULL, NULL, $1 ); }
        |       IDENTIFIER '=' complex_const_expr { $$ = create_const_node( create_decl( $1, NULL ), NULL, $3 ); }

const_expr_list:complex_const_list_elem { $$ = $1; }
        |       const_expr_list ',' complex_const_list_elem { $$ = append_node( $1, $3 ); };

/* Rule 6, plus sequence and map types for old cidl compatibility */
const_type:     integer_type
        |       floating_pt_type
        |       fixed_pt_const_type
        |       char_type
        |       wide_char_type
        |       boolean_type
        |       octet_type
        |       string_type
        |       wide_string_type
        |       sequence_type
        |       map_type
        |       scoped_name { $$ = lookup_node( $1 ); };

/* Rule 7 */
const_expr:     or_expr;

/* Rule 8 */
or_expr:        xor_expr
        |       or_expr '|' xor_expr { $$ = expr_binary( '|', $1, $3 ); };

/* Rule 9 */
xor_expr:       and_expr
        |       xor_expr '^' and_expr { $$ = expr_binary( '^', $1, $3 ); };

/* Rule 10 */
and_expr:       shift_expr
        |       and_expr '&' shift_expr { $$ = expr_binary( '&', $1, $3 ); };

/* Rule 11 */
shift_expr:     add_expr
        |       shift_expr LEFT_SHIFT add_expr { $$ = expr_binary( '<', $1, $3 ); }
        |       shift_expr RIGHT_SHIFT add_expr { $$ = expr_binary( '>', $1, $3 ); };

/* Rule 12 */
add_expr:       mult_expr
        |       add_expr '+' mult_expr { $$ = expr_binary( '+', $1, $3 ); }
        |       add_expr '-' mult_expr { $$ = expr_binary( '-', $1, $3 ); };

/* Rule 13 */
mult_expr:      unary_expr
        |       mult_expr '*' unary_expr { $$ = expr_binary( '*', $1, $3 ); }
        |       mult_expr '/' unary_expr { $$ = expr_binary( '/', $1, $3 ); }
        |       mult_expr '%' unary_expr { $$ = expr_binary( '%', $1, $3 ); };

/* Rule 14 and 15 */
unary_expr:     '+' primary_expr { $$ = $2; }
        |       '-' primary_expr { $$ = expr_unary( '-', $2 ); }
        |       '~' primary_expr { $$ = expr_unary( '~', $2 ); }
        |       primary_expr;

/* Rule 16, 17 and 18 (literals handled by lexer) */
primary_expr:   scoped_name { $$ = lookup_value( $1 ); }
        |       STRING_LITERAL
        |       CHAR_LITERAL
        |       CONSTANT
        |       '(' const_expr ')' { $$ = $2; };

/* Rule 19 */
positive_int_const:
                const_expr;

/* Rule 20 */
type_dcl:       constr_type_dcl
        |       native_dcl
        |       typedef_dcl;

/* Rule 21 and 206 */
type_spec:      simple_type_spec
        |       template_type_spec;

/* Rule 22 */
simple_type_spec:
                base_type_spec
        |       scoped_name { $$ = lookup_type( $1 ); };

/* Rule 23, 69 and 117 */
base_type_spec: integer_type
        |       floating_pt_type
        |       char_type
        |       wide_char_type
        |       boolean_type
        |       octet_type
        |       any_type
        |       object_type;

/* Rule 70 */
any_type:       IDL_ANY { $$ = &any_type; };

/* Rule 118 */
object_type:    IDL_OBJECT { $$ = &object_type; };

/* Rule 24 */
floating_pt_type:
                IDL_FLOAT { $$ = &float_type; }
        |       IDL_DOUBLE { $$ = &double_type; }
        |       IDL_LONG IDL_DOUBLE { $$ = &ldouble_type; };

/* Rule 25 */
integer_type:   signed_int
        |       unsigned_int;

/* Rule 26 */
signed_int:     IDL_SHORT { $$ = &short_type; }
        |       IDL_LONG { $$ = &long_type; }
        |       IDL_LONG IDL_LONG { $$ = &longlong_type; }
        |       IDL_LONGLONG { $$ = &longlong_type; };

/* Rules 27, 28, 29, 31, 32, 33 are integer names (handled by lexer) */

/* Rule 30 */
unsigned_int:   IDL_UNSIGNED IDL_SHORT { $$ = &ushort_type; }
        |       IDL_UNSIGNED IDL_LONG { $$ = &ulong_type; }
        |       IDL_UNSIGNED IDL_LONG IDL_LONG { $$ = &ulonglong_type; }
        |       IDL_USHORT { $$ = &ushort_type; }
        |       IDL_ULONG { $$ = &ulong_type; }
        |       IDL_ULONGLONG { $$ = &ulonglong_type; };

/* Rule 34 */
char_type:      IDL_CHAR { $$ = &char_type; };

/* Rule 35 */
wide_char_type: IDL_WCHAR { $$ = &wchar_type; };

/* Rule 36 */
boolean_type:   IDL_BOOLEAN { $$ = &boolean_type; };

/* Rule 37 */
octet_type:     IDL_OCTET { $$ = &octet_type; }
        |       IDL_INT8 { $$ = &int8_type; };

/* Rule 38 and 197*/
template_type_spec:
                sequence_type
        |       string_type
        |       wide_string_type
        |       fixed_pt_type
        |       map_type;

/* Rule 39 */
sequence_type:  SEQUENCE '<' annotations type_spec ',' positive_int_const '>' { --idl_subtype_count; $$ = create_sequence( annotate_alias( $4, $3 ), $6 ); }
        |       SEQUENCE '<' annotations type_spec '>' { --idl_subtype_count; $$ = create_sequence( annotate_alias( $4, $3 ), &num_undef ); };

/* Rule 40 */
string_type:    STRING '<' positive_int_const '>' { $$ = create_string( $3 ); }
        |       STRING { $$ = create_string( &num_undef ); };

/* Rule 41 */
wide_string_type:
                WSTRING '<' positive_int_const '>' { $$ = create_wstring( $3 ); }
        |       WSTRING { $$ = create_wstring( &num_undef ); };

/* Rule 42 */
fixed_pt_type:  FIXED '<' positive_int_const ',' positive_int_const '>' { $$ = create_fixed( $3, $5 ); };

/* Rule 43 */
fixed_pt_const_type:
                FIXED { $$ = &fixed_type; };

/* Rule 44 and 198 */
constr_type_dcl:struct_dcl
        |       union_dcl
        |       enum_dcl
        |       bitset_dcl
        |       bitmask_dcl;

/* Rule 45 */
struct_dcl:     struct_def
        |       struct_forward_dcl;

/* Rule 46 and 195 */
struct_def:     STRUCT name_or_anon doxy_comments { append_node(create_struct_start( $2, NULL ), $3); }
                BRACE_BEGIN members BRACE_END { $$ = create_struct_finish( $6, $7 ); }
        |       STRUCT name_or_anon ':' scoped_name doxy_comments { append_node(create_struct_start( $2, lookup_type( $4 ) ), $5); }
                BRACE_BEGIN members BRACE_END { $$ = create_struct_finish( $8, $9 ); };

/* Rule 47 */
members:        /* empty */ { $$ = NULL; }
        |       members member { $$ = append_node( $1, $2 ); }
        |       members annotation_appl_comment { $$ = annotate_last( $1, $2 ); };

/* Rule 47 */
member:         annotations type_spec declarators ';'  { $$ = create_member( $3, $2, $1 ); }
        |       annotations type_dcl ';'               { $$ = annotate( $2, $1 ); }
        |       doxy_comment;

/* Rule 48 */
struct_forward_dcl: STRUCT IDENTIFIER { $$ = create_struct_dcl( $2 ); };

/* Rule 49 */
union_dcl:      union_def
        |       union_forward_dcl;

/* Rule 50 */
union_def:      UNION name_or_anon { create_union_start( $2 ); }
                SWITCH '(' annotations switch_type_spec ')' doxy_comments BRACE_BEGIN switch_body BRACE_END
                { $$ = append_node(create_union_finish( create_member( create_decl( create_identifier("_d"), NULL ), $7, $6 ), $11, $12 ), $9); };

/* Rule 51 and 196 */
switch_type_spec:
                integer_type
        |       char_type
        |       boolean_type
        |       wide_char_type
        |       octet_type
        |       scoped_name { $$ = lookup_type( $1 ); };

/* Rule 52 */
switch_body:    case_
        |       switch_body case_ { $$ = append_node( $1, $2 ); };

/* Rule 53 */
case_:          annotations case_labels element_spec ';'
                { $$ = create_union_member($3, $2, $1); }
        |       doxy_comment;

/* Rule 54 */
case_labels:    case_label
        |       case_labels case_label { $$ = append_node( $1, $2 ); };

/* Rule 54 */
case_label:     CASE const_expr ':' doxy_comments { $$ = append_node(create_case_label( $2 ), $4); }
        |       DEFAULT ':' doxy_comments { $$ = append_node(create_default_case(), $3); };

/* Rule 55 */
element_spec:   annotations type_spec declarator { $$ = create_member( $3, $2, $1 ); }
        |       annotations NULL_LITERAL { $$ = create_null_node(); }

/* Rule 56 */
union_forward_dcl:
                UNION IDENTIFIER { $$ = create_union_dcl( $2 ); };

/* Rule 57 */
enum_dcl:       ENUM name_or_anon BRACE_BEGIN enumerators BRACE_END { $$ = create_enum( $2, $4, $5 ); };

/* Rule 58 */
enumerators:    enumerator { $$ = append_enum_node( NULL, $1 ); }
        |       enumerators ',' enumerator { $$ = append_enum_node( $1, $3 ); };

/* Rule 58 */
enumerator:     doxy_comments annotations IDENTIFIER doxy_comments { 
        $$ = append_node( $1, append_node( annotate( create_enum_value( $3, &num_undef ), $2 ), $4 ) ); }
        |       doxy_comments annotations IDENTIFIER '=' const_expr doxy_comments {
        $$ = append_node( $1, append_node( annotate( create_enum_value( $3, $5 ), $2 ), $6 ) ); pedantic_enum($$); };

/* Rule 59 */
array_declarator:
                simple_declarator fixed_array_sizes { $$ = set_array_bounds( $1, $2 ); };

/* Rule 60 */
fixed_array_sizes:
                fixed_array_size { $$ = append_array_size( NULL, $1 ); }
        |       fixed_array_sizes fixed_array_size { $$ = append_array_size( $1, $2 ); };

/* Rule 60 */
fixed_array_size:
                '[' positive_int_const ']' { $$ = $2; };

/* Rule 61 */
native_dcl:     NATIVE IDENTIFIER { $$ = create_native_type( $2 ); };

/* Rule 62 */
simple_declarator:
                annotations IDENTIFIER { $$ = create_decl( $2, $1 ); };

/* Rule 63 */
typedef_dcl:    TYPEDEF type_declarator { $$ = $2; };

/* Rule 64 */
type_declarator:annotations simple_type_spec any_declarators { $$ = annotate_list( create_type( $3, $2 ), $1 ); }
        |       annotations template_type_spec any_declarators { $$ = annotate_list( create_type( $3, $2 ), $1 ); }
        |       annotations constr_type_dcl any_declarators { $$ = annotate_list( append_node( $2, create_type( $3, $2 ) ), $1 ); };

/* Rule 65 */
any_declarators:any_declarator
        |       any_declarators any_declarator  { $$ = append_decl( $1, $2 ); };

/* Rule 66 */
any_declarator: simple_declarator
        |       array_declarator;

/* Rule 67 */
declarators:    declarator
        |       declarators ',' declarator { $$ = append_decl( $1, $3 ); };

simple_declarators:
                simple_declarator
        |       simple_declarators ',' simple_declarator { $$ = append_decl( $1, $3 ); };

/* Rule 68 and 207 */
declarator:     simple_declarator
        |       array_declarator;

/* Rule 72 */
except_dcl:     EXCEPTION IDENTIFIER { create_exception_start( $2 ); }
                BRACE_BEGIN members BRACE_END { $$ = create_exception_finish( $5, $6 ); };

/* Rule 73 */
interface_dcl:  interface_def
        |       interface_forward_dcl;

/* Rule 74 */
interface_def:  INTERFACE IDENTIFIER { create_interface_start( $2, NULL, 0 ); } BRACE_BEGIN interface_body BRACE_END
                { $$ = create_interface_finish( $5, $6 ); }
        |       INTERFACE IDENTIFIER interface_inheritance_spec { create_interface_start( $2, $3, 0 ); }
                BRACE_BEGIN interface_body BRACE_END { $$ = create_interface_finish( $6, $7 ); }
        |       LOCAL INTERFACE IDENTIFIER { create_interface_start( $3, NULL, 1 ); } BRACE_BEGIN interface_body BRACE_END
                { $$ = create_interface_finish( $6, $7 ); }
        |       LOCAL INTERFACE IDENTIFIER interface_inheritance_spec { create_interface_start( $3, $4, 1); }
                BRACE_BEGIN interface_body BRACE_END { $$ = create_interface_finish( $7, $8 ); };

/* Rule 75 */
interface_forward_dcl:
                INTERFACE IDENTIFIER { $$ = create_interface_dcl( $2, 0 ); }
        |       LOCAL INTERFACE IDENTIFIER { $$ = create_interface_dcl( $3, 1 ); };

/* Rule 76 and 77 part of rule 74 */

/* Rule 78 */
interface_inheritance_spec:
                ':' interface_names { $$ = $2; };

/* Rule 79 */
interface_name: scoped_name;

interface_names:interface_name { $$ = create_decl( $1, NULL ); }
        |       interface_names ',' interface_name { $$ = append_decl( $1, create_decl( $3, NULL )) ; }

/* Rule 80 */
interface_body: /* empty */ { $$ = NULL; }
        |       interface_body export { $$ = append_node( $1, $2 ); }

/* Rule 81 */
export:         annotations op_dcl ';' { $$ = annotate( $2, $1 ); }
        |       annotations attr_dcl ';' { $$ = annotate( $2, $1 ); }
        |       annotations type_dcl ';' { $$ = annotate( $2, $1 ); }
        |       annotations except_dcl ';' { $$ = annotate( $2, $1 ); }
        |       annotations const_dcl ';' { $$ = annotate( $2, $1 ); }
        |       doxy_comment;

/* Rule 82 */
op_dcl:         op_type_spec annotations IDENTIFIER '(' parameter_dcls ')' raises_expr_or_empty
                { $$ = annotate( create_interface_op( $3, $5, $1, $7 ), $2 ); }
        |       op_type_spec annotations IDENTIFIER '(' ')' raises_expr_or_empty
                { $$ = annotate( create_interface_op( $3, NULL, $1, $6 ), $2 ); };

op_type_spec:   type_spec
        |       IDL_VOID { $$ = NULL; };

parameter_dcls: param_dcl_doxy
        |       parameter_dcls ',' param_dcl_doxy { $$ = append_node( $1, $3 ); };

param_dcl_doxy: doxy_comments param_dcl doxy_comments { $$ = append_node( $1, append_node( $2, $3 ) );};

param_dcl:            type_spec simple_declarator { $$ = create_param_dcl( $2, $1, OPT_IN ); }
        |       IN    type_spec simple_declarator { $$ = create_param_dcl( $3, $2, OPT_IN ); }
        |       OUT   type_spec simple_declarator { $$ = create_param_dcl( $3, $2, OPT_OUT ); }
        |       INOUT type_spec simple_declarator { $$ = create_param_dcl( $3, $2, OPT_INOUT ); };

raises_expr_or_empty:
                /* empty */ { $$ = NULL; }
        |       raises_expr;

raises_expr:    RAISES '(' interface_names ')' { $$ = $3; };

attr_dcl:       READONLY ATTRIBUTE type_spec simple_declarator raises_expr
                { $$ = create_attribute( $4, $3, $5, NULL, 1 ); }
        |       READONLY ATTRIBUTE type_spec simple_declarators
                { $$ = create_attribute( $4, $3, NULL, NULL, 1 ); }
        |       ATTRIBUTE type_spec simple_declarator get_excep_expr set_excep_expr
                { $$ = create_attribute( $3, $2, $4, $5, 0 ); }
        |       ATTRIBUTE type_spec simple_declarator get_excep_expr 
                { $$ = create_attribute( $3, $2, $4, NULL, 0 ); }
        |       ATTRIBUTE type_spec simple_declarator set_excep_expr
                { $$ = create_attribute( $3, $2, NULL, $4, 0 ); }
        |       ATTRIBUTE type_spec simple_declarators
                { $$ = create_attribute( $3, $2, NULL, NULL, 0 ); };

get_excep_expr: GETRAISES '(' interface_names ')' { $$ = $3; };

set_excep_expr: SETRAISES '(' interface_names ')' { $$ = $3; };

/* Rule 99 */
value_dcl:      value_def
        |       value_forward_dcl;

/* Rule 100 */
value_def:      value_header BRACE_BEGIN value_elements BRACE_END { create_valuetype_finish( $3, $4 ); };

/* Rule 101, 102 and 103 */
value_header:   VALUETYPE IDENTIFIER { $$ = create_valuetype_start( $2, NULL, NULL ); }
        |       VALUETYPE IDENTIFIER ':' value_name
                { $$ = create_valuetype_start( $2, lookup_type( $4 ), NULL ); }
        |       VALUETYPE IDENTIFIER SUPPORTS interface_name
                { $$ = create_valuetype_start( $2, NULL, lookup_type( $4 ) ); }
        |       VALUETYPE IDENTIFIER ':' value_name SUPPORTS interface_name
                { $$ = create_valuetype_start( $2, lookup_type( $4 ), lookup_type( $6 ) ); }

/* Rule 104 */
value_name:     scoped_name;

/* Rule 105 */
value_element:  export
        |       state_member
        |       init_dcl;

value_elements: /* empty */ { $$ = NULL; }
        |       value_elements value_element { $$ = append_node( $1, $2 ); };

/* Rule 106 */
state_member:   annotations PUBLIC annotations type_spec declarators ';'
                { $$ = annotate_list( create_valuetype_member( $5, $4, 1 ), append_node( $1, $3) ); }
        |       annotations PRIVATE annotations type_spec declarators ';'
                { $$ = annotate_list( create_valuetype_member( $5, $4, 0 ), append_node( $1, $3) ); };

/* Rule 107 */
init_dcl:       FACTORY IDENTIFIER '(' init_param_dcls ')' raises_expr_or_empty ';'
                { $$ = create_valuetype_factory( $2, $4, $6 ); }
        |       FACTORY IDENTIFIER '(' ')' raises_expr_or_empty ';'
                { $$ = create_valuetype_factory( $2, NULL, $5 ); };

/* Rule 108 */
init_param_dcls:init_param_dcl { $$ = $1; }
        |       init_param_dcls ',' init_param_dcl { $$ = append_node( $1, $3 ); };

/* Rule 109 */
init_param_dcl: IN type_spec simple_declarator { $$ = create_valuetype_factory_param( $3, $2 ); };

/* Rule 110 */
value_forward_dcl:
                VALUETYPE IDENTIFIER { $$ = create_valuetype_dcl( $2 ); };

/* Rule 199 */
map_type:       MAP '<' annotations type_spec ',' annotations type_spec ',' positive_int_const '>' { --idl_subtype_count; $$ = create_map( annotate_alias( $4, $3 ), annotate_alias( $7, $6 ), $9 ); }
        |       MAP '<' annotations type_spec ',' annotations type_spec '>' { --idl_subtype_count; $$ = create_map( annotate_alias( $4, $3 ), annotate_alias( $7, $6 ), &num_undef ); };

/* Rule 200 */
bitset_dcl:     BITSET IDENTIFIER BRACE_BEGIN bitfields BRACE_END { $$ = create_bitset( $2, $4, NULL, $5 ); }
        |       BITSET IDENTIFIER ':' scoped_name BRACE_BEGIN bitfields BRACE_END { $$ = create_bitset( $2, $6, lookup_node($4), $7 ); };

/* Rule 201 */
bitfields:      /* empty */ { $$ = NULL; }
        |       bitfields bitfield { $$ = append_node( $1, $2 ); };

/* Rule 201 */
identifiers:    annotations IDENTIFIER { $$ = create_decl( $2, $1 ); };
        |       identifiers annotations IDENTIFIER { $$ = append_decl( $1, create_decl( $3, $2 ) );};

/* Rule 201 and 202 */
bitfield:       annotations BITFIELD '<' positive_int_const '>' identifiers ';'
                { $$ = annotate_list( create_bitfield( $6, $4, NULL ), $1 ); }
        |       annotations BITFIELD '<' positive_int_const ',' destination_type '>' identifiers ';'
                { $$ = annotate_list( create_bitfield( $8, $4, $6 ), $1 ); };

/* Rule 203 */
destination_type:
                boolean_type
        |       octet_type
        |       integer_type;

/* Rule 204 */
bitmask_dcl:    BITMASK IDENTIFIER BRACE_BEGIN bit_values BRACE_END { $$ = create_bitmask( $2, $4, $5 ); };

/* Rule 205 */
bit_values:     bit_value { $$ = append_enum_node( NULL, $1 ); }
        |       bit_values ',' bit_value { $$ = append_enum_node( $1, $3 ); };


/* Rule 205 */
bit_value:     doxy_comments annotations IDENTIFIER doxy_comments {
        $$ = append_node( $1, append_node( annotate( create_bitmask_value( $3, &num_undef ), $2 ), $4 ) ); }
        |       doxy_comments annotations IDENTIFIER '=' const_expr doxy_comments {
        $$ = append_node( $1, append_node( annotate( create_bitmask_value( $3, $5 ), $2 ), $6 ) ); pedantic_bitmask($$); };

/* Rule 209 and 210 */
annotation_dcl: ANNOTATION IDENTIFIER { create_annotation_dcl_start( $2 ); }
                BRACE_BEGIN annotation_inner BRACE_END { $$ = create_annotation_dcl_finish( $5, $6 ); };

annotation_inner: doxy_comments { $$ = $1; }
                | annotation_inner annotation_body doxy_comments { $$ = append_node( $1, append_node( $2, $3 ) ); };

/* Rule 211 */
annotation_body: annotation_member { $$ = $1; }
        |        bitmask_dcl ';' { $$ = $1; }
        |        enum_dcl ';' { $$ = $1; }
        |        const_dcl ';' { $$ = $1; }
        |        typedef_dcl ';' { $$ = $1; };

/* Rule 212 */
annotation_member:
                annotation_member_type simple_declarator ';'
                { $$ = create_annotation_member( $2, $1, &num_undef ); }
        |       annotation_member_type simple_declarator DEFAULT const_expr ';'
                { $$ = create_annotation_member( $2, $1, $4 ); };

/* Rule 213, missing "any" and bare scoped name */
annotation_member_type:
                const_type
        |       any_const_type;

/* Rule 214: any_const_type: "any" */
any_const_type: IDL_ANY { $$ = &any_type; };

annotations: /* empty */ { $$ = NULL; }
        |       annotations annotation_appl { $$ = append_node( $1, $2 ); }

/* Rule 215 */
annotation_appl:ANNOTATION_IDENT { create_annotation_start( $1 ); $$ = create_annotation_finish( NULL ); }
        |       ANNOTATION_IDENT { create_annotation_start( $1 ); }
                '(' annotation_appl_params ')' { $$ = create_annotation_finish( $4 ); };

/* Rule 215 */
annotation_appl_comment:
                annotation_appl_comment_ident { create_annotation_start( $1 ); $$ = create_annotation_finish( NULL ); }
        |       annotation_appl_comment_ident { create_annotation_start( $1 ); }
                '(' annotation_appl_params ')' { $$ = create_annotation_finish( $4 ); };

annotation_appl_comment_ident:
                ISOLATED_ANNOTATION_IDENT_POST {
                    idlwarning("\"//@\" (annotation to be applied to preceding identifier)"
                    " should not be used at the beginning of a line. Did you mean \"// @\"?");
                }
        |       ANNOTATION_IDENT_POST;

doxy_comment:
                DOXY_COMMENT { $$ = create_doc( $1, 0 ); };
        |       DOXY_COMMENT_POST { $$ = create_doc( $1, 1 ); };

doxy_comments: /* empty */ { $$ = NULL; }
        |       doxy_comments doxy_comment { $$ = append_node( $1, $2 ); }        

/* Rule 216 */
annotation_appl_params:
                complex_const_expr { $$ = create_annotation_param( create_identifier(NULL), $1 ); }
        |       annotation_appl_param_list;

/* Rule 216 */
annotation_appl_param_list:
                annotation_appl_param
        |       annotation_appl_param_list ',' annotation_appl_param { $$ = append_node( $1, $3 ); };

/* Rule 217 */
annotation_appl_param:
                IDENTIFIER '=' complex_const_expr { $$ = create_annotation_param( $1, $3 ); };

name_or_anon:   { $$ = create_anon_name(); }
        |       IDENTIFIER;
                
                
%%
#               include <stdio.h>

extern char idltext[];
extern struct position current_pos;

static void pedantic_bitmask(const struct ptree* node) {
    parse_pedantic(node,
        "Use of assignment operator on bitmasks is an InterCOM extension. "
        "Use the @position annotation instead");
}

static void pedantic_enum(const struct ptree* node) {
    parse_pedantic(node,
        "Use of assignment operator on enum literals is an InterCOM extension. "
        "Use the @value annotation instead");
}

int idlerror( const char * s )
{
   parse_error( s, current_input_file, current_pos.line );
   return 0;
}

int idlwarning( const char * s )
{
   parse_warning( s, current_input_file, current_pos.line );
   return 0;
}

struct identifier create_identifier( const char *name )
{
   struct identifier ident;
   ident.name = get_symbol(name);
   ident.pos = current_pos;
   return ident;
}

