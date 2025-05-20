use anyhow::{Result, anyhow};
use forge_runner::backtrace::is_backtrace_enabled;
use forge_runner::package_tests::with_config_resolved::TestTargetWithResolvedConfig;
use scarb_api::{ScarbCommand, package_matches_version_requirement};
use scarb_metadata::Metadata;
use semver::{Comparator, Op, Version, VersionReq};
use shared::print::print_as_warning;
use shared::rpc::create_rpc_client;
use shared::verify_and_warn_if_incompatible_rpc_version;
use std::collections::HashSet;
use std::env;
use url::Url;

pub(crate) fn warn_if_available_gas_used_with_incompatible_scarb_version(
    test_targets: &[TestTargetWithResolvedConfig],
) -> Result<()> {
    for test_target in test_targets {
        for case in &test_target.test_cases {
            if case
                .config
                .available_gas
                .as_ref().is_some_and(cheatnet::runtime_extensions::forge_config_extension::config::RawAvailableGasConfig::is_zero)
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

fn snforge_std_recommended_version() -> VersionReq {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let comparator = Comparator {
        op: Op::Caret,
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
    let snforge_std_version_requirement = snforge_std_recommended_version();
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

// TODO(#3272)
pub(crate) fn warn_if_backtrace_without_panic_hint(scarb_metadata: &Metadata) {
    if is_backtrace_enabled() {
        let is_panic_backtrace_set = scarb_metadata
            .compilation_units
            .iter()
            .filter(|unit| {
                unit.target.name.contains("unittest")
                    || unit.target.name.contains("integrationtest")
            })
            .all(|unit| match &unit.compiler_config {
                serde_json::Value::Object(map) => map
                    .get("panic_backtrace")
                    .is_some_and(|v| v == &serde_json::Value::Bool(true)),
                _ => false,
            });

        if !is_panic_backtrace_set {
            print_as_warning(&anyhow!(
                "To get accurate backtrace results, it is required to use the configuration available in the latest Cairo version. \
                For more details, please visit https://foundry-rs.github.io/starknet-foundry/snforge-advanced-features/backtrace.html"
            ));
        }
    }
}
