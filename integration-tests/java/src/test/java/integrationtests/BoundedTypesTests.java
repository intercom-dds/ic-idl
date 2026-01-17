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

import bounded_types.*;
import org.junit.jupiter.api.Test;

class BoundedTypesTests {

    @Test
    void bounded_string_typedef_maps_to_str() {
        var b = new BoundedFields();
        b.setName("short name");
        b.setDescription("longer description");
        assertEquals("short name", b.getName());
        assertEquals("longer description", b.getDescription());
    }

    @Test
    void bounded_sequence_typedef_maps_to_list() {
        var b = new BoundedFields();
        b.getValues().add(1);
        b.getValues().add(2);
        b.getTags().add("tag1");
        assertEquals(2, b.getValues().size());
        assertEquals(1, b.getTags().size());
    }

    @Test
    void bounded_fields_struct() {
        var b = new BoundedFields("name", "desc", java.util.Arrays.asList(1, 2), java.util.Arrays.asList("a"));
        assertEquals("name", b.getName());
        assertEquals("desc", b.getDescription());
        assertEquals(2, b.getValues().size());
        assertEquals(1, b.getTags().size());
    }

    @Test
    void bounded_fields_annotations() {
        var b = new BoundedFields();
        assertInstanceOf(String.class, b.getName());
        assertInstanceOf(java.util.List.class, b.getValues());
    }

    @Test
    void nested_bounded_struct() {
        var n = new NestedBounded();
        assertNotNull(n.getMatrix());
        assertNotNull(n.getIndexedLists());
    }

    @Test
    void nested_bounded_annotations() {
        var n = new NestedBounded();
        n.getMatrix().add(java.util.Arrays.asList(1, 2, 3));
        n.getIndexedLists().put("list", java.util.Arrays.asList(4, 5));
        assertEquals(1, n.getMatrix().size());
        assertEquals(1, n.getIndexedLists().size());
    }

    @Test
    void typedef_chain_with_bounds() {
        var b = new BoundedFields();
        b.setName("test name");
        assertEquals("test name", b.getName());
    }

    @Test
    void mixed_bounds_struct() {
        var m = new MixedBounds();
        assertNotNull(m);
    }

    @Test
    void mixed_bounds_annotations() {
        var m = new MixedBounds();
        m.setBoundedString("bounded");
        m.setUnboundedString("unbounded");
        assertEquals("bounded", m.getBoundedString());
        assertEquals("unbounded", m.getUnboundedString());
    }
}
