use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_integer::Integer;
use num_traits::{One, Zero};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::ops::{Shl, Shr, Sub};

enum CairoType {
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Felt252,
}

trait Argument {
    fn low(&self) -> BigUint;
    fn high(&self) -> BigUint;
    fn gen(&self, rng: &mut StdRng) -> Vec<Felt252>;
    fn min(&self) -> Vec<Felt252>;
    fn max(&self) -> Vec<Felt252>;
}

impl Argument for CairoType {
    fn low(&self) -> BigUint {
        BigUint::zero()
    }

    fn high(&self) -> BigUint {
        match self {
            CairoType::U8 => BigUint::from(u8::MAX),
            CairoType::U16 => BigUint::from(u16::MAX),
            CairoType::U32 => BigUint::from(u32::MAX),
            CairoType::U64 => BigUint::from(u64::MAX),
            CairoType::U128 => BigUint::from(u128::MAX),
            CairoType::U256 => {
                let max = BigUint::from(1_u32);
                let max = max.shl(256);
                max - BigUint::one()
            }
            CairoType::Felt252 => Felt252::prime().sub(BigUint::one()),
        }
    }

    fn gen(&self, rng: &mut StdRng) -> Vec<Felt252> {
        match self {
            CairoType::U8
            | CairoType::U16
            | CairoType::U32
            | CairoType::U64
            | CairoType::U128
            | CairoType::Felt252 => {
                vec![Felt252::from(
                    rng.gen_biguint_range(&self.low(), &self.high()),
                )]
            }
            CairoType::U256 => {
                let val = rng.gen_biguint_range(&self.low(), &self.high());
                u256_to_felt252(val)
            }
        }
    }

    fn min(&self) -> Vec<Felt252> {
        match self {
            CairoType::U8
            | CairoType::U16
            | CairoType::U32
            | CairoType::U64
            | CairoType::U128
            | CairoType::Felt252 => vec![Felt252::from(self.low())],
            CairoType::U256 => vec![Felt252::from(self.low()), Felt252::from(self.low())],
        }
    }

    fn max(&self) -> Vec<Felt252> {
        match self {
            CairoType::U8
            | CairoType::U16
            | CairoType::U32
            | CairoType::U64
            | CairoType::U128
            | CairoType::Felt252 => vec![Felt252::from(self.high())],
            CairoType::U256 => u256_to_felt252(self.high()),
        }
    }
}

fn u256_to_felt252(val: BigUint) -> Vec<Felt252> {
    let low = val.mod_floor(&BigUint::from(2_u32).pow(128));
    let high = val.shr(128);
    vec![Felt252::from(low), Felt252::from(high)]
}

impl CairoType {
    fn from_name(name: &str) -> Self {
        match name {
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "u128" => Self::U128,
            "u256" | "core::integer::u256" => Self::U256,
            "felt252" => Self::Felt252,
            _ => panic!(), // TODO add better handling
        }
    }
}

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
    pub fn from(rng: &mut StdRng, total_runs: u32, arguments: &[&str]) -> Self {
        assert!(total_runs >= 3);

        let arguments_number = arguments.len();
        let arguments = arguments
            .iter()
            .map(|arg| CairoType::from_name(arg))
            .collect();

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
            arguments,
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
    pub fn new(seed: u64, total_runs: u32, arguments: &[&str]) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let run_params = RunParams::from(&mut rng, total_runs, arguments);

        RandomFuzzer { rng, run_params }
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
        let mut fuzzer = RandomFuzzer::new(seed, 3, &["felt252", "felt252", "felt252"]);
        let values = fuzzer.next_args();

        let mut fuzzer = RandomFuzzer::new(seed, 3, &["felt252", "felt252", "felt252"]);
        let values_from_seed = fuzzer.next_args();

        assert_eq!(values, values_from_seed);
    }

    #[test]
    fn min_and_max_used_at_least_once_for_each_arg() {
        let seed = thread_rng().next_u64();
        let runs_number = 10;
        let arguments = vec!["felt252", "felt252", "felt252"];
        let args_number = arguments.len();

        let mut fuzzer = RandomFuzzer::new(seed, runs_number, &arguments);

        let mut min_used = vec![false; args_number];
        let mut max_used = vec![false; args_number];

        for _ in 1..=runs_number {
            let values = fuzzer.next_args();
            for (i, value) in values.iter().enumerate() {
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
}
