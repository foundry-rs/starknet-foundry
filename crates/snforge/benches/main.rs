use criterion::{criterion_group, criterion_main};

mod forge_bench;

criterion_group!(benches, forge_bench::benchmark::criterion_benchmark);
criterion_main!(benches);
