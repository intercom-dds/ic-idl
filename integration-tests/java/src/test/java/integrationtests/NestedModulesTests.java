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

import nested_module_types.*;
import nested_module_types.level1.*;
import nested_module_types.level1.level2.*;
import nested_module_types.level1.level2.level3.*;
import nested_module_types.sibling.*;
import org.junit.jupiter.api.Test;

class NestedModulesTests {

    @Test
    void top_level_types_exist() {
        assertNotNull(TopLevelStruct.class);
        assertNotNull(TopLevelEnum.class);
    }

    @Test
    void nested_module_level1_exists() {
        assertNotNull(Level1Struct.class);
        assertNotNull(Level1Enum.class);
    }

    @Test
    void nested_module_level2_exists() {
        assertNotNull(Level2Struct.class);
    }

    @Test
    void nested_module_level3_exists() {
        assertNotNull(Level3Struct.class);
        assertEquals(42, DEEP_CONST.value);
    }

    @Test
    void sibling_module_exists() {
        assertNotNull(SiblingStruct.class);
        assertNotNull(CrossRef.class);
    }

    @Test
    void top_level_struct_instantiation() {
        var t = new TopLevelStruct();
        t.setValue(100);
        assertEquals(100, t.getValue());
    }

    @Test
    void level1_struct_with_parent_ref() {
        var l1 = new Level1Struct();
        l1.setData(10);
        l1.setParentRef(new TopLevelStruct());
        assertEquals(10, l1.getData());
        assertNotNull(l1.getParentRef());
    }

    @Test
    void level2_struct_with_refs() {
        var l2 = new Level2Struct();
        l2.setName("test");
        l2.setLevel1Ref(new Level1Struct());
        l2.setTopRef(new TopLevelStruct());
        assertEquals("test", l2.getName());
        assertNotNull(l2.getLevel1Ref());
        assertNotNull(l2.getTopRef());
    }

    @Test
    void level3_struct_with_all_refs() {
        var l3 = new Level3Struct();
        l3.setId(30);
        l3.setLevel2Ref(new Level2Struct());
        l3.setLevel1Ref(new Level1Struct());
        l3.setTopRef(new TopLevelStruct());
        assertEquals(30, l3.getId());
        assertNotNull(l3.getLevel2Ref());
        assertNotNull(l3.getLevel1Ref());
        assertNotNull(l3.getTopRef());
    }

    @Test
    void deep_constant() {
        assertEquals(42, DEEP_CONST.value);
    }

    @Test
    void sibling_cross_ref_struct() {
        var c = new CrossRef();
        c.setFromLevel1(new Level1Struct());
        c.setFromLevel2(new Level2Struct());
        c.setFromLevel3(new Level3Struct());
        assertNotNull(c.getFromLevel1());
        assertNotNull(c.getFromLevel2());
        assertNotNull(c.getFromLevel3());
    }

    @Test
    void top_using_nested_struct() {
        var t = new TopUsingNested();
        t.setL1(new Level1Struct());
        t.setL2(new Level2Struct());
        t.setL3(new Level3Struct());
        assertNotNull(t.getL1());
        assertNotNull(t.getL2());
        assertNotNull(t.getL3());
    }

    @Test
    void level1_enum() {
        assertEquals(0, Level1Enum.A.getValue());
        assertEquals(1, Level1Enum.B.getValue());
        assertEquals(2, Level1Enum.C.getValue());
    }
}
