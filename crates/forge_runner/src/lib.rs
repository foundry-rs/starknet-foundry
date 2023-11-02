use crate::fuzzer::RandomFuzzer;
use crate::test_case_summary::TestCaseSummary;
use crate::test_crate_summary::TestCrateSummary;
use anyhow::{anyhow, Context, Result};
use cairo_felt::Felt252;
use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::Function;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use once_cell::sync::Lazy;
use smol_str::SmolStr;
use std::sync::Arc;
use test_collector::FuzzerConfig;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

mod fuzzer;
mod running;
pub mod test_case_summary;
pub mod test_crate_summary;
mod test_execution_syscall_handler;

pub static BUILTINS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "Pedersen",
        "RangeCheck",
        "Bitwise",
        "EcOp",
        "Poseidon",
        "SegmentArena",
        "GasBuiltin",
        "System",
    ]
});

/// Exit status of the runner
#[derive(Debug, PartialEq, Clone)]
pub enum RunnerStatus {
    /// Runner exited without problems
    Default,
    /// Some test failed
    TestFailed,
    /// Runner did not run, e.g. when test cases got skipped
    DidNotRun,
}

pub async fn run_tests_from_crate(
    tests: Arc<CompiledTestCrateRunnable>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
) -> Result<TestCrateSummary> {
    let runner = Arc::new(
        SierraCasmRunner::new(
            tests.sierra_program.clone(),
            Some(MetadataComputationConfig::default()),
            OrderedHashMap::default(),
        )
        .context("Failed setting up runner.")?,
    );

    let mut tasks = FuturesUnordered::new();
    let test_cases = &tests.test_cases;
    // Initiate two channels to manage the `--exit-first` flag.
    // Owing to `cheatnet` fork's utilization of its own Tokio runtime for RPC requests,
    // test execution must occur within a `tokio::spawn_blocking`.
    // As `spawn_blocking` can't be prematurely cancelled (refer: https://dtantsur.github.io/rust-openstack/tokio/task/fn.spawn_blocking.html),
    // a channel is used to signal the task that test processing is no longer necessary.
    let (send, mut rec) = channel(1);

    // The second channel serves as a hold point to ensure all tasks complete
    // their shutdown procedures before moving forward (more info: https://tokio.rs/tokio/topics/shutdown)
    let (send_shut_down, mut rec_shut_down) = channel(1);

    for case in test_cases.iter() {
        let case_name = case.name.clone();

        if !runner_config
            .tests_filter
            .should_be_run_based_on_ignored(case)
        {
            tasks.push(tokio::task::spawn(async {
                Ok(TestCaseSummary::Ignored { name: case_name })
            }));
            continue;
        };

        let function = runner.find_function(&case_name)?;
        let args = function_args(function, &BUILTINS);

        let case = Arc::new(case.clone());
        let args: Vec<ConcreteTypeId> = args.into_iter().cloned().collect();
        let runner = runner.clone();

        tasks.push(choose_test_strategy_and_run(
            args,
            case.clone(),
            runner,
            runner_config.clone(),
            runner_params.clone(),
            cancellation_tokens.clone(),
            &send,
            &send_shut_down,
        ));
    }

    let mut results = vec![];
    let mut interrupted = false;

    while let Some(task) = tasks.next().await {
        let result = task??;
        match result {
            // Because tests are executed parallel is possible to receive
            // Ok(TestCaseSummary::Interrupted) before Err
            TestCaseSummary::Interrupted {} => interrupted = true,
            result => {
                pretty_printing::print_test_result(&result);

                results.push(result);
            }
        }
    }

    rec.close();

    // Waiting for things to finish shutting down
    drop(send_shut_down);
    let _ = rec_shut_down.recv().await;

    // This Panic should never occur.
    // If TestCaseSummary::Interrupted is returned by a test,
    // this implies that there should be another test than returned an Err.
    assert!(!interrupted, "Tests were interrupted");

    let contained_fuzzed_tests = results.iter().any(|summary| summary.runs().is_some());
    Ok(TestCrateSummary {
        test_case_summaries: results,
        runner_exit_status: RunnerStatus::Default,
        test_crate_type: tests.tests_location,
        contained_fuzzed_tests,
    })
}

#[allow(clippy::too_many_arguments)]
fn choose_test_strategy_and_run(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    send: &Sender<()>,
    send_shut_down: &Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    if args.is_empty() {
        run_single_test(
            case,
            runner,
            runner_config,
            runner_params,
            cancellation_tokens,
            send.clone(),
            send_shut_down.clone(),
        )
    } else {
        run_with_fuzzing(
            args,
            case,
            runner,
            runner_config,
            runner_params,
            cancellation_tokens,
            send_shut_down.clone(),
        )
    }
}

fn run_single_test(
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    let exit_first = runner_config.exit_first;
    tokio::task::spawn(async move {
        tokio::select! {
            () = cancellation_tokens.exit_first.cancelled() => {
                // Stop executing all tests because flag --exit-first'
                // has been set and one test FAIL
                Ok(TestCaseSummary::skipped(&case))
            },
            () = cancellation_tokens.error.cancelled() => {
                // Stop executing all tests because
                // one of a test returns Err
                Ok(TestCaseSummary::Interrupted{  })
            },

            result = blocking_run_from_test(vec![], case.clone(),runner,  runner_config.clone(), runner_params.clone(), send.clone(), send_shut_down.clone() ) => {
                match result? {
                    Ok(result) => {
                        if exit_first {
                            if let TestCaseSummary::Failed { .. } = &result {
                                cancellation_tokens.exit_first.cancel();
                            }
                        }
                        Ok(result)
                    }
                    Err(e) => {
                        cancellation_tokens.error.cancel();
                        Err(e)
                    }
                }
            }
        }
    })
}

fn run_with_fuzzing(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    tokio::task::spawn(async move {
        let cancellation_fuzzing_token = CancellationToken::new();
        let (send, mut rec) = channel(1);
        let args = args
            .iter()
            .map(|arg| {
                arg.debug_name
                    .as_ref()
                    .ok_or_else(|| anyhow!("Type {arg:?} does not have a debug name"))
                    .map(SmolStr::as_str)
            })
            .collect::<Result<Vec<_>>>()?;

        let (fuzzer_runs, fuzzer_seed) = match case.fuzzer_config {
            Some(FuzzerConfig {
                fuzzer_runs,
                fuzzer_seed,
            }) => (fuzzer_runs, fuzzer_seed),
            _ => (runner_config.fuzzer_runs, runner_config.fuzzer_seed),
        };
        let mut fuzzer = RandomFuzzer::create(fuzzer_seed, fuzzer_runs, &args)?;

        let mut tasks = FuturesUnordered::new();

        for _ in 1..=fuzzer_runs {
            let args = fuzzer.next_args();

            tasks.push(run_fuzzing_subtest(
                args,
                case.clone(),
                runner.clone(),
                runner_config.clone(),
                runner_params.clone(),
                cancellation_tokens.clone(),
                cancellation_fuzzing_token.clone(),
                send.clone(),
                send_shut_down.clone(),
            ));
        }

        let mut results = vec![];
        let mut final_result = None;

        while let Some(task) = tasks.next().await {
            let result = task??;

            results.push(result.clone());
            final_result = Some(result.clone());

            match result {
                TestCaseSummary::Failed { .. } => {
                    cancellation_fuzzing_token.cancel();
                    break;
                }
                TestCaseSummary::Interrupted {} => {
                    break;
                }
                _ => (),
            }
        }

        rec.close();

        let runs = u32::try_from(
            results
                .iter()
                .filter(|item| {
                    matches!(
                        item,
                        TestCaseSummary::Passed { .. } | TestCaseSummary::Failed { .. }
                    )
                })
                .count(),
        )?;

        match final_result {
            Some(result) => Ok(result.with_runs(runs)),
            None => panic!("Test should always run at least once"),
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn run_fuzzing_subtest(
    args: Vec<Felt252>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    cancellation_fuzzing_token: CancellationToken,
    send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    let c = case.clone();
    task::spawn(async move {
        tokio::select! {
            () = cancellation_tokens.error.cancelled() => {
                // Stop executing all tests because
                // one of a test returns Err
                Ok(TestCaseSummary::Interrupted{  })
            },
            () = cancellation_tokens.exit_first.cancelled() => {
                // Stop executing all tests because flag --exit-first'
                // has been set and one test FAIL
                Ok(TestCaseSummary::skipped(&c))
            },
            () = cancellation_fuzzing_token.cancelled() => {
                // Stop executing all single fuzzing tests
                // because one of fuzzing test has been FAIL
                Ok(TestCaseSummary::Interrupted {  })

            },
           result = blocking_run_from_test(
                args.clone(),
                case,
                runner,
                runner_config.clone(),
                runner_params.clone(),
                send.clone(),
                send_shut_down.clone()
            ) => {
                match result? {
                    Ok(result) => {
                        if let TestCaseSummary::Failed { .. } = &result {
                            if runner_config.exit_first {
                                cancellation_tokens.exit_first.cancel();
                            }
                        }
                        Ok(result)
                    }
                    Err(e) => {
                        cancellation_tokens.error.cancel();
                        Err(e)
                    }
                }
            },
        }
    })
}

fn function_args<'a>(function: &'a Function, builtins: &[&str]) -> Vec<&'a ConcreteTypeId> {
    let builtins: Vec<_> = builtins
        .iter()
        .map(|builtin| Some(SmolStr::new(builtin)))
        .collect();

    function
        .signature
        .param_types
        .iter()
        .filter(|pt| !builtins.contains(&pt.debug_name))
        .collect()
}
