use crate::{
    compiled_raw::CompiledTestCrateRaw, pretty_printing::print_warning, replace_id_with_params,
    scarb::config::ForkTarget,
};
use anyhow::{anyhow, Context, Result};
use scarb_api::ScarbCommand;
use semver::{Version, VersionReq};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider};
use std::collections::HashSet;
use url::Url;

pub const EXPECTED_RPC_VERSION: &str = include_str!("../expected-version");

pub(crate) fn warn_if_available_gas_used_with_incompatible_scarb_version(
    test_crates: &Vec<CompiledTestCrateRaw>,
) -> Result<()> {
    for test_crate in test_crates {
        for case in &test_crate.test_cases {
            if case.available_gas == Some(0)
                && ScarbCommand::version().run()?.scarb <= Version::new(2, 4, 3)
            {
                print_warning(&anyhow!(
                    "`available_gas` attribute was probably specified when using Scarb ~2.4.3 \
                    Make sure to use Scarb >=2.4.4"
                ));
            }
        }
    }

    Ok(())
}

pub(crate) async fn warn_if_incompatible_rpc_version(
    test_crates: &[CompiledTestCrateRaw],
    fork_targets: &[ForkTarget],
) -> Result<()> {
    let mut urls = HashSet::<&str>::new();
    let expected_version = VersionReq::parse(EXPECTED_RPC_VERSION)?;

    // collect urls
    for test_crate in test_crates {
        for raw_fork_config in test_crate
            .test_cases
            .iter()
            .filter_map(|tc| tc.fork_config.as_ref())
        {
            let params = replace_id_with_params(raw_fork_config, fork_targets)?;

            urls.insert(&params.url);
        }
    }

    let mut handles = Vec::with_capacity(urls.len());

    // call rpc's
    for url in urls {
        let client = JsonRpcClient::new(HttpTransport::new(
            Url::parse(url).with_context(|| format!("could not parse url: {url}"))?,
        ));

        handles.push(async move {
            (
                client
                    .spec_version()
                    .await
                    .map(|version| {
                        version
                            .parse::<Version>()
                            .with_context(|| format!("could not parse version: {version}"))
                    })
                    .context("error while calling rpc node"),
                url,
            )
        });
    }

    // assert version
    for handle in handles {
        let (version, url) = handle.await;
        let version = version??;

        if !expected_version.matches(&version) {
            print_warning(&anyhow!(
                "The RPC node with url = {url} has unsupported version = ({version}), use node supporting RPC version {EXPECTED_RPC_VERSION}"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::warn_if_incompatible_rpc_version;
    use crate::compiled_raw::{
        CompiledTestCrateRaw, CrateLocation, RawForkConfig, RawForkParams, TestCaseRaw,
    };
    use axum::{http::StatusCode, routing::post, Json, Router};
    use cairo_lang_sierra::program::Program;
    use forge_runner::expected_result::ExpectedTestResult;
    use gag::BufferRedirect;
    use indoc::indoc;
    use serde_json::{json, Value};
    use serial_test::serial;
    use std::{io::read_to_string, sync::Once, time::Duration};

    /**
     * all tests using [`BufferRedirect`] must be run with --nocapture
     */

    static SERVERS: Once = Once::new();

    async fn setup_fake_nodes() {
        SERVERS.call_once(|| {
            setup_fake_node("127.0.0.1:3030");
            setup_fake_node("127.0.0.1:3035");
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    fn prepare_input<const L: usize>(urls: &[&str; L]) -> [CompiledTestCrateRaw; L] {
        urls.map(|url| CompiledTestCrateRaw {
            sierra_program: Program {
                funcs: Vec::new(),
                libfunc_declarations: Vec::new(),
                statements: Vec::new(),
                type_declarations: Vec::new(),
            },
            tests_location: CrateLocation::Tests,
            test_cases: vec![TestCaseRaw {
                name: String::new(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fuzzer_config: None,
                ignored: false,
                fork_config: Some(RawForkConfig::Params(RawForkParams {
                    url: url.into(),
                    block_id_type: String::new(),
                    block_id_value: String::new(),
                })),
            }],
        })
    }

    async fn handler(Json(input): Json<Value>) -> (StatusCode, String) {
        let id = input.as_object().unwrap().get("id");

        (
            StatusCode::OK,
            json!({
                "id": id,
                "result": "0.5.0"
            })
            .to_string(),
        )
    }

    fn setup_fake_node(address: impl Into<String>) {
        let address = address.into();

        tokio::spawn(async {
            let app = Router::new().route("/rpc", post(handler));

            let listener = tokio::net::TcpListener::bind(address).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    }

    // must be run with --nocapture or will fail
    #[tokio_shared_rt::test]
    #[serial]
    async fn should_dedup_urls() {
        setup_fake_nodes().await;

        let test_crates =
            prepare_input(&["http://127.0.0.1:3030/rpc", "http://127.0.0.1:3030/rpc"]);
        let buffer = BufferRedirect::stdout().unwrap();

        warn_if_incompatible_rpc_version(&test_crates, &[])
            .await
            .unwrap();

        let stdout = read_to_string(buffer.into_inner()).unwrap();

        assert!(stdout == "[WARNING] The RPC node with url = http://127.0.0.1:3030/rpc has unsupported version = (0.5.0), use node supporting RPC version 0.6.0\n");
    }

    // must be run with --nocapture or will fail
    #[tokio_shared_rt::test]
    #[serial]
    async fn should_print_warning() {
        setup_fake_nodes().await;

        let test_crates = prepare_input(&["http://127.0.0.1:3030/rpc"]);
        let buffer = BufferRedirect::stdout().unwrap();

        warn_if_incompatible_rpc_version(&test_crates, &[])
            .await
            .unwrap();

        let stdout = read_to_string(buffer.into_inner()).unwrap();

        assert!(stdout == "[WARNING] The RPC node with url = http://127.0.0.1:3030/rpc has unsupported version = (0.5.0), use node supporting RPC version 0.6.0\n");
    }

    // must be run with --nocapture or will fail
    #[tokio_shared_rt::test]
    #[serial]
    async fn should_print_for_each() {
        setup_fake_nodes().await;

        let test_crates =
            prepare_input(&["http://127.0.0.1:3030/rpc", "http://127.0.0.1:3035/rpc"]);
        let buffer = BufferRedirect::stdout().unwrap();

        warn_if_incompatible_rpc_version(&test_crates, &[])
            .await
            .unwrap();

        let stdout = read_to_string(buffer.into_inner()).unwrap();

        //TODO use assert_stdout_contains!()
        assert!(
            stdout
                == indoc!(
                    r"
                    [WARNING] The RPC node with url = http://127.0.0.1:3030/rpc has unsupported version = (0.5.0), use node supporting RPC version 0.6.0
                    [WARNING] The RPC node with url = http://127.0.0.1:3035/rpc has unsupported version = (0.5.0), use node supporting RPC version 0.6.0
                    "
                )
                || stdout
                    == indoc!(
                        r"
                    [WARNING] The RPC node with url = http://127.0.0.1:3035/rpc has unsupported version = (0.5.0), use node supporting RPC version 0.6.0
                    [WARNING] The RPC node with url = http://127.0.0.1:3030/rpc has unsupported version = (0.5.0), use node supporting RPC version 0.6.0
                    "
                    )
        );
    }

    #[tokio_shared_rt::test]
    #[allow(clippy::needless_return)]
    async fn should_fail_calling_rpc() {
        let test_crates = prepare_input(&["http://not.exist:3034/rpc"]);

        let err = warn_if_incompatible_rpc_version(&test_crates, &[])
            .await
            .unwrap_err();

        assert!(err.to_string().contains("error while calling rpc node"));
    }
}
