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
// obtain one at https://www.ncia.nato.int/downloads/NCoDe_Licence_V1.0.pdf

#include "cidl/ast.h"

#ifdef _WIN32
#  pragma warning(push)
#  pragma warning(disable : 4065)
#endif

std::size_t std::hash<ast::Span>::operator()(const ast::Span& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<uint32_t>()(s.start);
    h ^= std::hash<uint32_t>()(s.end);
    return h;
}

std::size_t std::hash<ast::Ident>::operator()(const ast::Ident& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<std::string>()(s.name);
    h ^= std::hash<ast::Span>()(s.span);
    return h;
}

std::size_t std::hash<ast::Path>::operator()(const ast::Path& s) const noexcept {
    result_type h = 0;
    if (s.leading_colons.has_value()) {
        h ^= std::hash<ast::Span>()(*s.leading_colons);
    }
    for (auto& value_0 : s.segments) {
        h ^= std::hash<ast::Ident>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::LitBool>::operator()(const ast::LitBool& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<bool>()(s.uppercase);
    h ^= std::hash<bool>()(s.value);
    return h;
}

std::size_t std::hash<ast::LiteralValue>::operator()(const ast::LiteralValue& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::LIT_BOOL:
        h ^= std::hash<ast::LitBool>()(s.bool_());
        break;
    case ast::LIT_INT:
        h ^= std::hash<uint64_t>()(s.int_());
        break;
    case ast::LIT_CHAR:
        h ^= std::hash<char>()(s.char_());
        break;
    case ast::LIT_STRING:
        h ^= std::hash<std::string>()(s.string());
        break;
    default:
        break;
    }
    return h;
}

std::size_t std::hash<ast::Literal>::operator()(const ast::Literal& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Span>()(s.span);
    h ^= std::hash<ast::LiteralValue>()(s.value);
    return h;
}

std::size_t std::hash<ast::Op>::operator()(const ast::Op& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Span>()(s.span);
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s.kind));
    return h;
}

std::size_t std::hash<ast::InitList>::operator()(const ast::InitList& s) const noexcept {
    result_type h = 0;
    for (auto& value_0 : s.values) {
        h ^= std::hash<ast::NamedExpr>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::Expr>::operator()(const ast::Expr& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::EXPR_LITERAL:
        h ^= std::hash<ast::Literal>()(s.literal());
        break;
    case ast::EXPR_PATH:
        h ^= std::hash<ast::Path>()(s.path());
        break;
    case ast::EXPR_UNARY:
        if (s.unary() != nullptr) {
            h ^= std::hash<ast::Unary>()(*s.unary());
        }
        break;
    case ast::EXPR_BINARY:
        if (s.binary() != nullptr) {
            h ^= std::hash<ast::Binary>()(*s.binary());
        }
        break;
    case ast::EXPR_INIT_LIST:
        h ^= std::hash<ast::InitList>()(s.init_list());
        break;
    }
    return h;
}

std::size_t std::hash<ast::NamedExpr>::operator()(const ast::NamedExpr& s) const noexcept {
    result_type h = 0;
    if (s.ident.has_value()) {
        h ^= std::hash<ast::Ident>()(*s.ident);
    }
    h ^= std::hash<ast::Expr>()(s.value);
    return h;
}

std::size_t std::hash<ast::Unary>::operator()(const ast::Unary& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Op>()(s.op);
    h ^= std::hash<ast::Expr>()(s.expr);
    return h;
}

std::size_t std::hash<ast::Binary>::operator()(const ast::Binary& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Expr>()(s.lhs);
    h ^= std::hash<ast::Op>()(s.op);
    h ^= std::hash<ast::Expr>()(s.rhs);
    return h;
}

std::size_t std::hash<ast::AnyType>::operator()(const ast::AnyType& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Span>()(s.span);
    return h;
}

std::size_t std::hash<ast::SequenceType>::operator()(const ast::SequenceType& s) const noexcept {
    result_type h = 0;
    if (s.ty != nullptr) {
        h ^= std::hash<ast::Type>()(*s.ty);
    }
    if (s.bound.has_value()) {
        h ^= std::hash<ast::Expr>()(*s.bound);
    }
    h ^= std::hash<ast::Span>()(s.span);
    return h;
}

std::size_t std::hash<ast::StringType>::operator()(const ast::StringType& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<bool>()(s.wide);
    if (s.bound.has_value()) {
        h ^= std::hash<ast::Expr>()(*s.bound);
    }
    h ^= std::hash<ast::Span>()(s.span);
    return h;
}

std::size_t std::hash<ast::MapType>::operator()(const ast::MapType& s) const noexcept {
    result_type h = 0;
    if (s.key != nullptr) {
        h ^= std::hash<ast::Type>()(*s.key);
    }
    if (s.value != nullptr) {
        h ^= std::hash<ast::Type>()(*s.value);
    }
    if (s.bound.has_value()) {
        h ^= std::hash<ast::Expr>()(*s.bound);
    }
    h ^= std::hash<ast::Span>()(s.span);
    return h;
}

std::size_t std::hash<ast::Fixed>::operator()(const ast::Fixed& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Expr>()(s.total);
    h ^= std::hash<ast::Expr>()(s.fractional);
    return h;
}

std::size_t std::hash<ast::FixedType>::operator()(const ast::FixedType& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Span>()(s.span);
    if (s.bounds.has_value()) {
        h ^= std::hash<ast::Fixed>()(*s.bounds);
    }
    return h;
}

std::size_t std::hash<ast::Type>::operator()(const ast::Type& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::TYPE_ANY:
        h ^= std::hash<ast::AnyType>()(s.any());
        break;
    case ast::TYPE_SEQUENCE:
        h ^= std::hash<ast::SequenceType>()(s.sequence());
        break;
    case ast::TYPE_STRING:
        h ^= std::hash<ast::StringType>()(s.string());
        break;
    case ast::TYPE_MAP:
        h ^= std::hash<ast::MapType>()(s.map());
        break;
    case ast::TYPE_FIXED:
        h ^= std::hash<ast::FixedType>()(s.fixed());
        break;
    case ast::TYPE_PATH:
        h ^= std::hash<ast::Path>()(s.path());
        break;
    }
    return h;
}

std::size_t std::hash<ast::ArrayDeclarator>::operator()(const ast::ArrayDeclarator& s
) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    for (auto& value_0 : s.bounds) {
        h ^= std::hash<ast::Expr>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::Declarator>::operator()(const ast::Declarator& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::DECLARATOR_SIMPLE:
        h ^= std::hash<ast::Ident>()(s.simple());
        break;
    case ast::DECLARATOR_ARRAY:
        h ^= std::hash<ast::ArrayDeclarator>()(s.array());
        break;
    }
    return h;
}

std::size_t std::hash<ast::AnnotationArg>::operator()(const ast::AnnotationArg& s) const noexcept {
    result_type h = 0;
    if (s.ident.has_value()) {
        h ^= std::hash<ast::Ident>()(*s.ident);
    }
    h ^= std::hash<ast::Span>()(s.span);
    h ^= std::hash<ast::Expr>()(s.value);
    return h;
}

std::size_t std::hash<ast::AnnotationAppl>::operator()(const ast::AnnotationAppl& s
) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    h ^= std::hash<ast::Span>()(s.span);
    for (auto& value_0 : s.args) {
        h ^= std::hash<ast::AnnotationArg>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::Stmt>::operator()(const ast::Stmt& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    h ^= std::hash<ast::Span>()(s.span);
    for (auto& value_0 : s.annotations) {
        h ^= std::hash<ast::AnnotationAppl>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::AnnotationField>::operator()(const ast::AnnotationField& s
) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::FIELD_DEFINITION:
        if (s.item() != nullptr) {
            h ^= std::hash<ast::Item>()(*s.item());
        }
        break;
    case ast::FIELD_MEMBER:
        if (s.member() != nullptr) {
            h ^= std::hash<ast::Field>()(*s.member());
        }
        break;
    }
    return h;
}

std::size_t std::hash<ast::AnnotationDef>::operator()(const ast::AnnotationDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.params) {
        h ^= std::hash<ast::AnnotationField>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::ModuleDef>::operator()(const ast::ModuleDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.definitions) {
        h ^= std::hash<ast::Item>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::Field>::operator()(const ast::Field& s) const noexcept {
    result_type h = 0;
    for (auto& value_0 : s.names) {
        h ^= std::hash<ast::Declarator>()(value_0);
    }
    h ^= std::hash<ast::Type>()(s.ty);
    return h;
}

std::size_t std::hash<ast::StructDef>::operator()(const ast::StructDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.members) {
        h ^= std::hash<ast::Field>()(value_0);
    }
    if (s.parent.has_value()) {
        h ^= std::hash<ast::Path>()(*s.parent);
    }
    return h;
}

std::size_t std::hash<ast::Discriminator>::operator()(const ast::Discriminator& s) const noexcept {
    result_type h = 0;
    for (auto& value_0 : s.annotations) {
        h ^= std::hash<ast::AnnotationAppl>()(value_0);
    }
    h ^= std::hash<ast::Type>()(s.ty);
    return h;
}

std::size_t std::hash<ast::Empty>::operator()(const ast::Empty&) const noexcept {
    result_type h = 0;
    return h;
}

std::size_t std::hash<ast::Label>::operator()(const ast::Label& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::LABEL_CASE:
        h ^= std::hash<ast::Expr>()(s.case_());
        break;
    case ast::LABEL_DEFAULT:
        h ^= std::hash<ast::Empty>()(s.default_());
        break;
    }
    return h;
}

std::size_t std::hash<ast::UnionMember>::operator()(const ast::UnionMember& s) const noexcept {
    result_type h = 0;
    if (s.ty != nullptr) {
        h ^= std::hash<ast::Type>()(*s.ty);
    }
    h ^= std::hash<ast::Declarator>()(s.decl);
    return h;
}

std::size_t std::hash<ast::UnionNull>::operator()(const ast::UnionNull& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Span>()(s.span);
    return h;
}

std::size_t std::hash<ast::UnionElement>::operator()(const ast::UnionElement& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::ELEMENT_MEMBER:
        h ^= std::hash<ast::UnionMember>()(s.member());
        break;
    case ast::ELEMENT_NULL:
        h ^= std::hash<ast::UnionNull>()(s.null());
        break;
    }
    return h;
}

std::size_t std::hash<ast::UnionField>::operator()(const ast::UnionField& s) const noexcept {
    result_type h = 0;
    for (auto& value_0 : s.annotations) {
        h ^= std::hash<ast::AnnotationAppl>()(value_0);
    }
    for (auto& value_0 : s.labels) {
        h ^= std::hash<ast::Label>()(value_0);
    }
    h ^= std::hash<ast::UnionElement>()(s.field);
    return h;
}

std::size_t std::hash<ast::UnionDef>::operator()(const ast::UnionDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    h ^= std::hash<ast::Discriminator>()(s.disc);
    for (auto& value_0 : s.fields) {
        h ^= std::hash<ast::UnionField>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::ConstDef>::operator()(const ast::ConstDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    h ^= std::hash<ast::Declarator>()(s.decl);
    h ^= std::hash<ast::Type>()(s.ty);
    h ^= std::hash<ast::Expr>()(s.value);
    return h;
}

std::size_t std::hash<ast::Enumerator>::operator()(const ast::Enumerator& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    for (auto& value_0 : s.annotations) {
        h ^= std::hash<ast::AnnotationAppl>()(value_0);
    }
    if (s.value.has_value()) {
        h ^= std::hash<ast::Expr>()(*s.value);
    }
    return h;
}

std::size_t std::hash<ast::EnumDef>::operator()(const ast::EnumDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.fields) {
        h ^= std::hash<ast::Enumerator>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::ExceptDef>::operator()(const ast::ExceptDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.members) {
        h ^= std::hash<ast::Field>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::AliasDef>::operator()(const ast::AliasDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.decl) {
        h ^= std::hash<ast::Declarator>()(value_0);
    }
    h ^= std::hash<ast::Type>()(s.ty);
    return h;
}

std::size_t std::hash<ast::Bit>::operator()(const ast::Bit& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    if (s.value.has_value()) {
        h ^= std::hash<ast::Expr>()(*s.value);
    }
    return h;
}

std::size_t std::hash<ast::BitmaskDef>::operator()(const ast::BitmaskDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.bits) {
        h ^= std::hash<ast::Bit>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::Bitfield>::operator()(const ast::Bitfield& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    h ^= std::hash<ast::Expr>()(s.size);
    return h;
}

std::size_t std::hash<ast::BitsetDef>::operator()(const ast::BitsetDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    if (s.parent.has_value()) {
        h ^= std::hash<ast::Path>()(*s.parent);
    }
    for (auto& value_0 : s.fields) {
        h ^= std::hash<ast::Bitfield>()(value_0);
    }
    return h;
}

std::size_t std::hash<ast::Attribute>::operator()(const ast::Attribute& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    h ^= std::hash<ast::Type>()(s.ty);
    if (s.readonly.has_value()) {
        h ^= std::hash<ast::Span>()(*s.readonly);
    }
    return h;
}

std::size_t std::hash<ast::Param>::operator()(const ast::Param& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    h ^= std::hash<ast::Type>()(s.ty);
    if (s.kind.has_value()) {
        h ^= std::hash<int32_t>()(static_cast<int32_t>(*s.kind));
    }
    return h;
}

std::size_t std::hash<ast::Prototype>::operator()(const ast::Prototype& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    h ^= std::hash<ast::Type>()(s.ret);
    for (auto& value_0 : s.params) {
        h ^= std::hash<ast::Param>()(value_0);
    }
    for (auto& value_0 : s.raises) {
        h ^= std::hash<ast::Path>()(value_0);
    }
    if (s.oneway.has_value()) {
        h ^= std::hash<ast::Span>()(*s.oneway);
    }
    return h;
}

std::size_t std::hash<ast::InterfaceDef>::operator()(const ast::InterfaceDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.members) {
        h ^= std::hash<ast::InterfaceMember>()(value_0);
    }
    for (auto& value_0 : s.inherits) {
        h ^= std::hash<ast::Path>()(value_0);
    }
    if (s.local.has_value()) {
        h ^= std::hash<ast::Span>()(*s.local);
    }
    return h;
}

std::size_t std::hash<ast::ValueMember>::operator()(const ast::ValueMember& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Ident>()(s.ident);
    h ^= std::hash<ast::Type>()(s.ty);
    if (s.public_.has_value()) {
        h ^= std::hash<ast::Span>()(*s.public_);
    }
    return h;
}

std::size_t std::hash<ast::ValuetypeDef>::operator()(const ast::ValuetypeDef& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    for (auto& value_0 : s.members) {
        h ^= std::hash<ast::ValueMember>()(value_0);
    }
    for (auto& value_0 : s.prototypes) {
        h ^= std::hash<ast::Prototype>()(value_0);
    }
    if (s.inherits.has_value()) {
        h ^= std::hash<ast::Path>()(*s.inherits);
    }
    if (s.supports.has_value()) {
        h ^= std::hash<ast::Path>()(*s.supports);
    }
    return h;
}

std::size_t std::hash<ast::Decl>::operator()(const ast::Decl& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<ast::Stmt>()(s);
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s.kind));
    return h;
}

std::size_t std::hash<ast::Item>::operator()(const ast::Item& s) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::ITEM_ANNOTATION:
        h ^= std::hash<ast::AnnotationDef>()(s.annotation_value());
        break;
    case ast::ITEM_MODULE:
        h ^= std::hash<ast::ModuleDef>()(s.module_value());
        break;
    case ast::ITEM_STRUCT:
        h ^= std::hash<ast::StructDef>()(s.struct_value());
        break;
    case ast::ITEM_UNION:
        h ^= std::hash<ast::UnionDef>()(s.union_value());
        break;
    case ast::ITEM_ENUM:
        h ^= std::hash<ast::EnumDef>()(s.enum_value());
        break;
    case ast::ITEM_EXCEPTION:
        h ^= std::hash<ast::ExceptDef>()(s.exception_value());
        break;
    case ast::ITEM_BITMASK:
        h ^= std::hash<ast::BitmaskDef>()(s.bitmask_value());
        break;
    case ast::ITEM_BITSET:
        h ^= std::hash<ast::BitsetDef>()(s.bitset_value());
        break;
    case ast::ITEM_CONST:
        h ^= std::hash<ast::ConstDef>()(s.const_value());
        break;
    case ast::ITEM_TYPEDEF:
        h ^= std::hash<ast::AliasDef>()(s.alias_value());
        break;
    case ast::ITEM_INTERFACE:
        h ^= std::hash<ast::InterfaceDef>()(s.interface_value());
        break;
    case ast::ITEM_VALUETYPE:
        h ^= std::hash<ast::ValuetypeDef>()(s.valuetype_value());
        break;
    case ast::ITEM_DECL:
        h ^= std::hash<ast::Decl>()(s.decl_value());
        break;
    }
    return h;
}

std::size_t std::hash<ast::InterfaceMember>::operator()(const ast::InterfaceMember& s
) const noexcept {
    result_type h = 0;
    h ^= std::hash<int32_t>()(static_cast<int32_t>(s._d()));
    switch (s._d()) {
    case ast::INTERFACE_ATTRIBUTE:
        h ^= std::hash<ast::Attribute>()(s.attr());
        break;
    case ast::INTERFACE_PROTOTYPE:
        h ^= std::hash<ast::Prototype>()(s.proto());
        break;
    case ast::INTERFACE_ITEM:
        h ^= std::hash<ast::Item>()(s.item());
        break;
    }
    return h;
}

#ifdef _WIN32
#  pragma warning(pop)
#endif
