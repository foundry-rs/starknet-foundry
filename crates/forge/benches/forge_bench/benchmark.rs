use crate::forge_bench::{collect_tests, compile_tests, declare_deploy_and_interact};
use criterion::{BenchmarkId, Criterion, SamplingMode};
use std::time::Duration;

#[allow(clippy::module_name_repetitions)]
pub fn criterion_benchmark(c: &mut Criterion) {
    let declare_and_interact_input = declare_deploy_and_interact::setup();
    let collect_tests_input = collect_tests::setup();
    let compile_tests_input = compile_tests::setup();

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
    group.bench_with_input(
        BenchmarkId::new("collect_tests", format!("{collect_tests_input:?}")),
        &collect_tests_input,
        |b, package| b.iter(|| collect_tests::collect_tests(package)),
    );
    group.bench_with_input(
        BenchmarkId::new("compile_tests", format!("{compile_tests_input:?}")),
        &compile_tests_input,
        |b, (compilation_target, linked_libraries, corelib_path, _)| {
            b.iter(|| {
                compile_tests::compile_tests(compilation_target, linked_libraries, corelib_path);
            });
        },
    );
    group.finish();
}
