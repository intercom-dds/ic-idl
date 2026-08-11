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

import default_types.*;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;

class DefaultsTests {

    @Test
    void const_string_values() {
        assertEquals("unnamed", DEFAULT_NAME.value);
        assertEquals(100, DEFAULT_COUNT.value);
        assertEquals(0.5, DEFAULT_RATE.value, 0.001);
    }

    @Test
    @Disabled("struct-typed constant initializers are not yet implemented in the Java backend")
    void struct_const_initializer() {
        assertNotNull(DEFAULT_INNER.value);
        assertNotNull(NESTED_INNER.value);
        assertEquals(10, DEFAULT_INNER.value.getX());
        assertEquals("default", DEFAULT_INNER.value.getY());
        assertEquals(99, NESTED_INNER.value.getX());
        assertEquals("nested", NESTED_INNER.value.getY());
    }

    @Test
    @Disabled("@optional is not yet mapped to a nullable type in the Java backend")
    void optional_fields_are_null_by_default() {
        var o = new OptionalFields();
        assertNull(o.getMaybeInt());
        assertNull(o.getMaybeString());
        assertNull(o.getMaybeStruct());
    }

    @Test
    void optional_fields_can_be_set() {
        var o = new OptionalFields();
        o.setMaybeInt(42);
        o.setMaybeString("hello");
        o.setMaybeStruct(new Inner());
        assertEquals(42, o.getMaybeInt());
        assertEquals("hello", o.getMaybeString());
        assertNotNull(o.getMaybeStruct());
    }

    @Test
    void enum_default_literal_exists() {
        assertNotNull(Priority.LOW);
        assertNotNull(Priority.MEDIUM);
        assertNotNull(Priority.HIGH);
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void primitive_bool_default() {
        var p = new PrimitiveDefaults();
        assertTrue(p.getBoolTrue());
        assertFalse(p.getBoolFalse());
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void primitive_int_default() {
        var p = new PrimitiveDefaults();
        assertEquals(42, p.getIntValue());
        assertEquals(-100, p.getIntNegative());
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void primitive_float_default() {
        var p = new PrimitiveDefaults();
        assertEquals(3.14159, p.getFloatValue(), 0.00001);
        assertEquals(-0.5, p.getFloatNegative(), 0.001);
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void primitive_string_default() {
        var p = new PrimitiveDefaults();
        assertEquals("hello", p.getStringValue());
        assertEquals("unnamed", p.getStringFromConst());
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void array_default_values() {
        var a = new ArrayDefaults();
        assertEquals(1, a.getArrayValues()[0]);
        assertEquals(2, a.getArrayValues()[1]);
        assertEquals(3, a.getArrayValues()[2]);
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void sequence_default_values() {
        var s = new SequenceDefaults();
        assertEquals(5, s.getSeqValues().size());
        assertEquals(1, s.getSeqValues().get(0));
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void map_default_values() {
        var m = new MapDefaults();
        assertEquals(1, m.getMapValues().get("one"));
        assertEquals(2, m.getMapValues().get("two"));
    }

    @Test
    @Disabled("@default annotations are not yet implemented in the Java backend")
    void enum_field_default() {
        var e = new EnumDefaults();
        assertEquals(Priority.HIGH, e.getPriorityHigh());
    }

    @Test
    void primitive_defaults_exist() {
        var p = new PrimitiveDefaults();
        assertNotNull(p);
        assertFalse(p.getBoolEmpty());
        assertEquals(0, p.getIntEmpty());
        assertEquals(0.0, p.getFloatEmpty(), 0.001);
        assertEquals("", p.getStringEmpty());
    }

    @Test
    void sequence_defaults_exist() {
        var s = new SequenceDefaults();
        assertNotNull(s.getSeqEmpty());
        assertNotNull(s.getSeqValues());
    }

    @Test
    void map_defaults_exist() {
        var m = new MapDefaults();
        assertNotNull(m.getMapEmpty());
        assertNotNull(m.getMapValues());
    }

    @Test
    void enum_defaults_exist() {
        var e = new EnumDefaults();
        assertNotNull(e.getPriorityEmpty());
        assertNotNull(e.getPriorityHigh());
    }
}
