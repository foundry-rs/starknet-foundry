use criterion::{criterion_group, criterion_main};

mod my_bench;

criterion_group!(benches, my_bench::benchmark::criterion_benchmark);
criterion_main!(benches);
