use crate::fuzzer::RunParams;
use anyhow::Result;
use rand::prelude::StdRng;
use rand::SeedableRng;
use starknet_types_core::felt::Felt;
use std::num::NonZeroU32;

#[derive(Debug, Clone)]
pub struct RandomFuzzer {
    rng: StdRng,
    run_params: RunParams,
}

impl RandomFuzzer {
    pub fn create(seed: u64, total_runs: NonZeroU32, arguments: &[&str]) -> Result<Self> {
        let mut rng = StdRng::seed_from_u64(seed);
        let run_params = RunParams::from(&mut rng, total_runs, arguments)?;

        Ok(Self { rng, run_params })
    }

    pub fn next_args(&mut self) -> Vec<Felt> {
        assert!(self.run_params.executed_runs < self.run_params.total_runs.get());

        self.next_run();

        self.run_params
            .arguments
            .iter()
            .flat_map(|argument| {
                let current_run = self.run_params.executed_runs;

                if argument.run_with_min_value == current_run {
                    argument.cairo_type.min()
                } else if argument.run_with_max_value == current_run {
                    argument.cairo_type.max()
                } else {
                    argument.cairo_type.gen(&mut self.rng)
                }
            })
            .collect()
    }

    fn next_run(&mut self) {
        self.run_params.executed_runs += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzzer::{arguments::CairoType, FuzzerArg};
    use num_bigint::BigUint;
    use num_traits::Zero;
    use rand::{thread_rng, RngCore};

    impl FuzzerArg {
        pub fn new(
            cairo_type: CairoType,
            run_with_min_value: u32,
            run_with_max_value: u32,
        ) -> Self {
            Self {
                cairo_type,
                run_with_min_value,
                run_with_max_value,
            }
        }
    }

    impl Default for RunParams {
        fn default() -> Self {
            Self {
                arguments: vec![],
                total_runs: NonZeroU32::new(256).unwrap(),
                executed_runs: 0,
            }
        }
    }

    fn all_values_different(vec1: &[Felt], vec2: &[Felt]) -> bool {
        vec1.iter().zip(vec2).all(|(i, j)| {
            i.to_biguint() != j.to_biguint()
                && i.to_biguint() != BigUint::zero()
                && j.to_biguint() != BigUint::zero()
        })
    }

    /// Verify that generated values are actually different between `next_args()` calls
    ///
    /// This test has been added after realizing that due to a logic bug (`rng` was being copied
    /// between calls), we were generating random values but same for each run. This tests
    /// will prevent future cases like this.
    #[test]
    fn fuzzer_generates_different_values() {
        let run_params = RunParams {
            arguments: vec![
                FuzzerArg::new(CairoType::Felt252, 4, 3),
                FuzzerArg::new(CairoType::U256, 4, 3),
                FuzzerArg::new(CairoType::U32, 4, 3),
            ],
            total_runs: NonZeroU32::new(10).unwrap(),
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };

        let args1 = fuzzer.next_args();
        let args2 = fuzzer.next_args();
        let args3 = fuzzer.next_args();

        assert!(all_values_different(&args1, &args2));
        assert!(all_values_different(&args1, &args3));
        assert!(all_values_different(&args2, &args3));
    }

    #[test]
    fn run_with_max_value() {
        let run_params = RunParams {
            arguments: vec![FuzzerArg::new(CairoType::Felt252, 2, 1)],
            total_runs: NonZeroU32::new(10).unwrap(),
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };
        let current_run = fuzzer.run_params.executed_runs;
        assert!(fuzzer.run_params.arguments[0].run_with_max_value != current_run);

        fuzzer.next_args();

        let current_run = fuzzer.run_params.executed_runs;
        assert!(fuzzer.run_params.arguments[0].run_with_max_value == current_run);
    }

    #[test]
    fn run_with_min_value() {
        let run_params = RunParams {
            total_runs: NonZeroU32::new(10).unwrap(),
            arguments: vec![FuzzerArg::new(CairoType::Felt252, 1, 2)],
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };

        let current_run = fuzzer.run_params.executed_runs;
        assert!(fuzzer.run_params.arguments[0].run_with_min_value != current_run);

        fuzzer.next_args();

        let current_run = fuzzer.run_params.executed_runs;
        assert!(fuzzer.run_params.arguments[0].run_with_min_value == current_run);
    }

    #[test]
    fn using_seed_consistent_result() {
        let seed = thread_rng().next_u64();
        let mut fuzzer = RandomFuzzer::create(
            seed,
            NonZeroU32::new(3).unwrap(),
            &["felt252", "felt252", "felt252"],
        )
        .unwrap();
        let values = fuzzer.next_args();

        let mut fuzzer = RandomFuzzer::create(
            seed,
            NonZeroU32::new(3).unwrap(),
            &["felt252", "felt252", "felt252"],
        )
        .unwrap();
        let values_from_seed = fuzzer.next_args();

        assert_eq!(values, values_from_seed);
    }

    #[test]
    fn min_and_max_used_at_least_once_for_each_arg() {
        let seed = thread_rng().next_u64();
        let runs_number = NonZeroU32::new(10).unwrap();
        let arguments = vec!["felt252", "felt252", "felt252"];
        let args_number = arguments.len();

        let mut fuzzer = RandomFuzzer::create(seed, runs_number, &arguments).unwrap();

        let mut min_used = vec![false; args_number];
        let mut max_used = vec![false; args_number];

        for _ in 1..=runs_number.get() {
            let values = fuzzer.next_args();
            for (i, value) in values.iter().enumerate() {
                assert!(
                    *value >= CairoType::Felt252.min()[0] && *value <= CairoType::Felt252.max()[0]
                );
                if *value == CairoType::Felt252.min()[0] {
                    min_used[i] = true;
                } else if *value == CairoType::Felt252.max()[0] {
                    max_used[i] = true;
                }
            }
        }

        assert_eq!(min_used, vec![true; args_number]);
        assert_eq!(max_used, vec![true; args_number]);
    }

    #[test]
    fn create_fuzzer_from_invalid_arguments() {
        let result = RandomFuzzer::create(
            1234,
            NonZeroU32::new(512).unwrap(),
            &["felt252", "invalid", "args"],
        );
        let err = result.unwrap_err();

        assert_eq!(
            err.to_string(),
            "Tried to use incorrect type for fuzzing. Type = invalid is not supported"
        );
    }
    #[test]
    fn fuzzer_less_than_3_runs() {
        for runs in 1..2 {
            let result = RandomFuzzer::create(1234, NonZeroU32::new(runs).unwrap(), &["felt252"]);
            let mut fuzzer = result.unwrap();

            // just check if it panics
            fuzzer.next_args();
        }
    }
}
