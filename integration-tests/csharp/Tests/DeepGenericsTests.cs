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

using Xunit;
using DeepGenericTypes;

namespace IntegrationTests;

public class DeepGenericsTests
{
    [Fact]
    public void TwoLevelSeq_EmptyByDefault()
    {
        var t = new TwoLevelSeq();
        Assert.Empty(t.Matrix);
    }

    [Fact]
    public void TwoLevelSeq_CanAddNestedLists()
    {
        var t = new TwoLevelSeq();
        t.Matrix.Add(new List<int> { 1, 2, 3 });
        t.Matrix.Add(new List<int> { 4, 5, 6 });

        Assert.Equal(2, t.Matrix.Count);
        Assert.Equal(new[] { 1, 2, 3 }, t.Matrix[0]);
        Assert.Equal(new[] { 4, 5, 6 }, t.Matrix[1]);
    }

    [Fact]
    public void ThreeLevelSeq_EmptyByDefault()
    {
        var t = new ThreeLevelSeq();
        Assert.Empty(t.Cube);
    }

    [Fact]
    public void ThreeLevelSeq_CanAddNestedLists()
    {
        var t = new ThreeLevelSeq();
        var innerList = new List<IList<int>> { new List<int> { 1, 2 }, new List<int> { 3, 4 } };
        t.Cube.Add(innerList);

        Assert.Single(t.Cube);
        Assert.Equal(2, t.Cube[0].Count);
    }

    [Fact]
    public void FourLevelDeep_EmptyByDefault()
    {
        var f = new FourLevelDeep();
        Assert.Empty(f.Hypercube);
    }

    [Fact]
    public void FourLevelDeep_CanAddNestedLists()
    {
        var f = new FourLevelDeep();
        var level3 = new List<IList<int>> { new List<int> { 1 } };
        var level2 = new List<IList<IList<int>>> { level3 };
        f.Hypercube.Add(level2);

        Assert.Single(f.Hypercube);
    }

    [Fact]
    public void MapOfSeq_EmptyByDefault()
    {
        var m = new MapOfSeq();
        Assert.Empty(m.IndexedLists);
    }

    [Fact]
    public void MapOfSeq_CanAddEntries()
    {
        var m = new MapOfSeq();
        m.IndexedLists["first"] = new List<int> { 1, 2, 3 };
        m.IndexedLists["second"] = new List<int> { 4, 5 };

        Assert.Equal(2, m.IndexedLists.Count);
        Assert.Equal(new[] { 1, 2, 3 }, m.IndexedLists["first"]);
    }

    [Fact]
    public void SeqOfMap_EmptyByDefault()
    {
        var s = new SeqOfMap();
        Assert.Empty(s.ListOfDicts);
    }

    [Fact]
    public void SeqOfMap_CanAddDicts()
    {
        var s = new SeqOfMap();
        s.ListOfDicts.Add(new Dictionary<string, int> { { "a", 1 }, { "b", 2 } });

        Assert.Single(s.ListOfDicts);
        Assert.Equal(1, s.ListOfDicts[0]["a"]);
    }

    [Fact]
    public void MapOfMap_EmptyByDefault()
    {
        var m = new MapOfMap();
        Assert.Empty(m.NestedDict);
    }

    [Fact]
    public void MapOfMap_CanAddEntries()
    {
        var m = new MapOfMap();
        m.NestedDict["outer"] = new Dictionary<string, int> { { "inner", 42 } };

        Assert.Single(m.NestedDict);
        Assert.Equal(42, m.NestedDict["outer"]["inner"]);
    }

    [Fact]
    public void MapSeqMap_EmptyByDefault()
    {
        var m = new MapSeqMap();
        Assert.Empty(m.ComplexStructure);
    }

    [Fact]
    public void MapSeqMap_CanAddComplexData()
    {
        var m = new MapSeqMap();
        var innerDict = new Dictionary<string, int> { { "x", 1 } };
        var innerList = new List<IDictionary<string, int>> { innerDict };
        m.ComplexStructure["key"] = innerList;

        Assert.Single(m.ComplexStructure);
        Assert.Equal(1, m.ComplexStructure["key"][0]["x"]);
    }

    [Fact]
    public void SeqMapSeq_EmptyByDefault()
    {
        var s = new SeqMapSeq();
        Assert.Empty(s.InverseStructure);
    }

    [Fact]
    public void Point_Instantiation()
    {
        var p = new Point(10, 20);
        Assert.Equal(10, p.X);
        Assert.Equal(20, p.Y);
    }

    [Fact]
    public void SeqOfPoints_EmptyByDefault()
    {
        var s = new SeqOfPoints();
        Assert.Empty(s.Points);
    }

    [Fact]
    public void SeqOfPoints_CanAddPoints()
    {
        var s = new SeqOfPoints();
        s.Points.Add(new Point(1, 2));
        s.Points.Add(new Point(3, 4));

        Assert.Equal(2, s.Points.Count);
        Assert.Equal(1, s.Points[0].X);
        Assert.Equal(4, s.Points[1].Y);
    }

    [Fact]
    public void MapOfPoints_EmptyByDefault()
    {
        var m = new MapOfPoints();
        Assert.Empty(m.NamedPoints);
    }

    [Fact]
    public void MapOfPoints_CanAddPoints()
    {
        var m = new MapOfPoints();
        m.NamedPoints["origin"] = new Point(0, 0);
        m.NamedPoints["offset"] = new Point(10, 20);

        Assert.Equal(2, m.NamedPoints.Count);
        Assert.Equal(0, m.NamedPoints["origin"].X);
        Assert.Equal(20, m.NamedPoints["offset"].Y);
    }

    [Fact]
    public void SeqOfSeqOfPoints_EmptyByDefault()
    {
        var s = new SeqOfSeqOfPoints();
        Assert.Empty(s.PointMatrix);
    }

    [Fact]
    public void SeqOfSeqOfPoints_CanAddPointLists()
    {
        var s = new SeqOfSeqOfPoints();
        s.PointMatrix.Add(new List<Point> { new Point(1, 1), new Point(2, 2) });

        Assert.Single(s.PointMatrix);
        Assert.Equal(2, s.PointMatrix[0].Count);
    }

    [Fact]
    public void MapOfSeqOfPoints_EmptyByDefault()
    {
        var m = new MapOfSeqOfPoints();
        Assert.Empty(m.PointLists);
    }

    [Fact]
    public void MapOfSeqOfPoints_CanAddPointLists()
    {
        var m = new MapOfSeqOfPoints();
        m.PointLists["line1"] = new List<Point> { new Point(0, 0), new Point(10, 10) };

        Assert.Single(m.PointLists);
        Assert.Equal(2, m.PointLists["line1"].Count);
    }

    [Fact]
    public void UsingTypedefChain_EmptyByDefault()
    {
        var u = new UsingTypedefChain();
        Assert.Empty(u.Data);
    }

    [Fact]
    public void ArrayOfSeq_HasFixedSize()
    {
        var a = new ArrayOfSeq();
        Assert.Equal(3, a.Items.Length);
    }

    [Fact]
    public void ArrayOfSeq_CanSetLists()
    {
        var items = new IList<int>[] {
            new List<int> { 1, 2 },
            new List<int> { 3, 4 },
            new List<int> { 5, 6 }
        };
        var a = new ArrayOfSeq(items);

        Assert.Equal(new[] { 1, 2 }, a.Items[0]);
        Assert.Equal(new[] { 5, 6 }, a.Items[2]);
    }

    [Fact]
    public void SeqOfArray_EmptyByDefault()
    {
        var s = new SeqOfArray();
        Assert.Empty(s.FixedTriples);
    }

    [Fact]
    public void SeqOfArray_CanAddArrays()
    {
        var s = new SeqOfArray();
        s.FixedTriples.Add(new int[] { 1, 2, 3 });
        s.FixedTriples.Add(new int[] { 4, 5, 6 });

        Assert.Equal(2, s.FixedTriples.Count);
        Assert.Equal(new[] { 1, 2, 3 }, s.FixedTriples[0]);
    }

    [Fact]
    public void MapOfArray_EmptyByDefault()
    {
        var m = new MapOfArray();
        Assert.Empty(m.NamedTriples);
    }

    [Fact]
    public void MapOfArray_CanAddArrays()
    {
        var m = new MapOfArray();
        m.NamedTriples["rgb"] = new int[] { 255, 128, 0 };

        Assert.Single(m.NamedTriples);
        Assert.Equal(new[] { 255, 128, 0 }, m.NamedTriples["rgb"]);
    }
}
