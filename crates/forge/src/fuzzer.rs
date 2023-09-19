use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_traits::One;
use rand::rngs::StdRng;
use rand::{thread_rng, Rng, RngCore, SeedableRng};
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
    /// Runs in which arguments have min value e.g. `run_with_min_value[0] = 5`
    /// means that the first argument has the lowest possible value in 5th run
    runs_with_min_value: Vec<u32>,
    /// Runs in which arguments have max value e.g. `run_with_min_value[0] = 5`
    /// means that the first argument has the highest possible value in 5th run
    runs_with_max_value: Vec<u32>,
}

pub struct Random {
    rng: StdRng,
    seed: u64,
    fuzzer_run_params: FuzzerRunParams,
    pub was_fuzzed: bool,
}

impl Random {
    pub fn new() -> Self {
        let mut seed_rng = thread_rng();
        let seed = seed_rng.next_u64();

        let rng = StdRng::seed_from_u64(seed);

        Random {
            rng,
            seed,
            fuzzer_run_params: FuzzerRunParams::default(),
            was_fuzzed: false,
        }
    }

    pub fn from_seed(seed: u64) -> Self {
        Random {
            rng: StdRng::seed_from_u64(seed),
            seed,
            fuzzer_run_params: FuzzerRunParams::default(),
            was_fuzzed: false,
        }
    }
}

impl Random {
    pub fn set_fuzzer_run_params(
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

    pub fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairo_felt::Felt252;
    use num_traits::Zero;

    #[test]
    fn seed_is_set() {
        let fuzzer = Random::from_seed(12345);
        assert_eq!(fuzzer.seed, 12345);
        assert_eq!(fuzzer.seed, fuzzer.seed());
    }

    #[test]
    fn using_seed_consistent_result() {
        let mut fuzzer = Random::new();
        fuzzer.set_fuzzer_run_params(3, 5, BigUint::zero(), Felt252::prime());
        let values = fuzzer.next_felt252_args();

        let mut fuzzer_from_seed = Random::from_seed(fuzzer.seed);
        fuzzer_from_seed.set_fuzzer_run_params(3, 5, BigUint::zero(), Felt252::prime());
        let values_from_seed = fuzzer_from_seed.next_felt252_args();

        assert_eq!(values, values_from_seed);
    }

    #[test]
    fn min_and_max_used_at_least_once_for_each_arg() {
        let fuzzer_runs = 100;
        let args_number = 10;
        let low = BigUint::from(420u16);
        let high = BigUint::from(2137u16);

        let mut fuzzer = Random::new();
        fuzzer.set_fuzzer_run_params(fuzzer_runs, args_number, low.clone(), high.clone());

        let low = Felt252::from(low);
        let high = Felt252::from(high);

        let mut min_used = vec![false; args_number];
        let mut max_used = vec![false; args_number];

        for _ in 1..=fuzzer_runs {
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
