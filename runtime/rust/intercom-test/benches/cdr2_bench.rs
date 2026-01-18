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

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use intercom_cts::cdr2::{from_le_bytes, to_le_bytes};
use intercom_test::cdr2::mutable::{Array2d, Array3d, PrimitiveArray};

fn bench_serialize_primitive_struct_final(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveStruct;
    let data = PrimitiveStruct::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_serialize_primitive_struct");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("final", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_primitive_struct_final(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveStruct;
    let data = PrimitiveStruct::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_deserialize_primitive_struct");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("final", |b| {
        b.iter(|| {
            let result: PrimitiveStruct = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_primitive_struct_appendable(c: &mut Criterion) {
    use intercom_test::cdr2::appendable::PrimitiveStruct;
    let data = PrimitiveStruct::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_serialize_primitive_struct");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("appendable", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_primitive_struct_appendable(c: &mut Criterion) {
    use intercom_test::cdr2::appendable::PrimitiveStruct;
    let data = PrimitiveStruct::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_deserialize_primitive_struct");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("appendable", |b| {
        b.iter(|| {
            let result: PrimitiveStruct = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_primitive_struct_mutable(c: &mut Criterion) {
    use intercom_test::cdr2::mutable::PrimitiveStruct;
    let data = PrimitiveStruct::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_serialize_primitive_struct");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("mutable", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_primitive_struct_mutable(c: &mut Criterion) {
    use intercom_test::cdr2::mutable::PrimitiveStruct;
    let data = PrimitiveStruct::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_deserialize_primitive_struct");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("mutable", |b| {
        b.iter(|| {
            let result: PrimitiveStruct = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_primitive_array(c: &mut Criterion) {
    let data = PrimitiveArray::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_serialize_primitive_array");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("mutable", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_primitive_array(c: &mut Criterion) {
    let data = PrimitiveArray::new();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_deserialize_primitive_array");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("mutable", |b| {
        b.iter(|| {
            let result: PrimitiveArray = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_vec_i32_small(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveSeqType;
    let data = PrimitiveSeqType {
        inner: (0..10).collect(),
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_serialize_vec_i32");
    group.throughput(Throughput::Bytes(bytes.len() as u64));

    group.bench_function("10_elements", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_vec_i32_medium(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveSeqType;
    let data = PrimitiveSeqType {
        inner: (0..100).collect(),
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_serialize_vec_i32");
    group.throughput(Throughput::Bytes(bytes.len() as u64));

    group.bench_function("100_elements", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_vec_i32_large(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveSeqType;
    let data = PrimitiveSeqType {
        inner: (0..1000).collect(),
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_serialize_vec_i32");
    group.throughput(Throughput::Bytes(bytes.len() as u64));

    group.bench_function("1000_elements", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_vec_i32_small(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveSeqType;
    let data = PrimitiveSeqType {
        inner: (0..10).collect(),
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_deserialize_vec_i32");
    group.throughput(Throughput::Bytes((data.inner.len() * 4) as u64));

    group.bench_function("10_elements", |b| {
        b.iter(|| {
            let result: PrimitiveSeqType = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_vec_i32_medium(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveSeqType;
    let data = PrimitiveSeqType {
        inner: (0..100).collect(),
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_deserialize_vec_i32");
    group.throughput(Throughput::Bytes((data.inner.len() * 4) as u64));

    group.bench_function("100_elements", |b| {
        b.iter(|| {
            let result: PrimitiveSeqType = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_vec_i32_large(c: &mut Criterion) {
    use intercom_test::cdr2::final_::PrimitiveSeqType;
    let data = PrimitiveSeqType {
        inner: (0..1000).collect(),
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_deserialize_vec_i32");
    group.throughput(Throughput::Bytes((data.inner.len() * 4) as u64));

    group.bench_function("1000_elements", |b| {
        b.iter(|| {
            let result: PrimitiveSeqType = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_string(c: &mut Criterion) {
    use intercom_test::cdr2::final_::SeqType;
    let data = SeqType {
        inner: vec!["Hello, World! This is a test string.".to_string()],
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_serialize_string");
    group.throughput(Throughput::Bytes(bytes.len() as u64));

    group.bench_function("short_string", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_string(c: &mut Criterion) {
    use intercom_test::cdr2::final_::SeqType;
    let data = SeqType {
        inner: vec!["Hello, World! This is a test string.".to_string()],
    };
    let bytes = to_le_bytes(&data).unwrap();
    let mut group = c.benchmark_group("cdr2_deserialize_string");
    group.throughput(Throughput::Bytes(data.inner[0].len() as u64));

    group.bench_function("short_string", |b| {
        b.iter(|| {
            let result: SeqType = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_array_2d(c: &mut Criterion) {
    let data = Array2d::default();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_serialize_array_2d");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("10x10_i32", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_array_2d(c: &mut Criterion) {
    let data = Array2d::default();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_deserialize_array_2d");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("10x10_i32", |b| {
        b.iter(|| {
            let result: Array2d = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_serialize_array_3d(c: &mut Criterion) {
    let data = Array3d::default();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_serialize_array_3d");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("4x4x4_u8", |b| {
        b.iter(|| {
            let result = to_le_bytes(black_box(&data)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

fn bench_deserialize_array_3d(c: &mut Criterion) {
    let data = Array3d::default();
    let bytes = to_le_bytes(&data).unwrap();

    let mut group = c.benchmark_group("cdr2_deserialize_array_3d");
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("4x4x4_u8", |b| {
        b.iter(|| {
            let result: Array3d = from_le_bytes(black_box(&bytes)).unwrap();
            black_box(result);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_serialize_primitive_struct_final,
    bench_deserialize_primitive_struct_final,
    bench_serialize_primitive_struct_appendable,
    bench_deserialize_primitive_struct_appendable,
    bench_serialize_primitive_struct_mutable,
    bench_deserialize_primitive_struct_mutable,
    bench_serialize_primitive_array,
    bench_deserialize_primitive_array,
    bench_serialize_array_2d,
    bench_deserialize_array_2d,
    bench_serialize_array_3d,
    bench_deserialize_array_3d,
    bench_serialize_vec_i32_small,
    bench_serialize_vec_i32_medium,
    bench_serialize_vec_i32_large,
    bench_deserialize_vec_i32_small,
    bench_deserialize_vec_i32_medium,
    bench_deserialize_vec_i32_large,
    bench_serialize_string,
    bench_deserialize_string,
);
criterion_main!(benches);
