use crate::fuzzer::arguments::CairoType;
use anyhow::{Ok, Result};
use rand::rngs::StdRng;
use rand::Rng;

mod arguments;
mod random;

pub use random::RandomFuzzer;
use std::num::NonZeroU32;

#[derive(Debug, Clone)]
pub struct FuzzerRun {
    pub(super) cairo_type: CairoType,
    pub(super) run_with_min_value: u32,
    pub(super) run_with_max_value: u32,
}

#[derive(Debug, Clone)]
pub struct RunParams {
    /// Arguments
    pub(super) arguments: Vec<FuzzerRun>,
    /// Total number of runs
    total_runs: NonZeroU32,
    /// Number of already executed runs
    pub(super) executed_runs: u32,
}

impl RunParams {
    pub fn from(rng: &mut StdRng, total_runs: NonZeroU32, arguments: &[&str]) -> Result<Self> {
        let arguments = arguments
            .iter()
            .map(|arg| -> Result<FuzzerRun> {
                let argument = CairoType::from_name(arg)?;
                if total_runs.get() >= 3 {
                    let run_with_min_value = rng.gen_range(1..=total_runs.get());
                    let run_with_max_value = rng.gen_range(1..=total_runs.get());

                    let run_with_max_value = if run_with_max_value == run_with_min_value {
                        run_with_min_value % total_runs.get() + 1
                    } else {
                        run_with_max_value
                    };

                    Ok(FuzzerRun {
                        cairo_type: argument,
                        run_with_max_value,
                        run_with_min_value,
                    })
                } else {
                    Ok(FuzzerRun {
                        cairo_type: argument,
                        run_with_max_value: u32::MAX,
                        run_with_min_value: u32::MAX,
                    })
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            arguments,
            total_runs,
            executed_runs: 0,
        })
    }
}
