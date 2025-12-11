use crate::{MINIMAL_SNFORGE_STD_DEPRECATED_VERSION, MINIMAL_SNFORGE_STD_VERSION};
use anyhow::{Result, anyhow};
use forge_runner::package_tests::with_config_resolved::TestTargetWithResolvedConfig;
use foundry_ui::UI;
use foundry_ui::components::warning::WarningMessage;
use scarb_api::package_matches_version_requirement;
use scarb_metadata::Metadata;
use semver::{Comparator, Op, Version, VersionReq};
use shared::rpc::create_rpc_client;
use shared::verify_and_warn_if_incompatible_rpc_version;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use url::Url;

pub(crate) async fn warn_if_incompatible_rpc_version(
    test_targets: &[TestTargetWithResolvedConfig],
    ui: Arc<UI>,
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
        let ui = ui.clone();
        handles.push(tokio::spawn(async move {
            let client = create_rpc_client(url.as_ref())?;

            verify_and_warn_if_incompatible_rpc_version(&client, &url, &ui).await
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

fn snforge_std_deprecated_recommended_version() -> VersionReq {
    snforge_std_recommended_version()
}

pub fn error_if_snforge_std_deprecated_missing(scarb_metadata: &Metadata) -> Result<()> {
    if !scarb_metadata
        .packages
        .iter()
        .any(|p| p.name == "snforge_std_deprecated")
    {
        return Err(anyhow!(
            "On Scarb versions < 2.12.0, the `snforge_std` package must be replaced with `snforge_std_deprecated`. Please update it in Scarb.toml"
        ));
    }
    Ok(())
}

pub fn error_if_snforge_std_deprecated_not_compatible(scarb_metadata: &Metadata) -> Result<()> {
    let snforge_std_deprecated_version_requirement_comparator = Comparator {
        op: Op::GreaterEq,
        major: MINIMAL_SNFORGE_STD_DEPRECATED_VERSION.major,
        minor: Some(MINIMAL_SNFORGE_STD_DEPRECATED_VERSION.minor),
        patch: Some(MINIMAL_SNFORGE_STD_DEPRECATED_VERSION.patch),
        pre: MINIMAL_SNFORGE_STD_DEPRECATED_VERSION.pre,
    };
    let snforge_std_deprecated_version_requirement = VersionReq {
        comparators: vec![snforge_std_deprecated_version_requirement_comparator],
    };

    if !package_matches_version_requirement(
        scarb_metadata,
        "snforge_std_deprecated",
        &snforge_std_deprecated_version_requirement,
    )? {
        return Err(anyhow!(
            "Package `snforge_std_deprecated` version does not meet the minimum required version {snforge_std_deprecated_version_requirement}. Please upgrade `snforge_std_deprecated` in Scarb.toml"
        ));
    }
    Ok(())
}

pub fn warn_if_snforge_std_deprecated_does_not_match_package_version(
    scarb_metadata: &Metadata,
    ui: &UI,
) -> Result<()> {
    let snforge_std_deprecated_version_requirement = snforge_std_deprecated_recommended_version();
    if !package_matches_version_requirement(
        scarb_metadata,
        "snforge_std_deprecated",
        &snforge_std_deprecated_version_requirement,
    )? {
        ui.println(&WarningMessage::new(&format!(
            "Package `snforge_std_deprecated` version does not meet the recommended version requirement {snforge_std_deprecated_version_requirement}, it might result in unexpected behaviour"
        )));
    }
    Ok(())
}

pub fn error_if_snforge_std_not_compatible(scarb_metadata: &Metadata) -> Result<()> {
    let snforge_std_version_requirement_comparator = Comparator {
        op: Op::GreaterEq,
        major: MINIMAL_SNFORGE_STD_VERSION.major,
        minor: Some(MINIMAL_SNFORGE_STD_VERSION.minor),
        patch: Some(MINIMAL_SNFORGE_STD_VERSION.patch),
        pre: MINIMAL_SNFORGE_STD_VERSION.pre,
    };
    let snforge_std_version_requirement = VersionReq {
        comparators: vec![snforge_std_version_requirement_comparator],
    };

    if !package_matches_version_requirement(
        scarb_metadata,
        "snforge_std",
        &snforge_std_version_requirement,
    )? {
        return Err(anyhow!(
            "Package snforge_std version does not meet the minimum required version {snforge_std_version_requirement}. Please upgrade snforge_std in Scarb.toml"
        ));
    }
    Ok(())
}

pub fn warn_if_snforge_std_does_not_match_package_version(
    scarb_metadata: &Metadata,
    ui: &UI,
) -> Result<()> {
    let snforge_std_version_requirement = snforge_std_recommended_version();
    if !package_matches_version_requirement(
        scarb_metadata,
        "snforge_std",
        &snforge_std_version_requirement,
    )? {
        ui.println(&WarningMessage::new(&format!(
            "Package snforge_std version does not meet the recommended version requirement {snforge_std_version_requirement}, it might result in unexpected behaviour"
        )));
    }
    Ok(())
}
