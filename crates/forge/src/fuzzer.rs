use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::default::Default;

#[derive(Default)]
#[allow(clippy::module_name_repetitions)]
pub struct FuzzerRunParams {
    /// Inclusive value
    low: BigUint,
    /// Exclusive value
    high: BigUint,
    /// Number of arguments
    args_number: usize,
    /// Total number of runs
    runs_number: u32,
    /// Number of already executed runs
    runs_executed: u32,
    /// Runs in which arguments have min value e.g. `runs_with_min_value[0] = 5`
    /// means that the first argument has the lowest possible value in 5th run
    runs_with_min_value: Vec<u32>,
    /// Runs in which arguments have max value e.g. `runs_with_max_value[0] = 5`
    /// means that the first argument has the highest possible value in 5th run
    runs_with_max_value: Vec<u32>,
}

#[allow(clippy::module_name_repetitions)]
pub struct RandomFuzzer {
    rng: StdRng,
    fuzzer_run_params: FuzzerRunParams,
}

impl RandomFuzzer {
    pub fn new(
        seed: u64,
        runs_number: u32,
        args_number: usize,
        low: BigUint,
        high: BigUint,
    ) -> Self {
        let rng = StdRng::seed_from_u64(seed);

        let mut fuzzer = RandomFuzzer {
            rng,
            fuzzer_run_params: FuzzerRunParams::default(),
        };
        fuzzer.set_fuzzer_run_params(runs_number, args_number, low, high);

        fuzzer
    }

    fn set_fuzzer_run_params(
        &mut self,
        runs_number: u32,
        args_number: usize,
        low: BigUint,
        high: BigUint,
    ) {
        assert!(low < high);
        assert!(runs_number >= 3);

        self.fuzzer_run_params.runs_executed = 0;
        self.fuzzer_run_params.low = low;
        self.fuzzer_run_params.high = high;
        self.fuzzer_run_params.runs_number = runs_number;
        self.fuzzer_run_params.args_number = args_number;

        let runs_with_min_value: Vec<u32> = vec![0; self.fuzzer_run_params.args_number]
            .into_iter()
            .map(|_| self.rng.gen_range(1..=self.fuzzer_run_params.runs_number))
            .collect();
        let runs_with_max_value: Vec<u32> = vec![0; self.fuzzer_run_params.args_number]
            .into_iter()
            .zip(&runs_with_min_value)
            .map(|(_, &run_with_min)| {
                let run_with_max = self.rng.gen_range(1..=self.fuzzer_run_params.runs_number);
                if run_with_max == run_with_min {
                    run_with_min + 1 % runs_number
                } else {
                    run_with_max
                }
            })
            .collect();

        self.fuzzer_run_params.runs_with_max_value = runs_with_max_value;
        self.fuzzer_run_params.runs_with_min_value = runs_with_min_value;
    }

    pub fn next_felt252_args(&mut self) -> Vec<Felt252> {
        assert!(self.fuzzer_run_params.runs_executed < self.fuzzer_run_params.runs_number);

        self.fuzzer_run_params.runs_executed += 1;
        let current_run = self.fuzzer_run_params.runs_executed;

        let args_values: Vec<Felt252> = vec![0; self.fuzzer_run_params.args_number]
            .into_iter()
            .enumerate()
            .map(|(i, _)| {
                Felt252::from(
                    if self.fuzzer_run_params.runs_with_min_value[i] == current_run {
                        self.fuzzer_run_params.low.clone()
                    } else if self.fuzzer_run_params.runs_with_max_value[i] == current_run {
                        self.fuzzer_run_params.high.clone() - BigUint::one()
                    } else {
                        self.rng.gen_biguint_range(
                            &self.fuzzer_run_params.low,
                            &self.fuzzer_run_params.high,
                        )
                    },
                )
            })
            .collect();

        args_values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairo_felt::Felt252;
    use num_traits::Zero;
    use rand::{thread_rng, RngCore};

    #[test]
    fn using_seed_consistent_result() {
        let seed = thread_rng().next_u64();
        let mut fuzzer = RandomFuzzer::new(seed, 3, 5, BigUint::zero(), Felt252::prime());
        let values = fuzzer.next_felt252_args();

        let mut fuzzer = RandomFuzzer::new(seed, 3, 5, BigUint::zero(), Felt252::prime());
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

        let mut fuzzer =
            RandomFuzzer::new(seed, runs_number, args_number, low.clone(), high.clone());

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
