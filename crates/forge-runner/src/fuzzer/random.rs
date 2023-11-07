use crate::fuzzer::arguments::Argument;
use crate::fuzzer::RunParams;
use anyhow::Result;
use cairo_felt::Felt252;
use rand::prelude::StdRng;
use rand::SeedableRng;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RandomFuzzer {
    rng: StdRng,
    run_params: RunParams,
}

impl RandomFuzzer {
    pub fn create(seed: u64, total_runs: u32, arguments: &[&str]) -> Result<Self> {
        let mut rng = StdRng::seed_from_u64(seed);
        let run_params = RunParams::from(&mut rng, total_runs, arguments)?;

        Ok(Self { rng, run_params })
    }

    pub fn next_args(&mut self) -> Vec<Felt252> {
        assert!(self.run_params.executed_runs < self.run_params.total_runs);

        self.next_run();

        let mut args = vec![];

        for (index, argument) in self.run_params.arguments.iter().enumerate() {
            if self.is_run_with_min_value_for_arg(index) {
                args.extend(argument.min());
            } else if self.is_run_with_max_value_for_arg(index) {
                args.extend(argument.max());
            } else {
                args.extend(argument.gen(&mut self.rng));
            }
        }

        args
    }

    fn next_run(&mut self) {
        self.run_params.executed_runs += 1;
    }

    fn is_run_with_min_value_for_arg(&self, arg_number: usize) -> bool {
        let current_run = self.run_params.executed_runs;
        self.run_params.run_with_min_value_for_argument[arg_number] == current_run
    }

    fn is_run_with_max_value_for_arg(&self, arg_number: usize) -> bool {
        let current_run = self.run_params.executed_runs;
        self.run_params.run_with_max_value_for_argument[arg_number] == current_run
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzzer::arguments::CairoType;
    use num_bigint::BigUint;
    use num_traits::Zero;
    use rand::{thread_rng, RngCore};

    impl Default for RunParams {
        fn default() -> Self {
            Self {
                arguments: vec![],
                total_runs: 256,
                executed_runs: 0,
                run_with_min_value_for_argument: vec![],
                run_with_max_value_for_argument: vec![],
            }
        }
    }

    fn all_values_different(vec1: &[Felt252], vec2: &[Felt252]) -> bool {
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
            run_with_max_value_for_argument: vec![3, 3, 3],
            run_with_min_value_for_argument: vec![4, 4, 4],
            arguments: vec![CairoType::Felt252, CairoType::U256, CairoType::U32],
            total_runs: 10,
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
            run_with_max_value_for_argument: vec![1],
            run_with_min_value_for_argument: vec![2],
            total_runs: 10,
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };

        assert!(!fuzzer.is_run_with_max_value_for_arg(0));
        fuzzer.next_args();
        assert!(fuzzer.is_run_with_max_value_for_arg(0));
    }

    #[test]
    fn run_with_min_value() {
        let run_params = RunParams {
            run_with_max_value_for_argument: vec![2],
            run_with_min_value_for_argument: vec![1],
            total_runs: 10,
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };

        assert!(!fuzzer.is_run_with_min_value_for_arg(0));
        fuzzer.next_args();
        assert!(fuzzer.is_run_with_min_value_for_arg(0));
    }

    #[test]
    fn using_seed_consistent_result() {
        let seed = thread_rng().next_u64();
        let mut fuzzer = RandomFuzzer::create(seed, 3, &["felt252", "felt252", "felt252"]).unwrap();
        let values = fuzzer.next_args();

        let mut fuzzer = RandomFuzzer::create(seed, 3, &["felt252", "felt252", "felt252"]).unwrap();
        let values_from_seed = fuzzer.next_args();

        assert_eq!(values, values_from_seed);
    }

    #[test]
    fn min_and_max_used_at_least_once_for_each_arg() {
        let seed = thread_rng().next_u64();
        let runs_number = 10;
        let arguments = vec!["felt252", "felt252", "felt252"];
        let args_number = arguments.len();

        let mut fuzzer = RandomFuzzer::create(seed, runs_number, &arguments).unwrap();

        let mut min_used = vec![false; args_number];
        let mut max_used = vec![false; args_number];

        for _ in 1..=runs_number {
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
        let result = RandomFuzzer::create(1234, 512, &["felt252", "invalid", "args"]);
        let err = result.unwrap_err();

        assert_eq!(
            err.to_string(),
            "Tried to use incorrect type for fuzzing. Type = invalid is not supported"
        );
    }
}
