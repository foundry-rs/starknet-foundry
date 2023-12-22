use crate::forge_bench::declare_deploy_and_interact;
use criterion::{BenchmarkId, Criterion, SamplingMode};
use std::time::Duration;

#[allow(clippy::module_name_repetitions)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let declare_and_interact_input = declare_deploy_and_interact::setup();

    let mut group = c.benchmark_group("benchmark-normal-flow");

    // Needed because our benchmark is long-running
    // https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#sampling-mode
    group.sampling_mode(SamplingMode::Flat);

    group.sample_size(50);
    group.measurement_time(Duration::from_secs(120));
    group.bench_with_input(
        BenchmarkId::new(
            "declare_deploy_and_interact",
            format!("{declare_and_interact_input:?}"),
        ),
        &declare_and_interact_input,
        |b, test_case| {
            b.iter(|| declare_deploy_and_interact::declare_deploy_and_interact(test_case));
        },
    );

    group.finish();
}
