#pragma once
#include <ic_cts/version.h>
#ifndef INTERCOM_VERSION_3_16_2
#error "CIDL-generated code does not match ic-idl product version: 3_16_2"
#endif // INTERCOM_VERSION_3_16_2

#ifdef _WIN32
#pragma warning(push)
#pragma warning(disable:4065)
#pragma warning(disable:4127)
#endif

#include <ic_cts/cts.h>
#include <ic_cts/cdr_serializer.h>
#include <ic_cts/json_serializer.h>

struct Matrix {
    Matrix() = default;
    Matrix(const Matrix&) = default;
    Matrix& operator=(const Matrix&) = default;
    Matrix(Matrix &&) = default;
    Matrix& operator=(Matrix &&) = default;
    explicit Matrix(
        ::std::array<::std::array<int32_t, 3>, 2> a_data);
    bool operator<(const Matrix & a_other) const;
    bool operator==(const Matrix & a_other) const;
    bool operator!=(const Matrix & a_other) const { return !(*this == a_other); }
    bool operator>(const Matrix & a_other) const { return a_other < *this; }
    bool operator<=(const Matrix & a_other) const { return !(a_other < *this); }
    bool operator>=(const Matrix & a_other) const { return !(*this < a_other); }

    ::std::array<::std::array<int32_t, 3>, 2> data;
};

using MatrixSeq = ::std::vector<Matrix>;
extern const Matrix m;
template<>
struct std::hash<Matrix> {
    using argument_type = Matrix;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct ic_cts::TypeTraits<Matrix> { //< \private
    using value_type = Matrix;
    using in_type = const Matrix&;
    using out_type = Matrix&;
    using inout_type = Matrix&;
    using ref_type = std::shared_ptr<Matrix>;
    using weak_ref_type = std::weak_ptr<Matrix>;
    using sequence_type = MatrixSeq;
    static const TypeInfo type_info;
    static const char* default_topic_name;
    static const char* intercom_type_identifier;
    static const bool has_member_accessor_functions = false;
    static const bool is_struct = true;
};
template <class Archive>
struct ic_cts::Serializer<Archive, Matrix> {
    void operator()(Archive& a_archive, Matrix& a_value, const TypeInfo*) {
        const TypeInfo* a_info = &ic_cts::TypeTraits<Matrix>::type_info;
        typename Archive::StructValue serializer(a_archive, a_info);
        serializer.io(a_info->members[0], a_value.data);
    }
};
inline Matrix::Matrix (
    ::std::array<::std::array<int32_t, 3>, 2> a_data) :
data(std::move(a_data)) {}

inline bool Matrix::operator<(const Matrix & a_other) const {
    return this->data < a_other.data;
}

inline bool Matrix::operator==(const Matrix & a_other) const {
    if (!(this->data == a_other.data)) { return false; }
    return true;
}

inline std::ostream& operator<<(std::ostream& stream, const Matrix& value) {
    return ic_cts::marshal_json(stream, value);
}

inline std::istream& operator>>(std::istream& stream, Matrix& value) {
    return ic_cts::unmarshal_json(stream, value);
}

#ifdef _WIN32
#pragma warning(pop)
#endif

