use crate::optimize_inlining::args::OptimizeInliningArgs;
use crate::optimize_inlining::runner::{OptimizationResult, TotalGas, run_optimization_iteration};
use anyhow::{Result, anyhow};
use argmin::core::{CostFunction, Error as ArgminError, Executor, State};
use argmin::solver::brent::BrentOpt;
use foundry_ui::UI;
use scarb_api::metadata::Metadata;
use std::cell::RefCell;
use std::collections::HashMap;
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

struct ScoreCostFunction<'a> {
    args: &'a OptimizeInliningArgs,
    scarb_metadata: &'a Metadata,
    ui: &'a Arc<UI>,
    gas_weight: f64,
    felts_weight: f64,
    max_gas: f64,
    max_felts: f64,
    min_threshold: f64,
    max_threshold: f64,
    step: f64,
    results: RefCell<Vec<OptimizationResult>>,
    cache: RefCell<HashMap<u32, f64>>,
}

impl CostFunction for ScoreCostFunction<'_> {
    type Param = f64;
    type Output = f64;

    fn cost(&self, p: &Self::Param) -> Result<Self::Output, ArgminError> {
        // let threshold = ((p / self.step).round() * self.step)
        //     .clamp(self.min_threshold, self.max_threshold) as u32;

        let threshold = p.clamp(self.min_threshold, self.max_threshold) as u32;

        if let Some(&cached_score) = self.cache.borrow().get(&threshold) {
            self.ui.print_blank_line();
            self.ui
                .println(&format!("Threshold {} (cached)", threshold));
            return Ok(cached_score);
        }

        self.ui.print_blank_line();
        self.ui
            .println(&format!("Testing threshold {}...", threshold));

        let result = run_optimization_iteration(threshold, self.args, self.scarb_metadata, self.ui)
            .map_err(|e| ArgminError::msg(e.to_string()))?;

        if let Some(ref error) = result.error {
            self.ui.println(&format!("  ✗ {}", error));
        } else {
            self.ui.println(&format!(
                "  ✓ Tests passed, gas: {}, max contract size: {} bytes, max felts: {}",
                result.total_gas.total(),
                result.max_contract_size,
                result.max_contract_felts
            ));
        }

        let score = if result.tests_passed && result.error.is_none() {
            let gas_ratio = result.total_gas.total() as f64 / self.max_gas;
            let felts_ratio = result.max_contract_felts as f64 / self.max_felts;
            let gas_component = gas_ratio * self.gas_weight;
            let felts_component = felts_ratio * self.felts_weight;
            let total_score = gas_component + felts_component;
            self.ui.println(&format!(
                "  Score: {:.6} (gas: {:.6} × {:.2} = {:.6}, felts: {:.6} × {:.2} = {:.6})",
                total_score,
                gas_ratio,
                self.gas_weight,
                gas_component,
                felts_ratio,
                self.felts_weight,
                felts_component
            ));
            total_score
        } else {
            f64::MAX
        };

        self.cache.borrow_mut().insert(threshold, score);
        self.results.borrow_mut().push(result);
        Ok(score)
    }
}

impl Optimizer {
    pub fn new(args: &OptimizeInliningArgs, scarb_metadata: &Metadata) -> Self {
        Self {
            min_threshold: args.min_threshold,
            max_threshold: args.max_threshold,
            step: args.step,
            gas_weight: args.gas_weight(),
            felts_weight: args.felts_weight(),
            results: Vec::new(),
            scarb_metadata: scarb_metadata.clone(),
        }
    }

    fn run_boundary_tests(&mut self, args: &OptimizeInliningArgs, ui: &Arc<UI>) -> Result<()> {
        ui.println(&"Running boundary tests...".to_string());

        ui.print_blank_line();
        ui.println(&format!("Testing min threshold {}...", self.min_threshold));
        let min_result =
            run_optimization_iteration(self.min_threshold, args, &self.scarb_metadata, ui)?;
        if min_result.tests_passed && min_result.error.is_none() {
            ui.println(&format!(
                "  ✓ gas: {}, felts: {}",
                min_result.total_gas.total(),
                min_result.max_contract_felts
            ));
        } else {
            ui.println(&format!(
                "  ✗ {}",
                min_result.error.as_deref().unwrap_or("failed")
            ));
        }

        ui.print_blank_line();
        ui.println(&format!("Testing max threshold {}...", self.max_threshold));
        let max_result =
            run_optimization_iteration(self.max_threshold, args, &self.scarb_metadata, ui)?;
        if max_result.tests_passed && max_result.error.is_none() {
            ui.println(&format!(
                "  ✓ gas: {}, felts: {}",
                max_result.total_gas.total(),
                max_result.max_contract_felts
            ));
        } else {
            ui.println(&format!(
                "  ✗ {}",
                max_result.error.as_deref().unwrap_or("failed")
            ));
        }

        self.results.push(min_result);
        self.results.push(max_result);

        Ok(())
    }

    fn get_max_values(&self) -> (u64, u64) {
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

        (max_gas, max_felts)
    }

    fn calculate_score(&self, result: &OptimizationResult, max_gas: u64, max_felts: u64) -> f64 {
        let gas_ratio = result.total_gas.total() as f64 / max_gas as f64;
        let felts_ratio = result.max_contract_felts as f64 / max_felts as f64;
        gas_ratio * self.gas_weight as f64 / 100.0 + felts_ratio * self.felts_weight as f64 / 100.0
    }

    pub fn optimize(&mut self, args: &OptimizeInliningArgs, ui: &Arc<UI>) -> Result<OptimalResult> {
        self.run_boundary_tests(args, ui)?;

        if args.bruteforce {
            self.optimize_bruteforce(args, ui)
        } else {
            self.optimize_brent(args, ui)
        }
    }

    fn optimize_brent(
        &mut self,
        args: &OptimizeInliningArgs,
        ui: &Arc<UI>,
    ) -> Result<OptimalResult> {
        ui.print_blank_line();
        ui.println(&"Running Brent optimization...".to_string());

        let (max_gas, max_felts) = self.get_max_values();

        let mut cache = HashMap::new();
        for r in &self.results {
            if r.tests_passed && r.error.is_none() {
                let score = self.calculate_score(r, max_gas, max_felts);
                cache.insert(r.threshold, score);
            }
        }

        let cost_function = ScoreCostFunction {
            args,
            scarb_metadata: &self.scarb_metadata,
            ui,
            gas_weight: self.gas_weight as f64 / 100.0,
            felts_weight: self.felts_weight as f64 / 100.0,
            max_gas: max_gas as f64,
            max_felts: max_felts as f64,
            min_threshold: self.min_threshold as f64,
            max_threshold: self.max_threshold as f64,
            step: self.step as f64,
            results: RefCell::new(Vec::new()),
            cache: RefCell::new(cache),
        };

        let solver = BrentOpt::new(self.min_threshold as f64, self.max_threshold as f64);

        let res = Executor::new(cost_function, solver)
            .configure(|state| state.max_iters(50))
            .run()
            .map_err(|e| anyhow!("Optimization failed: {}", e))?;

        let best_param = res
            .state()
            .get_best_param()
            .copied()
            .unwrap_or((self.min_threshold + self.max_threshold) as f64 / 2.0);
        let best_threshold = ((best_param / self.step as f64).round() * self.step as f64) as u32;

        let collected_results = res.problem.problem.unwrap().results.into_inner();
        self.results.extend(collected_results);

        ui.print_blank_line();
        ui.println(&format!(
            "Optimization complete. Best threshold: {}",
            best_threshold
        ));

        self.find_best_result()
    }

    fn optimize_bruteforce(
        &mut self,
        args: &OptimizeInliningArgs,
        ui: &Arc<UI>,
    ) -> Result<OptimalResult> {
        let total_iterations = ((self.max_threshold - self.min_threshold) / self.step) + 1;
        let mut current = self.min_threshold;
        let mut iteration = 1;

        while current <= self.max_threshold {
            if !self.results.iter().any(|r| r.threshold == current) {
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
            }
            current += self.step;
            iteration += 1;
        }

        self.find_best_result()
    }

    fn find_best_result(&self) -> Result<OptimalResult> {
        let (max_gas, max_felts) = self.get_max_values();

        self.results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .min_by(|a, b| {
                self.calculate_score(a, max_gas, max_felts)
                    .partial_cmp(&self.calculate_score(b, max_gas, max_felts))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| OptimalResult {
                threshold: r.threshold,
                total_gas: r.total_gas.clone(),
                max_contract_size: r.max_contract_size,
                max_contract_felts: r.max_contract_felts,
            })
            .ok_or_else(|| anyhow!("No valid optimization results found"))
    }

    pub fn print_results_table(&self, ui: &UI) {
        let mut sorted_results: Vec<_> = self.results.iter().collect();
        sorted_results.sort_by_key(|r| r.threshold);

        let (max_gas, max_felts) = self.get_max_values();

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

        for r in sorted_results {
            let status = if r.tests_passed && r.error.is_none() {
                "✓"
            } else {
                "✗"
            };
            let (gas_str, score_str) = if r.tests_passed && r.error.is_none() {
                let score = self.calculate_score(r, max_gas, max_felts);
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

        ui.print_blank_line();
        ui.println(&format!(
            "Normalization: max_gas={}, max_felts={}",
            max_gas, max_felts
        ));
        ui.println(&format!(
            "Weights: gas={}%, felts={}%",
            self.gas_weight, self.felts_weight
        ));

        if let Ok(best) = self.find_best_result() {
            let best_score = self.calculate_score(
                self.results
                    .iter()
                    .find(|r| r.threshold == best.threshold)
                    .unwrap(),
                max_gas,
                max_felts,
            );
            ui.print_blank_line();
            ui.println(&format!(
                "Best result: threshold={}, score={:.6}, gas={}, felts={}",
                best.threshold,
                best_score,
                best.total_gas.total(),
                best.max_contract_felts
            ));
        }
    }
}
