use indoc::indoc;

#[must_use]
pub fn token_not_supported_error_msg(fee_token: &str, deployment: &str) -> String {
    format!(
        indoc! {
            r"
            {} fee token is not supported for {} deployment.

            Possible values:
            +---------+----------+
            | Version | FeeToken |
            +---------+----------+
            | v1      | eth      |
            | v3      | strk     |
            +---------+----------+
            "
        },
        fee_token, deployment
    )
}
