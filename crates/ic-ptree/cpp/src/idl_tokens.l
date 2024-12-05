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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <cidl/ptree_builder.h>
#include "idl_grammar.tab.h"

extern struct identifier create_identifier( const char *name );
char g_ignore_doxy_comments = 0;

struct position current_pos;
void count( const char* text );
void prune_end( char* text );
int idl_subtype_count;
int include_level;
struct position comment_start_pos;
const char* included_as;
%}

D         [0-9]
L         [a-zA-Z_]
H         [a-fA-F0-9]
E         [Ee][+-]?{D}+
FS        [fF]
U         [uU]
LO        [lL]
white     [ \t]
newline   ("\v"|"\f"|"\n"|"\r"|"\n\r"|"\r\n")
whitestar [ \t*]

%x PRAGMA
%x IGNORE
%x CPP
%x CPP_INCLUDE
%x BLOCK_COMMENT
%x DOXY_LINE
%x DOXY_LINE_POST
%x DOXY_BLOCK
%%

{white}*"/*"[*!]+{whitestar}* { comment_start_pos = current_pos; count(idltext); BEGIN(DOXY_BLOCK); }
<DOXY_BLOCK>{whitestar}*"*/"{newline}?  { count(idltext); BEGIN(INITIAL); }
<DOXY_BLOCK>(.|white)*{newline}? {
                                 // Search for "*/" and terminate block comment if found
                                 char* end = idltext;
                                 while (*end) {
                                    if (*end == '*' && *(end+1) == '/') {
                                       REJECT; // Match terminating rule below
                                    }
                                    end++;
                                 }

                                 int col = current_pos.column;
                                 count(idltext);
                                 prune_end(idltext);
                                 while (col <= comment_start_pos.column && (*idltext == ' ' || *idltext == '*' || *idltext == '\t')) {
                                    idltext++;
                                    col++;
                                 }
                                 idllval.ident = create_identifier( get_symbol(idltext) );
                                 idllval.ident.pos = comment_start_pos;
                                 if (!g_ignore_doxy_comments) {
                                     return(DOXY_COMMENT);
                                 }
                              }

<DOXY_BLOCK>(.|white)*"*/"{newline}? {
                                 int col = current_pos.column;
                                 count(idltext);
                                 prune_end(idltext);
                                 idltext[strlen(idltext)-2] = '\0'; // Remove trailing comment end

                                 while (col <= comment_start_pos.column && (*idltext == ' ' || *idltext == '*' || *idltext == '\t')) {
                                    idltext++;
                                    col++;
                                 }
                                 idllval.ident = create_identifier( get_symbol(idltext) );
                                 idllval.ident.pos = comment_start_pos;
                                 BEGIN(INITIAL);
                                 if (!g_ignore_doxy_comments) {
                                     return(DOXY_COMMENT);
                                 }
                              }

{white}*"//"[/!]+     { comment_start_pos = current_pos; count(idltext); BEGIN(DOXY_LINE); }
<DOXY_LINE>(.|white)*{newline}? {
                                 count(idltext); prune_end(idltext);
                                 idllval.ident = create_identifier( get_symbol(idltext) );
                                 idllval.ident.pos = comment_start_pos;
                                 BEGIN(INITIAL);
                                 if (!g_ignore_doxy_comments) {
                                    return(DOXY_COMMENT);
                                 } }

{white}*"//"[/!]+"<"       { comment_start_pos = current_pos; count(idltext); BEGIN(DOXY_LINE_POST); }
<DOXY_LINE_POST>(.|white)*{newline}? {
                                 count(idltext); prune_end(idltext);
                                 idllval.ident = create_identifier( get_symbol(idltext) );
                                 idllval.ident.pos = comment_start_pos;
                                 BEGIN(INITIAL);
                                 if (!g_ignore_doxy_comments) {
                                    return(DOXY_COMMENT_POST);
                                 } }

"/*"                       { count(idltext); BEGIN(BLOCK_COMMENT); }
<BLOCK_COMMENT>"*/"        { count(idltext); BEGIN(INITIAL); }
<BLOCK_COMMENT>(.|{white}) { count(idltext); }
<BLOCK_COMMENT>{newline}   { count(idltext); }

^#                            { count(idltext); BEGIN(CPP); }
"//"[ \t]*("INTERCOM"|"DCPS") { count(idltext); yyless(2); BEGIN(PRAGMA); }
"//"                          { count(idltext); BEGIN(IGNORE); }
^"--"                         { count(idltext); BEGIN(IGNORE); }


<CPP>"included_as"[ \t]+        { count(idltext); BEGIN(CPP_INCLUDE); }
<CPP>"pragma"[ \t]+("INTERCOM"|"DCPS")  { count(idltext); yyless(6); BEGIN(PRAGMA); }
<CPP>"line"[ \t]+               { count(idltext); ; }
<CPP>({D}+)                     { count(idltext); sscanf(idltext,"%d",&current_pos.line); }
<CPP>L?\"(\\.|[^\\"])*\"        {   count(idltext);
                                    ++idltext;
                                    idltext[strlen(idltext)-1] = 0;
                                    current_input_file = get_symbol(idltext);
                                    if ( included_as )
                                    {
                                       idllval.ident = create_identifier( included_as );
                                       included_as = NULL;
                                       ++include_level;
                                       return( INCLUDE_BEGIN );
                                    }
                                    else if ( include_level > 0 )
                                    {
                                       --include_level;
                                       return( INCLUDE_END );
                                    }
                                 } /* " */
<CPP>{newline}                    { /* do not count, line inserted by preprocessor */ reset_comment(); BEGIN(INITIAL); }
<CPP_INCLUDE>"<"[^>]+">"          { count(idltext);  included_as = get_symbol(idltext); }
<CPP_INCLUDE>"\""[^"]+"\""        { count(idltext);  included_as = get_symbol(idltext); }

<PRAGMA>({L}|{D}|"::"|"."|"]"|"[")+       { count(idltext); pragma_arg(get_symbol(idltext)); }
<PRAGMA>"//"                      { count(idltext); pragma_finish(); BEGIN(IGNORE); }
<PRAGMA>{newline}                 { pragma_finish(); count(idltext); BEGIN(INITIAL); }

<IGNORE,CPP,CPP_INCLUDE>{newline} { count(idltext); reset_comment(); BEGIN(INITIAL); }
<PRAGMA,IGNORE,CPP,CPP_INCLUDE>.       { count(idltext); ; }

"null"        { count(idltext); return(NULL_LITERAL); }
"module"      { count(idltext); return(MODULE); }
"switch"      { count(idltext); return(SWITCH); }
"union"       { count(idltext); return(UNION); }
"enum"        { count(idltext); return(ENUM); }
"const"       { count(idltext); return(CONST); }
"case"        { count(idltext); return(CASE); }
"default"     { count(idltext); return(DEFAULT); }
"struct"      { count(idltext); return(STRUCT); }
"native"      { count(idltext); return(NATIVE); }
"typedef"     { count(idltext); return(TYPEDEF); }
"@annotation" { count(idltext); return(ANNOTATION); }
"::"          { count(idltext); return(DOUBLECOLON); }

"local"       { count(idltext); return(LOCAL); }
"exception"   { count(idltext); return(EXCEPTION); }
"interface"   { count(idltext); return(INTERFACE); }
"in"          { count(idltext); return(IN); }
"out"         { count(idltext); return(OUT); }
"inout"       { count(idltext); return(INOUT); }
"raises"      { count(idltext); return(RAISES); }
"getraises"   { count(idltext); return(GETRAISES); }
"setraises"   { count(idltext); return(SETRAISES); }
"attribute"   { count(idltext); return(ATTRIBUTE); }
"readonly"    { count(idltext); return(READONLY); }

"valuetype"   { count(idltext); return(VALUETYPE); }
"public"      { count(idltext); return(PUBLIC); }
"private"     { count(idltext); return(PRIVATE); }
"factory"     { count(idltext); return(FACTORY); }

"unsigned"    { count(idltext); return(IDL_UNSIGNED); }
"octet"       { count(idltext); return(IDL_OCTET); }
"boolean"     { count(idltext); return(IDL_BOOLEAN); }
"char"        { count(idltext); return(IDL_CHAR); }
"wchar"       { count(idltext); return(IDL_WCHAR); }
"short"       { count(idltext); return(IDL_SHORT); }
"long"        { count(idltext); return(IDL_LONG); }
"float"       { count(idltext); return(IDL_FLOAT); }
"double"      { count(idltext); return(IDL_DOUBLE); }
"any"         { count(idltext); return(IDL_ANY); }
"void"        { count(idltext); return(IDL_VOID); }
"Object"      { count(idltext); return(IDL_OBJECT); }

"char8"       { count(idltext); return(IDL_CHAR); }
"char16"      { count(idltext); return(IDL_WCHAR); }
"float32"     { count(idltext); return(IDL_FLOAT); }
"float64"     { count(idltext); return(IDL_DOUBLE); }
"int8"        { count(idltext); return(IDL_INT8); }
"uint8"       { count(idltext); return(IDL_OCTET); }
"int16"       { count(idltext); return(IDL_SHORT); }
"uint16"      { count(idltext); return(IDL_USHORT); }
"int32"       { count(idltext); return(IDL_LONG); }
"uint32"      { count(idltext); return(IDL_ULONG); }
"int64"       { count(idltext); return(IDL_LONGLONG); }
"uint64"      { count(idltext); return(IDL_ULONGLONG); }

"sequence"    { count(idltext); ++idl_subtype_count; return(SEQUENCE); }
"string"      { count(idltext); return(STRING); }
"wstring"     { count(idltext); return(WSTRING); }
"fixed"       { count(idltext); return(FIXED); }
"map"         { count(idltext); ++idl_subtype_count; return(MAP); }
"bitset"      { count(idltext); return(BITSET); }
"bitfield"    { count(idltext); return(BITFIELD); }
"bitmask"     { count(idltext); return(BITMASK); }
"{"           { count(idltext); idllval.pos = current_pos; return(BRACE_BEGIN); }
"}"           { count(idltext); idllval.pos = current_pos; return(BRACE_END); }


("TRUE"|"true")                  { count(idltext); idllval.num = create_bool(1); return(CONSTANT); }
("FALSE"|"false")                { count(idltext); idllval.num = create_bool(0); return(CONSTANT); }

{newline}{white}*"//@"{L}({L}|{D}|"::"{L})* { count(idltext); idllval.ident = create_identifier( get_symbol(strchr(idltext, '@')) ); return(ISOLATED_ANNOTATION_IDENT_POST); }
"//@"{L}({L}|{D}|"::"{L})*                  { count(idltext); idllval.ident = create_identifier( get_symbol(idltext+2) ); return(ANNOTATION_IDENT_POST); }
@{L}({L}|{D}|"::"{L})*                      { count(idltext); idllval.ident = create_identifier( get_symbol(idltext) ); return(ANNOTATION_IDENT); }
"oneway"                                    { count(idltext); idllval.ident = create_identifier( "@oneway" ); return(ANNOTATION_IDENT); }

_{L}({L}|{D})* { count(idltext); idllval.ident = create_identifier( get_symbol(idltext + 1) ); return(IDENTIFIER); }
{L}({L}|{D})*  { count(idltext); idllval.ident = create_identifier( get_symbol(idltext) ); return(IDENTIFIER); }

0[xX]{H}+ { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 16), 16); return(CONSTANT); }
0{D}+     { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 8), 8); return(CONSTANT); }
{D}+      { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 10), 10); return(CONSTANT); }

0[xX]{H}+{U} { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 16), 16); return(CONSTANT); }
0{D}+{U}     { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 8), 8); return(CONSTANT); }
{D}+{U}      { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 10), 10); return(CONSTANT); }

0[xX]{H}+{LO}              { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 16), 16); return(CONSTANT); }
0{D}+{LO}                  { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 8), 8); return(CONSTANT); }
{D}+{LO}                   { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 10), 10); return(CONSTANT); }
0[xX]{H}+({LO}{U}|{U}{LO}) { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 16), 16); return(CONSTANT); }
0{D}+({LO}{U}|{U}{LO})     { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 8), 8); return(CONSTANT); }
{D}+({LO}{U}|{U}{LO})      { count(idltext); idllval.num = create_u64(strtoull(idltext, NULL, 10), 10); return(CONSTANT); }

{D}+{E}               { count(idltext); idllval.num = create_double(atof(idltext)); return(CONSTANT); }
{D}*"."{D}+({E})?     { count(idltext); idllval.num = create_double(atof(idltext)); return(CONSTANT); }
{D}+"."{D}*({E})?     { count(idltext); idllval.num = create_double(atof(idltext)); return(CONSTANT); }
{D}+{E}{FS}           { count(idltext); idllval.num = create_float(atof(idltext)); return(CONSTANT); }
{D}*"."{D}+({E})?{FS} { count(idltext); idllval.num = create_float(atof(idltext)); return(CONSTANT); }
{D}+"."{D}*({E})?{FS} { count(idltext); idllval.num = create_float(atof(idltext)); return(CONSTANT); }

"'\\"{D}+"'"          { int val;
                        sscanf(idltext,"\'\\%i\'", &val);
                        idllval.num = create_char(val);
                        return CHAR_LITERAL; }
L?\"(\\.|[^\\"])*\"   { count(idltext);
                        while ( *idltext != '\"' ) ++idltext;
                        ++idltext;
                        idltext[strlen(idltext)-1] = '\0';
                        idllval.num = create_str(idltext);
                        return(STRING_LITERAL); }  /* " */
L?\'(\\.|[^\\'])+\'   { count(idltext);
                        while ( *idltext != '\'' ) ++idltext;
                        idllval.num = create_char(idltext[1]);
                        return(CHAR_LITERAL); }  /* ' */

"<<"      { count(idltext); return(LEFT_SHIFT); }
">>"      { count(idltext); if ( idl_subtype_count >= 1 ) { yyless(1); return '>'; } else { return(RIGHT_SHIFT); } }
{white}   { count(idltext); }
{newline} { count(idltext); }
.         { count(idltext); return idltext[0]; }

%%

void count( const char* text )
{
    const char* c = text;
    while ( *c )
    {
       ++current_pos.column;
       if ( *c == '\n' || *c == '\r' || *c == '\f' || *c == '\v' )
       {
          ++current_pos.line;
          current_pos.column = 0;
       }
       if ( ( *c == '\n' && *(c+1) == '\r' ) ||
            ( *c == '\r' && *(c+1) == '\n' ) )
       {
          ++c;
       }
       ++c;
    }
}

void prune_end( char* text )
{
   char *end = text;
   while (*end) { end++; }
   while (end != text && (*(end-1) == ' ' || *(end-1) == '\r' || *(end-1) == '\n' || *(end-1) == '\t')) {
      end--;
   }
   *end = '\0';
}

int idlparse(void);

int idlwrap(void)
{
    return(1);
}

int scan_file( FILE* file )
{
   int result;
   idl_subtype_count = 0;
   current_pos.line = current_pos.column = 0;
   idl_create_buffer( file, YY_BUF_SIZE );
   result = idlparse();
   yy_delete_buffer( YY_CURRENT_BUFFER );
   return result;
}

int scan_string( const char *str )
{
   int result;
   idl_subtype_count = 0;
   current_pos.line = current_pos.column = 0;
   idl_scan_string( (char*)str );
   result = idlparse();
   yy_delete_buffer( YY_CURRENT_BUFFER );
   return result;
}
