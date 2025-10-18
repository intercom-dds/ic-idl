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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#pragma once

#include <optional>

#include "InterCOM/dds_xtypes_constants.h"
#include "InterCOM/member_info.h"
#include "InterCOM/serialization.h"
#include "interp.h"

namespace intercom::icgen {

class ValueMarshal {
  public:
    enum { IS_READER = 0, IS_WRITER = 1 };

    ValueMarshal() = default;

    class StructValue {
      public:
        StructValue(ValueMarshal& builder, const TypeInfo*) : m_marshal(builder) {}

        template <typename T>
        void io(const MemberInfo& info, const T& value) {
            ValueMarshal marshal;
            marshal.io(value);
            m_members.assign(info.name, marshal.value());
        }

        template <typename T>
        void io(const MemberInfo& info, const std::optional<T>& value) {
            if (value) {
                io(info, *value);
            }
        }

        ~StructValue() {
            m_marshal.m_value = std::move(m_members);
        }

      private:
        ValueMarshal& m_marshal;
        Scope m_members;
    };

    template <typename T>
    class VectorValue {
      public:
        VectorValue(ValueMarshal& builder, const TypeInfo*) : m_marshal(builder) {}

        template <typename VECTOR>
        void io(const VECTOR& value) {
            if (!value.empty()) {
                io(&value[0], static_cast<uint32_t>(value.size()));
            }
        }

        void io(const T* value, uint32_t value_count) {
            std::vector<Value> list;
            Serializer<ValueMarshal, T> serialize;

            for (uint32_t i = 0; i < value_count; i++) {
                serialize(m_marshal, const_cast<T&>(value[i]), nullptr);
                list.emplace_back(std::move(m_marshal.m_value));
            }
            m_marshal.m_value = std::move(list);
        }

      private:
        ValueMarshal& m_marshal;
    };

    template <typename K, typename V>
    class MapValue {
      public:
        MapValue(ValueMarshal& builder, const TypeInfo*) : m_marshal(builder) {
            static_assert(
                std::is_integral<K>::value || std::is_same<K, std::string>::value,
                "Only strings and integers as supported as keys in maps"
            );
        }

        template <typename T>
        void io(const T& value) {
            Scope members;
            for (const auto& elem : value) {
                ValueMarshal marshal;
                marshal.io(elem.second);
                members.assign(elem.first, marshal.value());
            }
            m_marshal.m_value = std::move(members);
        }

      private:
        ValueMarshal& m_marshal;
    };

    template <CharacterEncoding E>
    class StringValue {
      public:
        explicit StringValue(ValueMarshal& builder, const TypeInfo*) : m_marshal(builder) {}

        void io(const std::string& value) {
            m_marshal.m_value = value;
        }

      private:
        ValueMarshal& m_marshal;
    };

    template <typename T>
    void io(const T& value) {
        Serializer<ValueMarshal, T> serialize;
        serialize(*this, const_cast<T&>(value), nullptr);
    }

    template <typename T>
    void primitive_io(const T& value, const TypeInfo* info) {
        if (info &&
            (info->kind == dcps::xtypes::TK_ENUM || info->kind == dcps::xtypes::TK_BITMASK)) {
            std::string res;
            if (enumToString(res, value, info)) {
                io(res);
            }
        } else {
            io(std::to_string(value));
        }
    }

    template <typename T>
    void primitive_io(const T* value, uint32_t count, const TypeInfo*) {
        VectorValue<T> ar(*this, nullptr);
        ar.io(value, count);
    }

    Value& value() {
        return m_value;
    }

    const Value& value() const {
        return m_value;
    }

  private:
    Value m_value;
};

}  // namespace intercom::icgen
