#include "a.h"

#include <ic_cts/dds_xtypes_constants.h>
#ifdef _WIN32
#pragma warning(push)
#pragma warning(disable:4065)
#endif

static ic_cts::MemberInfo Point_members[3] = {
    { 0, "x", uint32_t(ic_cts::dcps::xtypes::IS_IMPLICIT_KEY), ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS, &::ic_cts::LONG_TYPE_INFO, nullptr },
    { 1, "y", uint32_t(ic_cts::dcps::xtypes::IS_IMPLICIT_KEY), ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS, &::ic_cts::LONG_TYPE_INFO, nullptr },
    { 2, "z", uint32_t(ic_cts::dcps::xtypes::IS_IMPLICIT_KEY), ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS, &::ic_cts::LONG_TYPE_INFO, nullptr } };

const ic_cts::TypeInfo ic_cts::TypeTraits<Point>::type_info = {
    "Point", ic_cts::dcps::xtypes::TK_STRUCTURE, uint32_t(ic_cts::dcps::xtypes::IS_APPENDABLE), 0, 0, 0, nullptr, nullptr, nullptr, nullptr, nullptr, 3, Point_members
};

const Point MY_POINT { 1,  2,  3 };
const std::vector<int32_t> foo { 1,  2,  3 };
static ic_cts::TypeInfo MyStruct_type_info_0 = { "int32[3]", ic_cts::dcps::xtypes::TK_ARRAY, 0, 0, 0, 3, nullptr, nullptr, nullptr, nullptr, &::ic_cts::LONG_TYPE_INFO, 0, nullptr };
static ic_cts::MemberInfo MyStruct_members[1] = {
    { 0, "value", uint32_t(ic_cts::dcps::xtypes::IS_IMPLICIT_KEY), ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS, &MyStruct_type_info_0, nullptr } };

const ic_cts::TypeInfo ic_cts::TypeTraits<MyStruct>::type_info = {
    "MyStruct", ic_cts::dcps::xtypes::TK_STRUCTURE, uint32_t(ic_cts::dcps::xtypes::IS_APPENDABLE), 0, 0, 0, nullptr, nullptr, nullptr, nullptr, nullptr, 1, MyStruct_members
};

const std::array<int32_t, 3> MY_CONST { 1,  2,  3 };
std::size_t std::hash<Point>::operator()(const Point& s) const noexcept {
    result_type h = 0;
    h ^= std::hash< int32_t >()(s.x);
    h ^= std::hash< int32_t >()(s.y);
    h ^= std::hash< int32_t >()(s.z);
    return h;
}

std::size_t std::hash<MyStruct>::operator()(const MyStruct& s) const noexcept {
    result_type h = 0;
    for (auto& value_0 : s.value) {
        h ^= std::hash< int32_t >()(value_0);
    }
    return h;
}

#ifdef _WIN32
#pragma warning(pop)
#endif

