use crate::fuzzer::arguments::CairoType;
use anyhow::Result;
use rand::rngs::StdRng;
use rand::Rng;

mod arguments;
mod random;

#[allow(clippy::module_name_repetitions)]
pub use random::RandomFuzzer;

#[derive(Debug, Clone)]
pub struct RunParams {
    /// Arguments
    arguments: Vec<CairoType>,
    /// Total number of runs
    total_runs: u32,
    /// Number of already executed runs
    executed_runs: u32,
    /// Run in which an argument has a min value
    /// e.g. `run_with_min_value_argument[0] = 5`
    /// means that the first argument will have the lowest possible value in 5th run
    run_with_min_value_for_argument: Vec<u32>,
    /// Run in which argument has a max value
    /// e.g. `run_with_max_value_for_argument[0] = 5`
    /// means that the first argument will have the highest possible value in 5th run
    run_with_max_value_for_argument: Vec<u32>,
}

impl RunParams {
    pub fn from(rng: &mut StdRng, total_runs: u32, arguments: &[&str]) -> Result<Self> {
        assert!(total_runs >= 3);

        let arguments = arguments
            .iter()
            .map(|arg| CairoType::from_name(arg))
            .collect::<Result<Vec<_>>>()?;

        let run_with_min_value_for_argument: Vec<u32> = (0..arguments.len())
            .map(|_| rng.gen_range(1..=total_runs))
            .collect();
        let run_with_max_value_for_argument: Vec<u32> = run_with_min_value_for_argument
            .iter()
            .map(|&run_with_min| {
                let run_with_max = rng.gen_range(1..=total_runs);
                if run_with_max == run_with_min {
                    run_with_min + 1 % total_runs
                } else {
                    run_with_max
                }
            })
            .collect();

        Ok(Self {
            arguments,
            total_runs,
            executed_runs: 0,
            run_with_min_value_for_argument,
            run_with_max_value_for_argument,
        })
    }
}
