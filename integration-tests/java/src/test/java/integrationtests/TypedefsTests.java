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

import org.junit.jupiter.api.Test;
import typedef_types.*;

class TypedefsTests {

    @Test
    void testPointTypedefs() {
        var p1 = new Point(10, 20);
        assertEquals(10, p1.getX());
        assertEquals(20, p1.getY());

        var p2 = new Point();
        p2.setX(5);
        p2.setY(10);
        assertEquals(5, p2.getX());
        assertEquals(10, p2.getY());
    }

    @Test
    void sequence_typedef_values() {
        var c = new Container();
        c.getNumbers().add(1);
        c.getNumbers().add(2);
        assertEquals(2, c.getNumbers().size());
    }

    @Test
    void nested_typedef_values() {
        var m = new Measurement();
        m.setName("test");
        m.setValue(42);
        assertEquals("test", m.getName());
        assertEquals(42, m.getValue());
    }

    @Test
    void map_typedef_values() {
        var c = new Container();
        c.getLookup().put("key", 100);
        assertEquals(100, c.getLookup().get("key"));
    }

    @Test
    void array_typedef_value() {
        var w = new WithArrayTypedef();
        assertNotNull(w.getValues());
        assertEquals(10, w.getValues().length);
    }

    @Test
    void person_struct_field_types() {
        var person = new Person();
        person.setName("Alice");
        person.setAge(30);
        person.setActive(true);
        assertEquals("Alice", person.getName());
        assertEquals(30, person.getAge());
        assertTrue(person.getActive());
    }

    @Test
    void person_struct_values() {
        var person = new Person("Bob", 25, false);
        assertEquals("Bob", person.getName());
        assertEquals(25, person.getAge());
        assertFalse(person.getActive());
    }

    @Test
    void container_struct_field_types() {
        var c = new Container();
        assertNotNull(c.getNumbers());
        assertNotNull(c.getLabels());
        assertNotNull(c.getLookup());
    }

    @Test
    void container_struct_values() {
        var c = new Container();
        c.getNumbers().add(1);
        c.getNumbers().add(2);
        c.getLabels().add("a");
        c.getLabels().add("b");
        c.getLookup().put("x", 10);
        assertEquals(2, c.getNumbers().size());
        assertEquals(2, c.getLabels().size());
        assertEquals(1, c.getLookup().size());
    }

    @Test
    void nested_typedef_in_struct() {
        var m = new Measurement("temp", 100);
        assertEquals("temp", m.getName());
        assertEquals(100, m.getValue());
    }

    @Test
    void nested_typedef_struct_values() {
        var m = new Measurement();
        m.setName("pressure");
        m.setValue(50);
        assertEquals("pressure", m.getName());
        assertEquals(50, m.getValue());
    }

    @Test
    void array_typedef_in_struct() {
        var w = new WithArrayTypedef();
        assertNotNull(w.getValues());
    }

    @Test
    void array_typedef_struct_values() {
        var w = new WithArrayTypedef();
        w.getValues()[0] = 100;
        w.getValues()[4] = 500;
        assertEquals(100, w.getValues()[0]);
        assertEquals(500, w.getValues()[4]);
    }

    @Test
    void deep_typedef_chain_values() {
        var d = new DeepChainStruct();
        d.setDeepInt(42);
        assertEquals(42, d.getDeepInt());
    }

    @Test
    void deep_sequence_typedef_chain() {
        var d = new DeepChainStruct();
        d.getDeepSeq().add(1);
        d.getDeepSeq().add(2);
        d.getDeepSeq().add(3);
        assertEquals(3, d.getDeepSeq().size());
    }

    @Test
    void deep_map_typedef_chain() {
        var d = new DeepChainStruct();
        d.getDeepMap().put("key", 100);
        assertEquals(1, d.getDeepMap().size());
    }

    @Test
    void deep_chain_struct_field_types() {
        var d = new DeepChainStruct();
        assertNotNull(d.getDeepSeq());
        assertNotNull(d.getDeepMap());
    }

    @Test
    void deep_chain_struct_values() {
        var d = new DeepChainStruct();
        d.setDeepInt(999);
        assertEquals(999, d.getDeepInt());
    }
}
