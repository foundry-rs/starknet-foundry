use anyhow::{Result, anyhow};

pub fn validate_config(config_toml: &toml::Value) -> Result<()> {
    let root = config_toml
        .as_table()
        .ok_or_else(|| anyhow!("Root of TOML file must be a table"))?;

    let allowed_keys = [
        "url",
        "accounts-file",
        "account",
        "keystore",
        "wait-params",
        "block-explorer",
        "show-explorer-links",
    ];

    for (section_name, section_value) in root {
        // Only [sncast.<profile>] sections are allowed
        if section_name != "sncast" {
            return Err(anyhow!(
                "Invalid section [{section_name}]. All top-level sections must start with 'sncast.' (e.g. [sncast.default])"
            ));
        }

        let section = section_value.as_table().ok_or_else(|| {
            anyhow!(
                "Section [{section_name}] must be a table (e.g. key-value pairs inside [sncast.default])"
            )
        })?;

        for profile_name in section.keys() {
            let profile = section
                .get(profile_name)
                .and_then(|v| v.as_table())
                .ok_or_else(|| {
                    anyhow!(
                        "Profile [{profile_name}] in [{section_name}] must be a table (e.g. key-value pairs)"
                    )
                })?;

            for key in profile.keys() {
                if !allowed_keys.contains(&key.as_str()) {
                    return Err(anyhow!(
                        "Invalid key `{}` in [{}.{}]. Allowed keys are: {}",
                        key,
                        section_name,
                        profile_name,
                        allowed_keys.join(", ")
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_config;
    #[test]
    fn test_correct_config() {
        let toml_str = r#"
            [sncast.default]
            url = "https://starknet-sepolia.public.blastapi.io/rpc/v0_9"
            accounts-file = "../account-file"
            account = "mainuser"
            keystore = "~/keystore"
            wait-params = { timeout = 300, retry-interval = 10 }
            block-explorer = "StarkScan"
            show-explorer-links = true

            [sncast.profile1]
            url = "https://starknet-sepolia.public.blastapi.io/rpc/v0_9"
            accounts-file = "../account-file"
            account = "mainuser"
            keystore = "~/keystore"
            wait-params = { timeout = 300, retry-interval = 10 }
            block-explorer = "StarkScan"
            show-explorer-links = true
        "#;

        let parsed_toml: toml::Value = toml::from_str(toml_str).unwrap();
        assert!(validate_config(&parsed_toml).is_ok());
    }

    #[test]
    fn test_wrong_top_level_section() {
        let toml_str = r#"
            [xyz]
            url = "https://starknet-sepolia.public.blastapi.io/rpc/v0_9"
            accounts-file = "../account-file"
            account = "mainuser"
            keystore = "~/keystore"
            wait-params = { timeout = 300, retry-interval = 10 }
            block-explorer = "StarkScan"
            show-explorer-links = true
        "#;

        let parsed_toml: toml::Value = toml::from_str(toml_str).unwrap();
        let result = validate_config(&parsed_toml);

        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains(
                "Invalid section [xyz]. All top-level sections must start with 'sncast.'"
            )
        );
    }

    #[test]
    fn test_wrong_key() {
        let toml_str = r#"
            [sncast.default]
            some-key = "some value"
        "#;

        let parsed_toml: toml::Value = toml::from_str(toml_str).unwrap();
        let result = validate_config(&parsed_toml);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid key `some-key` in [sncast.default]. Allowed keys are:")
        );
    }
}
