use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::fs;

fn decode_qoi(input: &[Vec<u8>]) {
    for data in input.iter() {
        let output = qoi::decode_to_vec(&data).unwrap();
    }
}

fn decode_rapid_qoi(input: &[Vec<u8>]) {
    for data in input.iter() {
        let output = rapid_qoi::Qoi::decode_alloc(&data).unwrap();
    }
}

fn decode_qoi_qoi(input: &[Vec<u8>]) {
    for data in input.iter() {
        let output = qoi_qoi::decode(&data).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let input = [
        "dice",
        "kodim10",
        "kodim23",
        "qoi_logo",
        "testcard",
        "testcard_rgba",
        "wikipedia_008",
    ]
    .map(|name| fs::read(format!("images/input/{}.qoi", name)).unwrap());

    let mut group = c.benchmark_group("Qui decode");
    group.bench_function("qoi", |b| b.iter(|| decode_qoi(black_box(&input))));
    group.bench_function("rapid-qoi", |b| {
        b.iter(|| decode_rapid_qoi(black_box(&input)))
    });
    group.bench_function("qoi-qoi", |b| b.iter(|| decode_qoi_qoi(black_box(&input))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
