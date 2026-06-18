use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::get::spec_version::SpecVersionResponse;
use sncast::response::ui::UI;
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(about = "Get the version of the Starknet JSON-RPC specification used by the node")]
pub struct SpecVersion {
    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn spec_version(
    spec_version: SpecVersion,
    config: CastConfig,
    ui: &UI,
) -> Result<ExitCode> {
    // Reuse the spec version fetched during the compatibility check instead of
    // issuing a second `spec_version` request, so the reported value can't
    // disagree with a possible incompatibility warning.
    let (_provider, version) = spec_version
        .rpc
        .get_provider_with_spec_version(&config, ui)
        .await?;

    let result = Ok(SpecVersionResponse {
        spec_version: version.to_string(),
    });

    Ok(process_command_result("get spec-version", result, ui, None))
}
