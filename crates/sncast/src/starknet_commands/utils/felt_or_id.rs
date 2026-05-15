use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::felt::parse_string_to_felt;
use starknet_types_core::felt::Felt;
use std::str::FromStr;

pub const ID_PREFIX: char = '@';

#[derive(Deserialize, Debug, Clone)]
pub struct FeltOrId(String);

impl FeltOrId {
    pub fn new(s: String) -> Self {
        FeltOrId(s)
    }

    pub fn try_into_felt(&self) -> Result<Felt> {
        parse_string_to_felt(&self.0)
    }

    pub fn as_id(&self) -> Option<&str> {
        self.0.strip_prefix(ID_PREFIX)
    }

    pub fn resolve_alias_or_felt(&self, config: &CastConfig) -> Result<Felt> {
        if let Some(name) = self.as_id() {
            if name.is_empty() {
                bail!("Alias name cannot be empty");
            }
            config
                .aliases
                .get(name)
                .copied()
                .with_context(|| format!("Alias `{name}` not found in config"))
        } else {
            self.try_into_felt()
        }
    }
}

impl std::str::FromStr for FeltOrId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FeltOrId(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn config_with_aliases(aliases: BTreeMap<String, Felt>) -> CastConfig {
        CastConfig {
            aliases,
            ..CastConfig::default()
        }
    }

    #[test]
    fn resolve_alias_or_felt_hex() {
        let config = CastConfig::default();
        let input = FeltOrId::new("0x1".into());
        assert_eq!(input.resolve_alias_or_felt(&config).unwrap(), Felt::from(1));
    }

    #[test]
    fn resolve_alias_or_felt_at_prefix() {
        let mut aliases = BTreeMap::new();
        aliases.insert("x".into(), Felt::from(2));
        let config = config_with_aliases(aliases);

        let input = FeltOrId::new("@x".into());
        assert_eq!(input.resolve_alias_or_felt(&config).unwrap(), Felt::from(2));
    }

    #[test]
    fn resolve_alias_or_felt_unknown_alias() {
        let config = CastConfig::default();
        let input = FeltOrId::new("@missing".into());
        let result = input.resolve_alias_or_felt(&config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Alias `missing` not found in config"
        );
    }

    #[test]
    fn resolve_alias_or_felt_empty_alias_name() {
        let config = CastConfig::default();
        let input = FeltOrId::new("@".into());
        let result = input.resolve_alias_or_felt(&config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Alias name cannot be empty"
        );
    }
}
