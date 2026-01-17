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

import deep_generic_types.*;
import java.util.*;
import org.junit.jupiter.api.Test;

class DeepGenericsTests {

    @Test
    void two_level_seq() {
        var t = new TwoLevelSeq();
        t.getMatrix().add(Arrays.asList(1, 2, 3));
        t.getMatrix().add(Arrays.asList(4, 5, 6));
        assertEquals(2, t.getMatrix().size());
        assertEquals(3, t.getMatrix().get(0).size());
    }

    @Test
    void three_level_seq() {
        var t = new ThreeLevelSeq();
        var inner = new ArrayList<List<Integer>>();
        inner.add(Arrays.asList(1, 2));
        t.getCube().add(inner);
        assertEquals(1, t.getCube().size());
    }

    @Test
    void four_level_deep() {
        var f = new FourLevelDeep();
        assertNotNull(f.getHypercube());
    }

    @Test
    void map_of_seq() {
        var m = new MapOfSeq();
        m.getIndexedLists().put("row1", Arrays.asList(1, 2, 3));
        m.getIndexedLists().put("row2", Arrays.asList(4, 5, 6));
        assertEquals(2, m.getIndexedLists().size());
        assertEquals(3, m.getIndexedLists().get("row1").size());
    }

    @Test
    void seq_of_map() {
        var s = new SeqOfMap();
        var map = new HashMap<String, Integer>();
        map.put("a", 1);
        s.getListOfDicts().add(map);
        assertEquals(1, s.getListOfDicts().size());
    }

    @Test
    void map_of_map() {
        var m = new MapOfMap();
        var inner = new HashMap<String, Integer>();
        inner.put("x", 10);
        m.getNestedDict().put("outer", inner);
        assertEquals(10, m.getNestedDict().get("outer").get("x"));
    }

    @Test
    void map_seq_map() {
        var m = new MapSeqMap();
        assertNotNull(m.getComplexStructure());
    }

    @Test
    void seq_map_seq() {
        var s = new SeqMapSeq();
        assertNotNull(s.getInverseStructure());
    }

    @Test
    void point_struct() {
        var p = new Point();
        p.setX(10);
        p.setY(20);
        assertEquals(10, p.getX());
        assertEquals(20, p.getY());
    }

    @Test
    void seq_of_points() {
        var s = new SeqOfPoints();
        var p = new Point();
        p.setX(1);
        p.setY(2);
        s.getPoints().add(p);
        assertEquals(1, s.getPoints().size());
        assertEquals(1, s.getPoints().get(0).getX());
    }

    @Test
    void map_of_points() {
        var m = new MapOfPoints();
        var p = new Point();
        p.setX(5);
        p.setY(10);
        m.getNamedPoints().put("origin", p);
        assertEquals(5, m.getNamedPoints().get("origin").getX());
    }

    @Test
    void seq_of_seq_of_points() {
        var s = new SeqOfSeqOfPoints();
        var inner = new ArrayList<Point>();
        var p = new Point();
        p.setX(1);
        p.setY(2);
        inner.add(p);
        s.getPointMatrix().add(inner);
        assertEquals(1, s.getPointMatrix().size());
    }

    @Test
    void map_of_seq_of_points() {
        var m = new MapOfSeqOfPoints();
        var points = new ArrayList<Point>();
        var p = new Point();
        p.setX(3);
        p.setY(4);
        points.add(p);
        m.getPointLists().put("list1", points);
        assertEquals(1, m.getPointLists().get("list1").size());
    }

    @Test
    void typedef_aliases_exist() {
        // IntList, IntMatrix, NamedMatrices are typedefs - resolved to base types
        var u = new UsingTypedefChain();
        assertNotNull(u.getData());
    }

    @Test
    void using_typedef_chain() {
        var u = new UsingTypedefChain();
        var matrix = new ArrayList<List<Integer>>();
        var row = new ArrayList<Integer>();
        row.add(1);
        matrix.add(row);
        u.getData().put("key", matrix);
        assertEquals(1, u.getData().size());
    }

    @Test
    void array_of_seq() {
        var a = new ArrayOfSeq();
        assertNotNull(a.getItems());
        assertEquals(3, a.getItems().length);
    }

    @Test
    void seq_of_array() {
        var s = new SeqOfArray();
        assertNotNull(s.getFixedTriples());
    }

    @Test
    void map_of_array() {
        var m = new MapOfArray();
        assertNotNull(m.getNamedTriples());
    }
}
