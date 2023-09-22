use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct RunParams {
    /// Inclusive value
    low: BigUint,
    /// Exclusive value
    high: BigUint,
    /// Number of arguments
    arguments_number: usize,
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
    pub fn from(
        rng: &mut StdRng,
        total_runs: u32,
        arguments_number: usize,
        low: &BigUint,
        high: &BigUint,
    ) -> Self {
        assert!(low < high);
        assert!(total_runs >= 3);

        let run_with_min_value_for_argument: Vec<u32> = (0..arguments_number)
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

        Self {
            low: low.clone(),
            high: high.clone(),
            arguments_number,
            total_runs,
            executed_runs: 0,
            run_with_min_value_for_argument,
            run_with_max_value_for_argument,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct RandomFuzzer {
    rng: StdRng,
    run_params: RunParams,
}

impl RandomFuzzer {
    pub fn new(
        seed: u64,
        total_runs: u32,
        arguments_number: usize,
        low: &BigUint,
        high: &BigUint,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let run_params = RunParams::from(&mut rng, total_runs, arguments_number, low, high);

        RandomFuzzer { rng, run_params }
    }

    pub fn next_felt252_args(&mut self) -> Vec<Felt252> {
        assert!(self.run_params.executed_runs < self.run_params.total_runs);

        self.next_run();

        (0..self.run_params.arguments_number)
            .map(|arg_number| {
                Felt252::from(if self.is_run_with_min_value_for_arg(arg_number) {
                    self.run_params.low.clone()
                } else if self.is_run_with_max_value_for_arg(arg_number) {
                    self.run_params.high.clone() - BigUint::one()
                } else {
                    self.rng
                        .gen_biguint_range(&self.run_params.low, &self.run_params.high)
                })
            })
            .collect()
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
    use cairo_felt::Felt252;
    use num_traits::Zero;
    use rand::{thread_rng, RngCore};

    impl Default for RunParams {
        fn default() -> Self {
            Self {
                low: BigUint::zero(),
                high: Felt252::prime(),
                arguments_number: 0,
                total_runs: 256,
                executed_runs: 0,
                run_with_min_value_for_argument: vec![],
                run_with_max_value_for_argument: vec![],
            }
        }
    }

    #[test]
    fn run_with_max_value() {
        let run_params = RunParams {
            run_with_max_value_for_argument: vec![1],
            run_with_min_value_for_argument: vec![2],
            arguments_number: 1,
            total_runs: 10,
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };

        assert!(!fuzzer.is_run_with_max_value_for_arg(0));
        fuzzer.next_felt252_args();
        assert!(fuzzer.is_run_with_max_value_for_arg(0));
    }

    #[test]
    fn run_with_min_value() {
        let run_params = RunParams {
            run_with_max_value_for_argument: vec![2],
            run_with_min_value_for_argument: vec![1],
            arguments_number: 1,
            total_runs: 10,
            ..Default::default()
        };
        let mut fuzzer = RandomFuzzer {
            rng: StdRng::seed_from_u64(1234),
            run_params,
        };

        assert!(!fuzzer.is_run_with_min_value_for_arg(0));
        fuzzer.next_felt252_args();
        assert!(fuzzer.is_run_with_min_value_for_arg(0));
    }

    #[test]
    fn using_seed_consistent_result() {
        let seed = thread_rng().next_u64();
        let mut fuzzer = RandomFuzzer::new(seed, 3, 5, &BigUint::zero(), &Felt252::prime());
        let values = fuzzer.next_felt252_args();

        let mut fuzzer = RandomFuzzer::new(seed, 3, 5, &BigUint::zero(), &Felt252::prime());
        let values_from_seed = fuzzer.next_felt252_args();

        assert_eq!(values, values_from_seed);
    }

    #[test]
    fn min_and_max_used_at_least_once_for_each_arg() {
        let seed = thread_rng().next_u64();
        let runs_number = 100;
        let args_number = 10;
        let low = BigUint::from(420u16);
        let high = BigUint::from(2137u16);

        let mut fuzzer = RandomFuzzer::new(seed, runs_number, args_number, &low, &high);

        let low = Felt252::from(low);
        let high = Felt252::from(high);

        let mut min_used = vec![false; args_number];
        let mut max_used = vec![false; args_number];

        for _ in 1..=runs_number {
            let values = fuzzer.next_felt252_args();
            for (i, value) in values.iter().enumerate() {
                assert!(*value >= low && *value < high);
                if *value == low {
                    min_used[i] = true;
                } else if *value == high.clone() - Felt252::one() {
                    max_used[i] = true;
                }
            }
        }

        assert_eq!(min_used, vec![true; args_number]);
        assert_eq!(max_used, vec![true; args_number]);
    }
}
