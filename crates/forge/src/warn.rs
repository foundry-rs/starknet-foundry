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

pub(crate) const EXPECTED_RPC_VERSION: &str = "0.6.0";

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
