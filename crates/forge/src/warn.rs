use crate::{
    compiled_raw::CompiledTestCrateRaw, replace_id_with_params, scarb::config::ForkTarget,
};
use anyhow::{anyhow, Result};
use scarb_api::ScarbCommand;
use semver::Version;
use shared::print::print_as_warning;
use shared::rpc::create_rpc_client;
use shared::verify_and_warn_if_incompatible_rpc_version;
use std::collections::HashSet;

pub(crate) fn warn_if_available_gas_used_with_incompatible_scarb_version(
    test_crates: &[CompiledTestCrateRaw],
) -> Result<()> {
    for test_crate in test_crates {
        for case in &test_crate.test_cases {
            if case.available_gas == Some(0)
                && ScarbCommand::version().run()?.scarb <= Version::new(2, 4, 3)
            {
                print_as_warning(&anyhow!(
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
    let mut urls = HashSet::<String>::new();

    // collect urls
    for test_crate in test_crates {
        for raw_fork_config in test_crate
            .test_cases
            .iter()
            .filter_map(|tc| tc.fork_config.as_ref())
        {
            let params = replace_id_with_params(raw_fork_config, fork_targets)?;

            urls.insert(params.url.clone());
        }
    }

    let mut handles = Vec::with_capacity(urls.len());

    for url in urls {
        handles.push(tokio::spawn(async move {
            let client = create_rpc_client(&url)?;

            verify_and_warn_if_incompatible_rpc_version(&client, &url).await
        }));
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}
