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
import union_types.*;

class UnionsTests {

    @Test
    void intOrString_defaultsToDefaultVal() {
        var u = new IntOrString();
        assertEquals(0, u.getDiscriminator());
        assertFalse(u.getDefaultVal());
    }

    @Test
    void defaultDiscriminatorCase_defaultsToValue() {
        var u = new DefaultDiscriminatorCase();
        assertEquals(0, u.getDiscriminator());
        assertEquals(0, u.getValue());
    }

    @Test
    void intOrString_setIntVal() {
        var u = new IntOrString();
        u.setIntVal(42);
        assertEquals(1, u.getDiscriminator());
        assertEquals(42, u.getIntVal());
    }

    @Test
    void intOrString_setStrVal() {
        var u = new IntOrString();
        u.setStrVal("hello");
        assertEquals(2, u.getDiscriminator());
        assertEquals("hello", u.getStrVal());
    }

    @Test
    void intOrString_setDefaultVal() {
        var u = new IntOrString();
        u.setIntVal(42);
        u.setDefaultVal(true);
        assertEquals(0, u.getDiscriminator());
        assertTrue(u.getDefaultVal());
    }

    @Test
    void intOrString_wrongAccessThrows() {
        var u = new IntOrString();
        u.setStrVal("test");
        assertThrows(IllegalStateException.class, () -> u.getIntVal());
    }

    @Test
    void intOrString_copyConstructor() {
        var u1 = new IntOrString();
        u1.setStrVal("copy me");
        var u2 = new IntOrString(u1);
        assertEquals(u1.getDiscriminator(), u2.getDiscriminator());
        assertEquals(u1.getStrVal(), u2.getStrVal());
        assertNotSame(u1, u2);
    }

    @Test
    void intOrString_equality() {
        var u1 = new IntOrString();
        u1.setIntVal(100);
        var u2 = new IntOrString();
        u2.setIntVal(100);
        var u3 = new IntOrString();
        u3.setIntVal(200);
        assertEquals(u1, u2);
        assertNotEquals(u1, u3);
    }

    @Test
    void intOrString_hashCodeConsistent() {
        var u1 = new IntOrString();
        u1.setIntVal(42);
        var u2 = new IntOrString();
        u2.setIntVal(42);
        assertEquals(u1.hashCode(), u2.hashCode());
    }

    @Test
    void intOrString_clone() {
        var u1 = new IntOrString();
        u1.setStrVal("original");
        var u2 = u1.clone();
        assertEquals(u1, u2);
        assertNotSame(u1, u2);
    }

    @Test
    void boolSwitch_defaultsToFalse() {
        var u = new BoolSwitch();
        assertFalse(u.getDiscriminator());
        assertEquals("", u.getFalseVal());
    }

    @Test
    void boolSwitch_setTrueVal() {
        var u = new BoolSwitch();
        u.setTrueVal(123);
        assertTrue(u.getDiscriminator());
        assertEquals(123, u.getTrueVal());
    }

    @Test
    void boolSwitch_setFalseVal() {
        var u = new BoolSwitch();
        u.setFalseVal("false branch");
        assertFalse(u.getDiscriminator());
        assertEquals("false branch", u.getFalseVal());
    }

    @Test
    void boolSwitch_wrongAccessThrows() {
        var u = new BoolSwitch();
        u.setTrueVal(1);
        assertThrows(IllegalStateException.class, () -> u.getFalseVal());
    }

    @Test
    void boolSwitch_equality() {
        var u1 = new BoolSwitch();
        u1.setFalseVal("test");
        var u2 = new BoolSwitch();
        u2.setFalseVal("test");
        assertEquals(u1, u2);
    }

    @Test
    void typedValue_defaultsToIntKind() {
        var u = new TypedValue();
        assertEquals(ValueKind.INT_KIND, u.getDiscriminator());
        assertEquals(0, u.getIntValue());
    }

    @Test
    void typedValue_setIntValue() {
        var u = new TypedValue();
        u.setIntValue(42);
        assertEquals(ValueKind.INT_KIND, u.getDiscriminator());
        assertEquals(42, u.getIntValue());
    }

    @Test
    void typedValue_setFloatValue() {
        var u = new TypedValue();
        u.setFloatValue(3.14);
        assertEquals(ValueKind.FLOAT_KIND, u.getDiscriminator());
        assertEquals(3.14, u.getFloatValue(), 0.001);
    }

    @Test
    void typedValue_setStringValue() {
        var u = new TypedValue();
        u.setStringValue("hello");
        assertEquals(ValueKind.STRING_KIND, u.getDiscriminator());
        assertEquals("hello", u.getStringValue());
    }

    @Test
    void typedValue_wrongAccessThrows() {
        var u = new TypedValue();
        u.setIntValue(1);
        assertThrows(IllegalStateException.class, () -> u.getFloatValue());
        assertThrows(IllegalStateException.class, () -> u.getStringValue());
    }

    @Test
    void typedValue_equality() {
        var u1 = new TypedValue();
        u1.setFloatValue(2.718);
        var u2 = new TypedValue();
        u2.setFloatValue(2.718);
        assertEquals(u1, u2);
    }

    @Test
    void multiCase_defaultsToFlag() {
        var u = new MultiCase();
        assertEquals(0, u.getDiscriminator());
        assertFalse(u.getFlag());
    }

    @Test
    void multiCase_setSmallVal() {
        var u = new MultiCase();
        u.setSmallVal(42);
        assertEquals(1, u.getDiscriminator());
        assertEquals(42, u.getSmallVal());
    }

    @Test
    void multiCase_setSmallValWithDiscriminator() {
        var u = new MultiCase();
        u.setSmallVal(99, 2);
        assertEquals(2, u.getDiscriminator());
        assertEquals(99, u.getSmallVal());
    }

    @Test
    void multiCase_setSmallValWithDiscriminator3() {
        var u = new MultiCase();
        u.setSmallVal(77, 3);
        assertEquals(3, u.getDiscriminator());
        assertEquals(77, u.getSmallVal());
    }

    @Test
    void multiCase_setTextVal() {
        var u = new MultiCase();
        u.setTextVal("hello");
        assertEquals(10, u.getDiscriminator());
        assertEquals("hello", u.getTextVal());
    }

    @Test
    void multiCase_setTextValWithDiscriminator() {
        var u = new MultiCase();
        u.setTextVal("world", 20);
        assertEquals(20, u.getDiscriminator());
        assertEquals("world", u.getTextVal());
    }

    @Test
    void multiCase_setFlag() {
        var u = new MultiCase();
        u.setFlag(true);
        assertEquals(0, u.getDiscriminator());
        assertTrue(u.getFlag());
    }

    @Test
    void multiCase_wrongAccessThrows() {
        var u = new MultiCase();
        u.setSmallVal(1);
        assertThrows(IllegalStateException.class, () -> u.getTextVal());
        assertThrows(IllegalStateException.class, () -> u.getFlag());
    }

    @Test
    void multiCase_invalidDiscriminatorForSmallValThrows() {
        var u = new MultiCase();
        assertThrows(IllegalStateException.class, () -> u.setSmallVal(1, 5));
    }

    @Test
    void multiCase_equality() {
        var u1 = new MultiCase();
        u1.setTextVal("test", 20);
        var u2 = new MultiCase();
        u2.setTextVal("test", 20);
        var u3 = new MultiCase();
        u3.setTextVal("test", 10);
        assertEquals(u1, u2);
        assertNotEquals(u1, u3);
    }
}
