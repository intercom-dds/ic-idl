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
import valuetype_types.*;

class ValuetypesTests {

    @Test
    void valuetype_instantiation() {
        var v = new SimpleValue();
        v.id = 10;
        v.name = "test";
        assertEquals(10, v.id);
        assertEquals("test", v.name);
    }

    @Test
    void valuetype_defaults() {
        var v = new SimpleValue();
        assertEquals(0, v.id);
        assertNull(v.name);
    }

    @Test
    void valuetype_inheritance() {
        var v = new DerivedValue();
        assertTrue(v instanceof SimpleValueAbstract);
    }

    @Test
    void empty_valuetype() {
        var v = new Empty();
        assertNotNull(v);
    }

    @Test
    void valuetype_with_sequence() {
        var v = new WithSequence();
        assertNotNull(v.numbers);
        assertNotNull(v.names);
    }

    @Test
    void valuetype_equality() {
        // TODO: should valuetypes override `equals`?
        var v1 = new SimpleValue();
        v1.id = 42;
        v1.name = "test";
        var v2 = new SimpleValue();
        v2.id = 42;
        v2.name = "test";
        assertNotSame(v1, v2);
    }

    @Test
    void valuetype_supports_interface() {
        var v = new IdentifiableValue();
        assertTrue(v instanceof Identifiable);
    }

    @Test
    void valuetype_supports_named() {
        var v = new NamedValue();
        assertTrue(v instanceof Named);
    }

    @Test
    void valuetype_inheritance_and_supports() {
        var v = new FullValue();
        assertTrue(v instanceof SimpleValueAbstract);
        assertTrue(v instanceof Identifiable);
    }

    @Test
    void valuetype_field_types() {
        var v = new SimpleValue();
        v.id = Integer.MAX_VALUE;
        v.name = "string value";
        assertEquals(Integer.MAX_VALUE, v.id);
        assertEquals("string value", v.name);
    }

    @Test
    void valuetype_sequence_field_types() {
        var v = new WithSequence();
        v.numbers = new java.util.ArrayList<>();
        v.numbers.add(1);
        v.names = new java.util.ArrayList<>();
        v.names.add("a");
        assertEquals(1, v.numbers.size());
        assertEquals(1, v.names.size());
    }

    @Test
    void valuetype_derived_field_types() {
        var v = new DerivedValue();
        v.description = "test description";
        assertEquals("test description", v.description);
    }
}
