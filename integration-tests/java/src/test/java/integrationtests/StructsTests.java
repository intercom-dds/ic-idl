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
import struct_types.*;

class StructsTests {

    @Test
    void point_defaultConstructor() {
        var p = new Point();
        assertEquals(0, p.getX());
        assertEquals(0, p.getY());
    }

    @Test
    void point_parameterizedConstructor() {
        var p = new Point(10, 20);
        assertEquals(10, p.getX());
        assertEquals(20, p.getY());
    }

    @Test
    void point_setters() {
        var p = new Point();
        p.setX(5);
        p.setY(15);
        assertEquals(5, p.getX());
        assertEquals(15, p.getY());
    }

    @Test
    void point_copyConstructor() {
        var p1 = new Point(3, 4);
        var p2 = new Point(p1);
        assertEquals(p1.getX(), p2.getX());
        assertEquals(p1.getY(), p2.getY());
        assertNotSame(p1, p2);
    }

    @Test
    void point_equality() {
        var p1 = new Point(1, 2);
        var p2 = new Point(1, 2);
        var p3 = new Point(3, 4);
        assertEquals(p1, p2);
        assertNotEquals(p1, p3);
    }

    @Test
    void point_hashCode() {
        var p1 = new Point(1, 2);
        var p2 = new Point(1, 2);
        assertEquals(p1.hashCode(), p2.hashCode());
    }

    @Test
    void point_clone() {
        var p1 = new Point(5, 10);
        var p2 = p1.clone();
        assertEquals(p1, p2);
        assertNotSame(p1, p2);
    }

    @Test
    void point3d_inheritance() {
        var p = new Point3D();
        p.setX(1);
        p.setY(2);
        p.setZ(3);
        assertEquals(1, p.getX());
        assertEquals(2, p.getY());
        assertEquals(3, p.getZ());
        assertTrue(p instanceof Point);
    }

    @Test
    void point3d_parameterizedConstructor() {
        var p = new Point3D(10, 20, 30);
        assertEquals(10, p.getX());
        assertEquals(20, p.getY());
        assertEquals(30, p.getZ());
    }

    @Test
    void point3d_copyConstructor() {
        var p1 = new Point3D(1, 2, 3);
        var p2 = new Point3D(p1);
        assertEquals(p1.getX(), p2.getX());
        assertEquals(p1.getY(), p2.getY());
        assertEquals(p1.getZ(), p2.getZ());
        assertNotSame(p1, p2);
    }

    @Test
    void point4d_deepInheritance() {
        var p = new Point4D();
        p.setX(1);
        p.setY(2);
        p.setZ(3);
        p.setW(4);
        assertEquals(1, p.getX());
        assertEquals(2, p.getY());
        assertEquals(3, p.getZ());
        assertEquals(4, p.getW());
        assertTrue(p instanceof Point3D);
        assertTrue(p instanceof Point);
    }

    @Test
    void point4d_parameterizedConstructor() {
        var p = new Point4D(10, 20, 30, 40);
        assertEquals(10, p.getX());
        assertEquals(20, p.getY());
        assertEquals(30, p.getZ());
        assertEquals(40, p.getW());
    }

    @Test
    void point4d_copyConstructor() {
        var p1 = new Point4D(5, 6, 7, 8);
        var p2 = new Point4D(p1);
        assertEquals(p1.getX(), p2.getX());
        assertEquals(p1.getY(), p2.getY());
        assertEquals(p1.getZ(), p2.getZ());
        assertEquals(p1.getW(), p2.getW());
        assertNotSame(p1, p2);
    }

    @Test
    void empty_canBeInstantiated() {
        var e = new Empty();
        assertNotNull(e);
    }

    @Test
    void allPrimitives_defaults() {
        var a = new AllPrimitives();
        assertFalse(a.getBoolVal());
        assertEquals(0, a.getByteVal());
        assertEquals(0, a.getShortVal());
        assertEquals(0, a.getUshortVal());
        assertEquals(0, a.getLongVal());
        assertEquals(0, a.getUlongVal());
        assertEquals(0L, a.getLonglongVal());
        assertEquals(0L, a.getUlonglongVal());
        assertEquals(0.0f, a.getFloatVal());
        assertEquals(0.0, a.getDoubleVal());
        assertEquals("", a.getStringVal());
    }

    @Test
    void allPrimitives_setAndGet() {
        var a = new AllPrimitives();
        a.setBoolVal(true);
        a.setLongVal(42);
        a.setDoubleVal(3.14);
        a.setStringVal("hello");

        assertTrue(a.getBoolVal());
        assertEquals(42, a.getLongVal());
        assertEquals(3.14, a.getDoubleVal(), 0.001);
        assertEquals("hello", a.getStringVal());
    }

    @Test
    void withSequence_emptyByDefault() {
        var w = new WithSequence();
        assertNotNull(w.getNumbers());
        assertTrue(w.getNumbers().isEmpty());
        assertNotNull(w.getNames());
        assertTrue(w.getNames().isEmpty());
    }

    @Test
    void withSequence_canAddElements() {
        var w = new WithSequence();
        w.getNumbers().add(1);
        w.getNumbers().add(2);
        w.getNumbers().add(3);
        assertEquals(3, w.getNumbers().size());
        assertEquals(1, w.getNumbers().get(0));
    }

    @Test
    void withMap_emptyByDefault() {
        var w = new WithMap();
        assertNotNull(w.getStringToInt());
        assertTrue(w.getStringToInt().isEmpty());
    }

    @Test
    void withMap_canAddEntries() {
        var w = new WithMap();
        w.getStringToInt().put("key1", 100);
        w.getStringToInt().put("key2", 200);
        assertEquals(2, w.getStringToInt().size());
        assertEquals(100, w.getStringToInt().get("key1"));
    }

    @Test
    void withArray_hasFixedSize() {
        var w = new WithArray();
        assertEquals(5, w.getFixedNumbers().length);
    }

    @Test
    void withArray_canSetValues() {
        var w = new WithArray();
        w.getFixedNumbers()[0] = 10;
        w.getFixedNumbers()[4] = 50;
        assertEquals(10, w.getFixedNumbers()[0]);
        assertEquals(50, w.getFixedNumbers()[4]);
    }

    @Test
    void rectangle_nestedStruct() {
        var r = new Rectangle();
        r.setTopLeft(new Point(0, 0));
        r.setBottomRight(new Point(10, 10));
        assertEquals(0, r.getTopLeft().getX());
        assertEquals(10, r.getBottomRight().getX());
    }

    @Test
    void rectangle_equality() {
        var r1 = new Rectangle(new Point(0, 0), new Point(10, 10));
        var r2 = new Rectangle(new Point(0, 0), new Point(10, 10));
        var r3 = new Rectangle(new Point(0, 0), new Point(20, 20));
        assertEquals(r1, r2);
        assertNotEquals(r1, r3);
    }

    @Test
    void withDefaults_defaults() {
        var w = new WithDefaults();
        assertEquals(0, w.getCount());
        assertEquals("", w.getName());
        assertEquals(0.0, w.getValue());
    }

    @Test
    void withDefaults_canSetValues() {
        var w = new WithDefaults();
        w.setCount(42);
        w.setName("test");
        w.setValue(3.14);
        assertEquals(42, w.getCount());
        assertEquals("test", w.getName());
        assertEquals(3.14, w.getValue(), 0.001);
    }
}
