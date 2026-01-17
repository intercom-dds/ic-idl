// Copyright 2026 KONGSBERG
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

package integrationtests;

import static org.junit.jupiter.api.Assertions.*;

import any_types.*;
import java.util.*;
import org.junit.jupiter.api.Test;
import struct_types.Point;

class AnyTypeTests {

    @Test
    void any_default_is_null() {
        var c = new ContainsAny();
        assertNull(c.getValue());
    }

    @Test
    void any_accepts_int() {
        var c = new ContainsAny();
        c.setValue(42);
        assertEquals(42, c.getValue());
    }

    @Test
    void any_accepts_string() {
        var c = new ContainsAny();
        c.setValue("hello");
        assertEquals("hello", c.getValue());
    }

    @Test
    void any_accepts_list() {
        var c = new ContainsAny();
        var list = Arrays.asList(1, 2, 3);
        c.setValue(list);
        assertEquals(list, c.getValue());
    }

    @Test
    void any_accepts_map() {
        var c = new ContainsAny();
        var map = new HashMap<String, Integer>();
        map.put("a", 1);
        c.setValue(map);
        assertEquals(map, c.getValue());
    }

    @Test
    void any_accepts_nested_struct() {
        var c = new ContainsAny();
        var p = new Point(10, 20);
        c.setValue(p);
        assertEquals(p, c.getValue());
    }

    @Test
    void multiple_any_fields() {
        var m = new MultipleAny();
        m.setFirst(1);
        m.setSecond("two");
        m.setThird(Arrays.asList(3, 4, 5));
        assertEquals(1, m.getFirst());
        assertEquals("two", m.getSecond());
        assertNotNull(m.getThird());
    }

    @Test
    void any_with_other_fields() {
        var a = new AnyWithOtherFields();
        a.setId(100);
        a.setName("test");
        a.setPayload("payload data");
        assertEquals(100, a.getId());
        assertEquals("test", a.getName());
        assertEquals("payload data", a.getPayload());
    }

    @Test
    void sequence_of_any() {
        var s = new SequenceOfAny();
        s.getItems().add(1);
        s.getItems().add("two");
        s.getItems().add(3.14);
        assertEquals(3, s.getItems().size());
    }

    @Test
    void map_with_any() {
        var m = new MapWithAny();
        m.getProperties().put("int", 42);
        m.getProperties().put("str", "hello");
        assertEquals(42, m.getProperties().get("int"));
        assertEquals("hello", m.getProperties().get("str"));
    }

    @Test
    void optional_any_default() {
        var o = new OptionalAny();
        assertNull(o.getMaybeValue());
    }

    @Test
    void optional_any_with_value() {
        var o = new OptionalAny();
        o.setMaybeValue("set value");
        assertEquals("set value", o.getMaybeValue());
    }

    @Test
    void any_alias_typedef() {
        var u = new UsingAnyAlias();
        assertNotNull(u);
    }

    @Test
    void using_any_alias() {
        var u = new UsingAnyAlias();
        u.setData(123);
        assertEquals(123, u.getData());
    }

    @Test
    void any_can_be_reassigned() {
        var c = new ContainsAny();
        c.setValue(42);
        assertEquals(42, c.getValue());
        c.setValue("now a string");
        assertEquals("now a string", c.getValue());
        c.setValue(Arrays.asList(1, 2, 3));
        assertTrue(c.getValue() instanceof List);
    }
}
