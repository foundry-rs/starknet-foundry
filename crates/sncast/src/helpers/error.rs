use indoc::formatdoc;

#[must_use]
pub fn token_not_supported_for_deployment(fee_token: &str, deployment: &str) -> String {
    token_not_supported_error_msg(fee_token, deployment, "deployment", "v1")
}

#[must_use]
pub fn token_not_supported_for_declaration(fee_token: &str, declaration: &str) -> String {
    token_not_supported_error_msg(fee_token, declaration, "declaration", "v2")
}

#[must_use]
pub fn token_not_supported_for_invoke(fee_token: &str, declaration: &str) -> String {
    token_not_supported_error_msg(fee_token, declaration, "invoke", "v1")
}

#[must_use]
fn token_not_supported_error_msg(
    fee_token: &str,
    deployment: &str,
    name: &str,
    eth_version: &str,
) -> String {
    let deprecation_info = deprecation_info(fee_token, deployment);

    formatdoc! {
        r"
        {} fee token is not supported for {} {}.{}

        Possible values:
        +---------+-----------+
        | Version | Fee Token |
        +---------+-----------+
        | {}      | eth       |
        | v3      | strk      |
        +---------+-----------+
        ",
        fee_token, deployment, name, deprecation_info, eth_version
    }
}

fn deprecation_info(fee_token: &str, deployment: &str) -> String {
    if fee_token == "eth" && deployment == "v3" {
        String::from("\nFee payment in `eth` will be deprecated in the future. Please specify `--version` while using eth.")
    } else {
        String::new()
    }
}
