use crate::optimize_inlining::args::OptimizeInliningArgs;
use crate::optimize_inlining::runner::{OptimizationResult, TotalGas, run_optimization_iteration};
use anyhow::{Result, anyhow};
use foundry_ui::UI;
use scarb_api::metadata::Metadata;
use std::sync::Arc;

pub struct Optimizer {
    pub min_threshold: u32,
    pub max_threshold: u32,
    pub step: u32,
    pub gas_weight: u8,
    pub felts_weight: u8,
    pub results: Vec<OptimizationResult>,
    scarb_metadata: Metadata,
}

pub struct OptimalResult {
    pub threshold: u32,
    pub total_gas: TotalGas,
    pub max_contract_size: u64,
    pub max_contract_felts: u64,
}

impl Optimizer {
    pub fn new(args: &OptimizeInliningArgs, scarb_metadata: &Metadata) -> Self {
        Self {
            min_threshold: args.min_threshold,
            max_threshold: args.max_threshold,
            step: args.step,
            gas_weight: args.gas_weight,
            felts_weight: args.felts_weight,
            results: Vec::new(),
            scarb_metadata: scarb_metadata.clone(),
        }
    }

    pub fn optimize(&mut self, args: &OptimizeInliningArgs, ui: &Arc<UI>) -> Result<OptimalResult> {
        let total_iterations = ((self.max_threshold - self.min_threshold) / self.step) + 1;
        let mut current = self.min_threshold;
        let mut iteration = 1;

        while current <= self.max_threshold {
            ui.print_blank_line();
            ui.println(&format!(
                "[{}/{}] Testing threshold {}...",
                iteration, total_iterations, current
            ));

            let result = run_optimization_iteration(current, args, &self.scarb_metadata, ui)?;

            if let Some(ref error) = result.error {
                ui.println(&format!("  ✗ {}", error));
            } else {
                ui.println(&format!(
                    "  ✓ Tests passed, gas: {}, max contract size: {} bytes, max felts: {}",
                    result.total_gas.total(),
                    result.max_contract_size,
                    result.max_contract_felts
                ));
            }

            self.results.push(result);
            current += self.step;
            iteration += 1;
        }

        self.find_best_result()
    }

    fn find_best_result(&self) -> Result<OptimalResult> {
        self.results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .min_by_key(|r| r.total_gas.total())
            .map(|r| OptimalResult {
                threshold: r.threshold,
                total_gas: r.total_gas.clone(),
                max_contract_size: r.max_contract_size,
                max_contract_felts: r.max_contract_felts,
            })
            .ok_or_else(|| anyhow!("No valid optimization results found"))
    }

    pub fn print_results_table(&self, ui: &UI) {
        let valid_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .collect();

        let max_gas = valid_results
            .iter()
            .map(|r| r.total_gas.total())
            .max()
            .unwrap_or(1);
        let max_felts = valid_results
            .iter()
            .map(|r| r.max_contract_felts)
            .max()
            .unwrap_or(1);

        ui.println(
            &"┌──────────────┬─────────────────┬──────────────────┬──────────────┬────────────┬────────┐"
                .to_string(),
        );
        ui.println(
            &"│  Threshold   │    Total Gas    │  Contract Size   │    Felts     │   Score    │ Status │"
                .to_string(),
        );
        ui.println(
            &"├──────────────┼─────────────────┼──────────────────┼──────────────┼────────────┼────────┤"
                .to_string(),
        );

        for r in &self.results {
            let status = if r.tests_passed && r.error.is_none() {
                "✓"
            } else {
                "✗"
            };
            let (gas_str, score_str) = if r.tests_passed && r.error.is_none() {
                let gas_ratio = r.total_gas.total() as f64 / max_gas as f64;
                let felts_ratio = r.max_contract_felts as f64 / max_felts as f64;
                let score =
                    gas_ratio * self.gas_weight as f64 + felts_ratio * self.felts_weight as f64;
                (
                    format!("{:>13}", r.total_gas.total()),
                    format!("{:>8.6}", score),
                )
            } else {
                (format!("{:>13}", "-"), format!("{:>8}", "-"))
            };
            ui.println(&format!(
                "│ {:>10}   │ {}   │ {:>14}   │ {:>10}   │ {}   │   {}    │",
                r.threshold, gas_str, r.max_contract_size, r.max_contract_felts, score_str, status
            ));
        }

        ui.println(
            &"└──────────────┴─────────────────┴──────────────────┴──────────────┴────────────┴────────┘"
                .to_string(),
        );
    }
}
