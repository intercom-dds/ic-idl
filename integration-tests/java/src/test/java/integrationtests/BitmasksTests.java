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

import bitmask_types.*;
import org.junit.jupiter.api.Test;

class BitmasksTests {

    @Test
    void permissions_values() {
        assertEquals(1, Permissions.READ);
        assertEquals(2, Permissions.WRITE);
        assertEquals(4, Permissions.EXECUTE);
        assertEquals(8, Permissions.DELETE);
    }

    @Test
    void permissions_arePowersOfTwo() {
        assertEquals(1, Integer.bitCount(Permissions.READ));
        assertEquals(1, Integer.bitCount(Permissions.WRITE));
        assertEquals(1, Integer.bitCount(Permissions.EXECUTE));
        assertEquals(1, Integer.bitCount(Permissions.DELETE));
    }

    @Test
    void permissions_canCombineWithOr() {
        int readWrite = Permissions.READ | Permissions.WRITE;
        assertEquals(3, readWrite);
    }

    @Test
    void permissions_canCheckWithAnd() {
        int perms = Permissions.READ | Permissions.EXECUTE;
        assertTrue((perms & Permissions.READ) != 0);
        assertTrue((perms & Permissions.EXECUTE) != 0);
        assertFalse((perms & Permissions.WRITE) != 0);
        assertFalse((perms & Permissions.DELETE) != 0);
    }

    @Test
    void permissions_allCombined() {
        int all = Permissions.READ | Permissions.WRITE | Permissions.EXECUTE | Permissions.DELETE;
        assertEquals(15, all);
    }

    @Test
    void explicitFlags_values() {
        assertEquals(2, ExplicitFlags.FLAG_A);
        assertEquals(4, ExplicitFlags.FLAG_B);
        assertEquals(16, ExplicitFlags.FLAG_C);
        assertEquals(256, ExplicitFlags.FLAG_D);
    }

    @Test
    void explicitFlags_arePowersOfTwo() {
        assertEquals(1, Integer.bitCount(ExplicitFlags.FLAG_A));
        assertEquals(1, Integer.bitCount(ExplicitFlags.FLAG_B));
        assertEquals(1, Integer.bitCount(ExplicitFlags.FLAG_C));
        assertEquals(1, Integer.bitCount(ExplicitFlags.FLAG_D));
    }

    @Test
    void mixedFlags_values() {
        assertEquals(1, MixedFlags.AUTO_FIRST);
        assertEquals(16, MixedFlags.EXPLICIT_FOUR);
        assertEquals(32, MixedFlags.AUTO_FIVE);
        assertEquals(64, MixedFlags.AUTO_SIX);
    }

    @Test
    void mixedFlags_arePowersOfTwo() {
        assertEquals(1, Integer.bitCount(MixedFlags.AUTO_FIRST));
        assertEquals(1, Integer.bitCount(MixedFlags.EXPLICIT_FOUR));
        assertEquals(1, Integer.bitCount(MixedFlags.AUTO_FIVE));
        assertEquals(1, Integer.bitCount(MixedFlags.AUTO_SIX));
    }

    @Test
    void gappedFlags_values() {
        assertEquals(1, GappedFlags.LOW);
        assertEquals(128, GappedFlags.HIGH);
    }

    @Test
    void gappedFlags_canCombine() {
        int both = GappedFlags.LOW | GappedFlags.HIGH;
        assertEquals(129, both);
    }

    @Test
    void singleFlag_value() {
        assertEquals(1, SingleFlag.ONLY);
    }

    @Test
    void fileInfo_defaults() {
        var f = new FileInfo();
        assertEquals("", f.getPath());
        assertNotNull(f.getPerms());
        assertTrue(f.getPerms().isEmpty());
    }

    @Test
    void fileInfo_parameterizedConstructor() {
        var perms = new java.util.BitSet();
        perms.set(0);
        perms.set(2);
        var f = new FileInfo("/tmp/test", perms);
        assertEquals("/tmp/test", f.getPath());
        assertTrue(f.getPerms().get(0));
        assertFalse(f.getPerms().get(1));
        assertTrue(f.getPerms().get(2));
    }

    @Test
    void fileInfo_setters() {
        var f = new FileInfo();
        f.setPath("/home/user");
        var perms = new java.util.BitSet();
        perms.set(1);
        f.setPerms(perms);
        assertEquals("/home/user", f.getPath());
        assertTrue(f.getPerms().get(1));
    }

    @Test
    void fileInfo_copyConstructor() {
        var perms = new java.util.BitSet();
        perms.set(3);
        var f1 = new FileInfo("/path", perms);
        var f2 = new FileInfo(f1);
        assertEquals(f1.getPath(), f2.getPath());
        assertEquals(f1.getPerms(), f2.getPerms());
        assertNotSame(f1, f2);
        assertNotSame(f1.getPerms(), f2.getPerms());
        f2.getPerms().set(5);
        assertTrue(f2.getPerms().get(5));
        assertFalse(f1.getPerms().get(5));
    }

    @Test
    void fileInfo_equality() {
        var perms1 = new java.util.BitSet();
        perms1.set(0);
        var f1 = new FileInfo("/same", perms1);
        var perms2 = new java.util.BitSet();
        perms2.set(0);
        var f2 = new FileInfo("/same", perms2);
        var perms3 = new java.util.BitSet();
        perms3.set(1);
        var f3 = new FileInfo("/same", perms3);
        assertEquals(f1, f2);
        assertNotEquals(f1, f3);
    }

    @Test
    void fileInfo_clone() {
        var perms = new java.util.BitSet();
        perms.set(2);
        var f1 = new FileInfo("/clone", perms);
        var f2 = f1.clone();
        assertEquals(f1, f2);
        assertNotSame(f1, f2);
        assertNotSame(f1.getPerms(), f2.getPerms());
        f2.getPerms().clear(2);
        assertFalse(f2.getPerms().get(2));
        assertTrue(f1.getPerms().get(2));
    }
}
