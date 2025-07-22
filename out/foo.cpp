#include "foo.h"

#include <ic_cts/dds_xtypes_constants.h>
#ifdef _WIN32
#pragma warning(push)
#pragma warning(disable:4065)
#endif

static ic_cts::TypeInfo Matrix_type_info_0_dim_element = { "int32[3]", ic_cts::dcps::xtypes::TK_ARRAY, 0, 0, 0, 3, nullptr, nullptr, nullptr, nullptr, &::ic_cts::LONG_TYPE_INFO, 0, nullptr };
static ic_cts::TypeInfo Matrix_type_info_0 = { "Matrix::int32[3][2]", ic_cts::dcps::xtypes::TK_ARRAY, 0, 0, 0, 2, nullptr, nullptr, nullptr, nullptr, &Matrix_type_info_0_dim_element, 0, nullptr };
static ic_cts::MemberInfo Matrix_members[1] = {
    { 0, "data", uint32_t(ic_cts::dcps::xtypes::IS_IMPLICIT_KEY), ic_cts::MEMBER_INFO_EMPTY_CASE_LABELS, &Matrix_type_info_0, nullptr } };

const ic_cts::TypeInfo ic_cts::TypeTraits<Matrix>::type_info = {
    "Matrix", ic_cts::dcps::xtypes::TK_STRUCTURE, uint32_t(ic_cts::dcps::xtypes::IS_APPENDABLE), 0, 0, 0, nullptr, nullptr, nullptr, nullptr, nullptr, 1, Matrix_members
};

const Matrix m { std::array<std::array<int32_t, 3>, 2>{{ std::array<int32_t, 3>{{ 1,  0,  3 }},  std::array<int32_t, 3>{{ 4,  5,  6 }} }} };
std::size_t std::hash<Matrix>::operator()(const Matrix& s) const noexcept {
    result_type h = 0;
    for (auto& value_0 : s.data) {
        for (auto& value_1 : value_0) {
            h ^= std::hash< int32_t >()(value_1);
        }
    }
    return h;
}

#ifdef _WIN32
#pragma warning(pop)
#endif

