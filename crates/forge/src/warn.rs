use anyhow::{anyhow, Result};
use forge_runner::package_tests::with_config_resolved::TestTargetWithResolvedConfig;
use scarb_api::{package_matches_version_requirement, ScarbCommand};
use scarb_metadata::Metadata;
use semver::{Comparator, Op, Version, VersionReq};
use shared::print::print_as_warning;
use shared::rpc::create_rpc_client;
use shared::verify_and_warn_if_incompatible_rpc_version;
use std::collections::HashSet;
use url::Url;

pub(crate) fn warn_if_available_gas_used_with_incompatible_scarb_version(
    test_targets: &[TestTargetWithResolvedConfig],
) -> Result<()> {
    for test_target in test_targets {
        for case in &test_target.test_cases {
            if case.config.available_gas == Some(0)
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
    test_targets: &[TestTargetWithResolvedConfig],
) -> Result<()> {
    let mut urls = HashSet::<Url>::new();

    // collect urls
    for test_target in test_targets {
        for fork_config in test_target
            .test_cases
            .iter()
            .filter_map(|tc| tc.config.fork_config.as_ref())
        {
            urls.insert(fork_config.url.clone());
        }
    }

    let mut handles = Vec::with_capacity(urls.len());

    for url in urls {
        handles.push(tokio::spawn(async move {
            let client = create_rpc_client(url.as_ref())?;

            verify_and_warn_if_incompatible_rpc_version(&client, &url).await
        }));
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

fn snforge_std_version_requirement() -> VersionReq {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let comparator = Comparator {
        op: Op::Exact,
        major: version.major,
        minor: Some(version.minor),
        patch: Some(version.patch),
        pre: version.pre,
    };
    VersionReq {
        comparators: vec![comparator],
    }
}

pub fn warn_if_snforge_std_not_compatible(scarb_metadata: &Metadata) -> Result<()> {
    let snforge_std_version_requirement = snforge_std_version_requirement();
    if !package_matches_version_requirement(
        scarb_metadata,
        "snforge_std",
        &snforge_std_version_requirement,
    )? {
        print_as_warning(&anyhow!(
            "Package snforge_std version does not meet the recommended version requirement {snforge_std_version_requirement}, it might result in unexpected behaviour"
        ));
    }
    Ok(())
}
