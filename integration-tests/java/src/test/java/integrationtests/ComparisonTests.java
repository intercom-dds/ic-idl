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

import exception_types.*;
import org.junit.jupiter.api.Test;
import struct_types.*;
import union_types.*;

class ComparisonTests {
    @Test
    void struct_equality() {
        var p1 = new Point(1, 2);
        var p2 = new Point(1, 2);
        var p3 = new Point(3, 4);
        assertEquals(p1, p2);
        assertNotEquals(p1, p3);
    }

    @Test
    void struct_hashable() {
        var p1 = new Point(1, 2);
        var p2 = new Point(1, 2);
        assertEquals(p1.hashCode(), p2.hashCode());
        var set = new java.util.HashSet<Point>();
        set.add(p1);
        assertTrue(set.contains(p2));
    }

    @Test
    void union_equality() {
        var u1 = new IntOrString();
        u1.setIntVal(42);
        var u2 = new IntOrString();
        u2.setIntVal(42);
        var u3 = new IntOrString();
        u3.setIntVal(99);
        assertEquals(u1, u2);
        assertNotEquals(u1, u3);
    }
}
