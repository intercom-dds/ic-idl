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

import enum_types.*;
import org.junit.jupiter.api.Test;

class EnumsTests {

    @Test
    void color_values() {
        assertEquals(0, Color.RED.getValue());
        assertEquals(1, Color.GREEN.getValue());
        assertEquals(2, Color.BLUE.getValue());
    }

    @Test
    void color_valueOf() {
        assertEquals(Color.RED, Color.valueOf(0));
        assertEquals(Color.GREEN, Color.valueOf(1));
        assertEquals(Color.BLUE, Color.valueOf(2));
    }

    @Test
    void color_invalidValueThrows() {
        assertThrows(IllegalArgumentException.class, () -> Color.valueOf(99));
    }

    @Test
    void status_values() {
        assertEquals(0, Status.OK.getValue());
        assertEquals(100, Status.WARNING.getValue());
        assertEquals(200, Status.ERROR.getValue());
    }

    @Test
    void gappedEnum_values() {
        assertEquals(0, GappedEnum.FIRST.getValue());
        assertEquals(5, GappedEnum.SECOND.getValue());
        assertEquals(10, GappedEnum.THIRD.getValue());
        assertEquals(100, GappedEnum.FOURTH.getValue());
    }

    @Test
    void gappedEnum_valueOf() {
        assertEquals(GappedEnum.FIRST, GappedEnum.valueOf(0));
        assertEquals(GappedEnum.SECOND, GappedEnum.valueOf(5));
        assertEquals(GappedEnum.THIRD, GappedEnum.valueOf(10));
        assertEquals(GappedEnum.FOURTH, GappedEnum.valueOf(100));
    }

    @Test
    void negativeEnum_values() {
        assertEquals(-2, NegativeEnum.NEG_TWO.getValue());
        assertEquals(-1, NegativeEnum.NEG_ONE.getValue());
        assertEquals(0, NegativeEnum.ZERO.getValue());
        assertEquals(1, NegativeEnum.POS_ONE.getValue());
    }

    @Test
    void mixedEnum_values() {
        assertEquals(0, MixedEnum.AUTO_FIRST.getValue());
        assertEquals(10, MixedEnum.EXPLICIT_TEN.getValue());
        assertEquals(11, MixedEnum.AUTO_ELEVEN.getValue());
        assertEquals(100, MixedEnum.EXPLICIT_HUNDRED.getValue());
        assertEquals(101, MixedEnum.AUTO_HUNDRED_ONE.getValue());
    }

    @Test
    void enum_nameMethod() {
        assertEquals("RED", Color.RED.name());
        assertEquals("GREEN", Color.GREEN.name());
        assertEquals("BLUE", Color.BLUE.name());
    }

    @Test
    void enum_ordinalMethod() {
        assertEquals(0, Color.RED.ordinal());
        assertEquals(1, Color.GREEN.ordinal());
        assertEquals(2, Color.BLUE.ordinal());
    }

    @Test
    void enum_valuesArray() {
        Color[] colors = Color.values();
        assertEquals(3, colors.length);
        assertEquals(Color.RED, colors[0]);
        assertEquals(Color.GREEN, colors[1]);
        assertEquals(Color.BLUE, colors[2]);
    }
}
