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

import annotation_types.*;
import java.util.ArrayList;
import org.junit.jupiter.api.Test;

class AnnotationsTests {

    @Test
    void keyed_struct_exists() {
        var k = new KeyedStruct();
        k.setId(1);
        k.setName("test");
        k.setValue(3.14);
        assertEquals(1, k.getId());
        assertEquals("test", k.getName());
        assertEquals(3.14, k.getValue(), 0.001);
    }

    @Test
    void multi_key_struct() {
        var m = new MultiKeyStruct();
        m.setNamespace("ns1");
        m.setId(1);
        m.setData("data");
        assertEquals("ns1", m.getNamespace());
        assertEquals(1, m.getId());
        assertEquals("data", m.getData());
    }

    @Test
    void optional_fields_default_values() {
        var o = new OptionalStruct();
        assertNull(o.getOptionalInt());
        assertNull(o.getOptionalString());
        assertNull(o.getOptionalSeq());
    }

    @Test
    void optional_fields_can_be_set() {
        var o = new OptionalStruct();
        o.setOptionalInt(42);
        o.setOptionalString("hello");
        o.setOptionalSeq(new ArrayList<>());
        o.getOptionalSeq().add(1);
        assertEquals(42, o.getOptionalInt());
        assertEquals("hello", o.getOptionalString());
        assertEquals(1, o.getOptionalSeq().size());
    }

    @Test
    void optional_type_annotations() {
        var o = new OptionalStruct();
        assertNull(o.getOptionalInt());
        o.setOptionalInt(100);
        assertEquals(100, o.getOptionalInt());
    }

    @Test
    void nested_struct() {
        var n = new NestedStruct();
        assertNotNull(n);
    }

    @Test
    void shared_refs_struct() {
        var s = new SharedRefs();
        assertNotNull(s);
    }

    @Test
    void combined_annotations() {
        var c = new CombinedAnnotations();
        c.setId(1);
        c.setMaybeSharedName("test");
        assertEquals(1, c.getId());
        assertEquals("test", c.getMaybeSharedName());
    }

    @Test
    void annotated_interface_exists() {
        assertTrue(AnnotatedInterface.class.isInterface());
    }

    @Test
    void topic_struct() {
        var t = new TopicMessage();
        t.setMessageId(1);
        t.setPayload("data");
        assertEquals(1, t.getMessageId());
        assertEquals("data", t.getPayload());
    }

    @Test
    void mutable_struct() {
        var m = new MutableStruct();
        m.setVersion(42);
        assertEquals(42, m.getVersion());
    }

    @Test
    void final_struct() {
        var f = new FinalStruct();
        f.setFixedField(100);
        assertEquals(100, f.getFixedField());
    }
}
