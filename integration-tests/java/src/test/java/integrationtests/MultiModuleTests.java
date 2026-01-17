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

import constants_only.*;
import enums_only.*;
import module_a.*;
import module_b.*;
import org.junit.jupiter.api.Test;

class MultiModuleTests {

    @Test
    void module_a_exists() {
        assertDoesNotThrow(() -> new StructA1());
    }

    @Test
    void module_b_exists() {
        assertDoesNotThrow(() -> new StructB1());
    }

    @Test
    void module_a_first_opening() {
        var s = new StructA1();
        s.setValue(10);
        assertEquals(10, s.getValue());
        assertEquals(100, CONST_A1.value);
        assertNotNull(EnumA.X);
    }

    @Test
    void module_a_second_opening() {
        var s = new StructA2();
        s.setData(20.0);
        assertEquals(20.0, s.getData(), 0.001);
        assertEquals(101, CONST_A2.value);
        assertNotNull(EnumA2.P);
    }

    @Test
    void module_a_third_opening() {
        var s = new StructA3();
        s.setFlag(true);
        assertTrue(s.getFlag());
        assertEquals(102, CONST_A3.value);
    }

    @Test
    void module_b_both_openings() {
        var s1 = new StructB1();
        s1.setName("test");
        assertEquals("test", s1.getName());
        assertEquals(200, CONST_B1.value);

        var s2 = new StructB2();
        s2.setId(20);
        assertEquals(20, s2.getId());
        assertEquals(201, CONST_B2.value);
    }

    @Test
    void reopened_module_types_can_reference_earlier() {
        var s2 = new StructA2();
        s2.setRefToA1(new StructA1());
        assertNotNull(s2.getRefToA1());
    }

    @Test
    void reopened_module_chain() {
        var s3 = new StructA3();
        s3.setA1(new StructA1());
        s3.setA2(new StructA2());
        assertNotNull(s3.getA1());
        assertNotNull(s3.getA2());
    }

    @Test
    void constants_only_module() {
        assertEquals(1, C1.value);
        assertEquals(2, C2.value);
        assertEquals(3, C3.value);
    }

    @Test
    void enums_only_module() {
        assertEquals(0, Color.RED.getValue());
        assertEquals(1, Color.GREEN.getValue());
        assertEquals(0, Size.SMALL.getValue());
        assertEquals(1, Size.MEDIUM.getValue());
    }
}
