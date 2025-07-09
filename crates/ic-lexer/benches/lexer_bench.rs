// Benchmark for lexer performance
// Run with: cargo bench

#![feature(test)]
extern crate test;

use ic_lexer::cursor::Cursor;
use ic_vfs::SourceMap;
use std::hint::black_box;
use test::Bencher;

const SAMPLE_IDL: &str = r#"
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
"#;

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
    let keywords = "module struct interface enum typedef const public private readonly attribute in out inout";
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