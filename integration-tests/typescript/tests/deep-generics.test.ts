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

import { describe, expect, test } from "bun:test";
import type {
  TwoLevelSeq,
  ThreeLevelSeq,
  MapOfSeq,
  SeqOfMap,
  MapOfMap,
  MapSeqMap,
  SeqMapSeq,
  FourLevelDeep,
  SeqOfPoints,
  MapOfPoints,
  SeqOfSeqOfPoints,
  MapOfSeqOfPoints,
  UsingTypedefChain,
  ArrayOfSeq,
  SeqOfArray,
  MapOfArray,
} from "../generated/deepGenericTypes";

describe("deep generics", () => {
  describe("nested sequences", () => {
    test("TwoLevelSeq - matrix", () => {
      const obj: TwoLevelSeq = {
        matrix: [
          [1, 2, 3],
          [4, 5, 6],
        ],
      };
      expect(obj.matrix[0][0]).toBe(1);
      expect(obj.matrix[1][2]).toBe(6);
    });

    test("ThreeLevelSeq - cube", () => {
      const obj: ThreeLevelSeq = {
        cube: [
          [
            [1, 2],
            [3, 4],
          ],
          [
            [5, 6],
            [7, 8],
          ],
        ],
      };
      expect(obj.cube[0][0][0]).toBe(1);
      expect(obj.cube[1][1][1]).toBe(8);
    });

    test("FourLevelDeep - hypercube", () => {
      const obj: FourLevelDeep = {
        hypercube: [[[[1, 2]], [[3, 4]]]],
      };
      expect(obj.hypercube[0][0][0][0]).toBe(1);
    });
  });

  describe("map/sequence combinations", () => {
    test("MapOfSeq", () => {
      const obj: MapOfSeq = {
        indexedLists: {
          first: [1, 2, 3],
          second: [4, 5, 6],
        },
      };
      expect(obj.indexedLists.first).toEqual([1, 2, 3]);
    });

    test("SeqOfMap", () => {
      const obj: SeqOfMap = {
        listOfDicts: [
          { a: 1, b: 2 },
          { c: 3, d: 4 },
        ],
      };
      expect(obj.listOfDicts[0].a).toBe(1);
    });

    test("MapOfMap", () => {
      const obj: MapOfMap = {
        nestedDict: {
          outer: { inner: 42 },
        },
      };
      expect(obj.nestedDict.outer.inner).toBe(42);
    });

    test("MapSeqMap - complex structure", () => {
      const obj: MapSeqMap = {
        complexStructure: {
          key: [{ a: 1 }, { b: 2 }],
        },
      };
      expect(obj.complexStructure.key[0].a).toBe(1);
    });

    test("SeqMapSeq - inverse structure", () => {
      const obj: SeqMapSeq = {
        inverseStructure: [{ nums: [1, 2, 3] }],
      };
      expect(obj.inverseStructure[0].nums).toEqual([1, 2, 3]);
    });
  });

  describe("struct collections", () => {
    test("SeqOfPoints", () => {
      const obj: SeqOfPoints = {
        points: [
          { x: 0, y: 0 },
          { x: 1, y: 1 },
        ],
      };
      expect(obj.points[0].x).toBe(0);
      expect(obj.points[1].y).toBe(1);
    });

    test("MapOfPoints", () => {
      const obj: MapOfPoints = {
        namedPoints: {
          origin: { x: 0, y: 0 },
          target: { x: 10, y: 20 },
        },
      };
      expect(obj.namedPoints.origin.x).toBe(0);
      expect(obj.namedPoints.target.y).toBe(20);
    });

    test("SeqOfSeqOfPoints - point matrix", () => {
      const obj: SeqOfSeqOfPoints = {
        pointMatrix: [
          [
            { x: 0, y: 0 },
            { x: 1, y: 0 },
          ],
          [
            { x: 0, y: 1 },
            { x: 1, y: 1 },
          ],
        ],
      };
      expect(obj.pointMatrix[1][1].x).toBe(1);
      expect(obj.pointMatrix[1][1].y).toBe(1);
    });

    test("MapOfSeqOfPoints", () => {
      const obj: MapOfSeqOfPoints = {
        pointLists: {
          line: [
            { x: 0, y: 0 },
            { x: 10, y: 10 },
          ],
        },
      };
      expect(obj.pointLists.line.length).toBe(2);
    });
  });

  describe("typedef chains", () => {
    test("UsingTypedefChain", () => {
      const obj: UsingTypedefChain = {
        data: {
          matrix1: [
            [1, 2],
            [3, 4],
          ],
        },
      };
      expect(obj.data.matrix1[0][0]).toBe(1);
    });
  });

  describe("array of sequences", () => {
    test("ArrayOfSeq", () => {
      const obj: ArrayOfSeq = {
        items: [
          [1, 2, 3],
          [4, 5, 6],
          [7, 8, 9],
        ],
      };
      expect(obj.items.length).toBe(3);
      expect(obj.items[2]).toEqual([7, 8, 9]);
    });

    test("SeqOfArray", () => {
      const obj: SeqOfArray = {
        fixedTriples: [
          [1, 2, 3],
          [4, 5, 6],
        ],
      };
      expect(obj.fixedTriples[0]).toEqual([1, 2, 3]);
    });

    test("MapOfArray", () => {
      const obj: MapOfArray = {
        namedTriples: {
          first: [1, 2, 3],
          second: [4, 5, 6],
        },
      };
      expect(obj.namedTriples.first).toEqual([1, 2, 3]);
    });
  });
});
