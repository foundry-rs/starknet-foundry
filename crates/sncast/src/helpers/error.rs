use indoc::formatdoc;

#[must_use]
pub fn token_not_supported_error_msg(fee_token: &str, deployment: &str) -> String {
    formatdoc! {
        r"
        {} fee token is not supported for {} deployment.

        Possible values:
        +---------+----------+
        | Version | Fee Token |
        +---------+----------+
        | v1      | eth      |
        | v3      | strk     |
        +---------+----------+
        ",
        fee_token, deployment
    }
}
