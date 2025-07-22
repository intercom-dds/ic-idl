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

struct Point {
    Point() = default;
    Point(const Point&) = default;
    Point& operator=(const Point&) = default;
    Point(Point &&) = default;
    Point& operator=(Point &&) = default;
    Point(
        int32_t a_x,
        int32_t a_y,
        int32_t a_z);
    bool operator<(const Point & a_other) const;
    bool operator==(const Point & a_other) const;
    bool operator!=(const Point & a_other) const { return !(*this == a_other); }
    bool operator>(const Point & a_other) const { return a_other < *this; }
    bool operator<=(const Point & a_other) const { return !(a_other < *this); }
    bool operator>=(const Point & a_other) const { return !(*this < a_other); }

    int32_t x { 0 };
    int32_t y { 0 };
    int32_t z { 0 };
};

using PointSeq = ::std::vector<Point>;
extern const Point MY_POINT;
extern const std::vector<int32_t> foo;
struct MyStruct {
    MyStruct() = default;
    MyStruct(const MyStruct&) = default;
    MyStruct& operator=(const MyStruct&) = default;
    MyStruct(MyStruct &&) = default;
    MyStruct& operator=(MyStruct &&) = default;
    explicit MyStruct(
        ::std::array<int32_t, 3> a_value);
    bool operator<(const MyStruct & a_other) const;
    bool operator==(const MyStruct & a_other) const;
    bool operator!=(const MyStruct & a_other) const { return !(*this == a_other); }
    bool operator>(const MyStruct & a_other) const { return a_other < *this; }
    bool operator<=(const MyStruct & a_other) const { return !(a_other < *this); }
    bool operator>=(const MyStruct & a_other) const { return !(*this < a_other); }

    ::std::array<int32_t, 3> value{{ 0,  0,  0}};
};

using MyStructSeq = ::std::vector<MyStruct>;
extern const std::array<int32_t, 3> MY_CONST;
template<>
struct std::hash<Point> {
    using argument_type = Point;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template<>
struct std::hash<MyStruct> {
    using argument_type = MyStruct;
    using result_type = std::size_t;
    result_type operator()(const argument_type&) const noexcept;
};
template <>
struct ic_cts::TypeTraits<Point> { //< \private
    using value_type = Point;
    using in_type = const Point&;
    using out_type = Point&;
    using inout_type = Point&;
    using ref_type = std::shared_ptr<Point>;
    using weak_ref_type = std::weak_ptr<Point>;
    using sequence_type = PointSeq;
    static const TypeInfo type_info;
    static const char* default_topic_name;
    static const char* intercom_type_identifier;
    static const bool has_member_accessor_functions = false;
    static const bool is_struct = true;
};
template <class Archive>
struct ic_cts::Serializer<Archive, Point> {
    void operator()(Archive& a_archive, Point& a_value, const TypeInfo*) {
        const TypeInfo* a_info = &ic_cts::TypeTraits<Point>::type_info;
        typename Archive::StructValue serializer(a_archive, a_info);
        serializer.io(a_info->members[0], a_value.x);
        serializer.io(a_info->members[1], a_value.y);
        serializer.io(a_info->members[2], a_value.z);
    }
};
template <>
struct ic_cts::TypeTraits<MyStruct> { //< \private
    using value_type = MyStruct;
    using in_type = const MyStruct&;
    using out_type = MyStruct&;
    using inout_type = MyStruct&;
    using ref_type = std::shared_ptr<MyStruct>;
    using weak_ref_type = std::weak_ptr<MyStruct>;
    using sequence_type = MyStructSeq;
    static const TypeInfo type_info;
    static const char* default_topic_name;
    static const char* intercom_type_identifier;
    static const bool has_member_accessor_functions = false;
    static const bool is_struct = true;
};
template <class Archive>
struct ic_cts::Serializer<Archive, MyStruct> {
    void operator()(Archive& a_archive, MyStruct& a_value, const TypeInfo*) {
        const TypeInfo* a_info = &ic_cts::TypeTraits<MyStruct>::type_info;
        typename Archive::StructValue serializer(a_archive, a_info);
        serializer.io(a_info->members[0], a_value.value);
    }
};
inline Point::Point (
    int32_t a_x,
    int32_t a_y,
    int32_t a_z) :
x(a_x),
y(a_y),
z(a_z) {}

inline bool Point::operator<(const Point & a_other) const {
    if (this->x < a_other.x) { return true; }
    if (a_other.x < this->x) { return false; }
    if (this->y < a_other.y) { return true; }
    if (a_other.y < this->y) { return false; }
    return this->z < a_other.z;
}

inline bool Point::operator==(const Point & a_other) const {
    if (!(this->x == a_other.x)) { return false; }
    if (!(this->y == a_other.y)) { return false; }
    if (!(this->z == a_other.z)) { return false; }
    return true;
}

inline MyStruct::MyStruct (
    ::std::array<int32_t, 3> a_value) :
value(std::move(a_value)) {}

inline bool MyStruct::operator<(const MyStruct & a_other) const {
    return this->value < a_other.value;
}

inline bool MyStruct::operator==(const MyStruct & a_other) const {
    if (!(this->value == a_other.value)) { return false; }
    return true;
}

inline std::ostream& operator<<(std::ostream& stream, const Point& value) {
    return ic_cts::marshal_json(stream, value);
}

inline std::istream& operator>>(std::istream& stream, Point& value) {
    return ic_cts::unmarshal_json(stream, value);
}

inline std::ostream& operator<<(std::ostream& stream, const MyStruct& value) {
    return ic_cts::marshal_json(stream, value);
}

inline std::istream& operator>>(std::istream& stream, MyStruct& value) {
    return ic_cts::unmarshal_json(stream, value);
}

#ifdef _WIN32
#pragma warning(pop)
#endif

