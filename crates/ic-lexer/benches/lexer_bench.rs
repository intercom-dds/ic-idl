// Copyright 2024 KONGSBERG
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
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![feature(test)]
extern crate test;

use std::hint::black_box;

use ic_lexer::cursor::Cursor;
use ic_vfs::SourceMap;
use test::Bencher;

const SAMPLE_IDL: &str = r"
module Example {
    // This is a comment
    struct Point {
        float x;
        float y;
        float z;
    };
    
    interface Calculator {
        float add(float a, float b);
        float multiply(float a, float b);
        sequence<Point> generatePoints(long count);
    };
    
    const float PI = 3.14159265359;
    const long MAX_POINTS = 1000000;
    
    enum Status {
        OK,
        ERROR,
        PENDING
    };
    
    typedef sequence<Point> PointList;
    typedef map<string, Point> PointMap;
}
";

#[bench]
fn bench_tokenize_sample(b: &mut Bencher) {
    let mut vfs = SourceMap::default();
    let id = vfs.embed(SAMPLE_IDL);
    let src = vfs.source(id);

    b.iter(|| {
        let mut cursor = Cursor::new(src.clone(), id);
        let mut tokens = Vec::with_capacity(100);
        while let Some(token) = cursor.next() {
            tokens.push(black_box(token));
        }
        black_box(tokens)
    });
}

#[bench]
fn bench_tokenize_keywords(b: &mut Bencher) {
    let keywords =
        "module struct interface enum typedef const public private readonly attribute in out inout";
    let mut vfs = SourceMap::default();
    let id = vfs.embed(keywords);
    let src = vfs.source(id);

    b.iter(|| {
        let mut cursor = Cursor::new(src.clone(), id);
        let mut count = 0;
        while let Some(token) = cursor.next() {
            count += 1;
            black_box(token);
        }
        black_box(count)
    });
}

#[bench]
fn bench_tokenize_numbers(b: &mut Bencher) {
    let numbers = "123 456 0xFF 0777 3.14 2.71828 1e10 1.5e-10";
    let mut vfs = SourceMap::default();
    let id = vfs.embed(numbers);
    let src = vfs.source(id);

    b.iter(|| {
        let mut cursor = Cursor::new(src.clone(), id);
        let mut count = 0;
        while let Some(token) = cursor.next() {
            count += 1;
            black_box(token);
        }
        black_box(count)
    });
}

#[bench]
fn bench_tokenize_strings(b: &mut Bencher) {
    let strings = r#""hello" "world" "foo\nbar" "escaped\"quote" 'a' 'b' '\n'"#;
    let mut vfs = SourceMap::default();
    let id = vfs.embed(strings);
    let src = vfs.source(id);

    b.iter(|| {
        let mut cursor = Cursor::new(src.clone(), id);
        let mut count = 0;
        while let Some(token) = cursor.next() {
            count += 1;
            black_box(token);
        }
        black_box(count)
    });
}

#[bench]
fn bench_tokenize_operators(b: &mut Bencher) {
    let operators = "+ - * / % < > <= >= == != & | ^ ~ && || = : :: ; , . ( ) [ ] { }";
    let mut vfs = SourceMap::default();
    let id = vfs.embed(operators);
    let src = vfs.source(id);

    b.iter(|| {
        let mut cursor = Cursor::new(src.clone(), id);
        let mut count = 0;
        while let Some(token) = cursor.next() {
            count += 1;
            black_box(token);
        }
        black_box(count)
    });
}
