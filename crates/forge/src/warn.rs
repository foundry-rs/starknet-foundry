use anyhow::Result;
use forge_runner::backtrace::is_backtrace_enabled;
use forge_runner::package_tests::with_config_resolved::TestTargetWithResolvedConfig;
use foundry_ui::UI;
use foundry_ui::components::warning::WarningMessage;
use indoc::formatdoc;
use scarb_metadata::Metadata;
use shared::rpc::create_rpc_client;
use shared::verify_and_warn_if_incompatible_rpc_version;
use std::collections::HashSet;
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

// TODO(#3679): Remove this function when we decide to bump minimal scarb version to 2.12.
pub(crate) fn warn_if_backtrace_without_panic_hint(scarb_metadata: &Metadata, ui: &UI) {
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
            let message = formatdoc! {
                "Scarb version should be 2.12 or higher and `Scarb.toml` should have the following Cairo compiler configuration to get accurate backtrace results:

                [profile.{profile}.cairo]
                unstable-add-statements-functions-debug-info = true
                unstable-add-statements-code-locations-debug-info = true
                panic-backtrace = true # only for scarb 2.12 or higher
                ... other entries ...
                ",
                profile = scarb_metadata.current_profile
            };
            ui.println(&WarningMessage::new(message));
        }
    }
}
