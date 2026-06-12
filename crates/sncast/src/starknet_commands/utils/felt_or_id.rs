use crate::starknet_commands::multicall::contract_registry::ContractRegistry;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::felt::felt_from_string;
use starknet_types_core::felt::Felt;

pub const ID_PREFIX: char = '@';

#[derive(Deserialize, Debug, Clone)]
pub struct FeltOrId(String);

impl FeltOrId {
    pub fn new(s: String) -> Self {
        FeltOrId(s)
    }

    pub fn try_into_felt(&self) -> Result<Felt> {
        felt_from_string(&self.0)
    }

    pub fn as_id(&self) -> Option<&str> {
        self.0.strip_prefix(ID_PREFIX)
    }

    /// If `@ID`, resolve the felt value corresponding to the ID from the config aliases.
    /// Otherwise, parse it as felt.
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

    /// If `@ID`, attempt to resolve felt value from, in the following order:
    /// 1. [`ContractRegistry`]
    /// 2. Aliases from config
    ///
    /// Otherwise, parse it as felt.
    pub fn resolve_for_multicall(
        &self,
        registry: &ContractRegistry,
        config: &CastConfig,
    ) -> Result<Felt> {
        let Some(name) = self.as_id() else {
            return self.try_into_felt();
        };

        if name.is_empty() {
            bail!("Alias name cannot be empty");
        }

        registry
            .get_address_by_id(name)
            .or_else(|| config.aliases.get(name).copied())
            .with_context(|| {
                format!(
                    "`@{name}`: not found as multicall step id or in [sncast.<profile>.aliases]"
                )
            })
    }
}

impl std::str::FromStr for FeltOrId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FeltOrId(s.to_owned()))
    }
}

/// If `@ID`, attempt to resolve the felt value from aliases in config.
///
/// Non-`@` values are parsed as felts (hex or decimal).
pub fn resolve_calldata_to_felts(calldata: &[String], config: &CastConfig) -> Result<Vec<Felt>> {
    calldata
        .iter()
        .map(|raw_input| FeltOrId::new(raw_input.clone()).resolve_alias_or_felt(config))
        .collect()
}

/// If `@ID`, attempt to resolve the felt value from, in the following order:
/// 1. [`ContractRegistry`]
/// 2. Aliases from config
///
/// Non-`@` values are parsed as felts (hex or decimal).
pub fn resolve_multicall_calldata_to_felts(
    calldata: &[String],
    config: &CastConfig,
    registry: &ContractRegistry,
) -> Result<Vec<Felt>> {
    calldata
        .iter()
        .map(|raw_input| FeltOrId::new(raw_input.clone()).resolve_for_multicall(registry, config))
        .collect()
}

macro_rules! felt_or_id_newtype {
    ($name:ident, $context:literal) => {
        #[derive(Clone, Debug, Deserialize)]
        pub struct $name(pub FeltOrId);

        #[allow(dead_code)]
        impl $name {
            pub fn resolve(&self, config: &CastConfig) -> Result<Felt> {
                self.0.resolve_alias_or_felt(config).context($context)
            }

            pub fn try_into_felt(&self) -> Result<Felt> {
                self.0.try_into_felt()
            }

            pub fn resolve_optional(
                value: Option<&Self>,
                config: &CastConfig,
            ) -> Result<Option<Felt>> {
                value.map(|v| v.resolve(config)).transpose()
            }

            pub fn resolve_in_multicall(
                &self,
                registry: &ContractRegistry,
                config: &CastConfig,
            ) -> Result<Felt> {
                self.0
                    .resolve_for_multicall(registry, config)
                    .context($context)
            }
        }

        impl std::str::FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(FeltOrId::from_str(s)?))
            }
        }
    };
}

felt_or_id_newtype!(ContractAddress, "Invalid contract address");
felt_or_id_newtype!(ClassHash, "Invalid class hash");
felt_or_id_newtype!(TokenAddress, "Invalid token address");
felt_or_id_newtype!(DeployerAccountAddress, "Invalid deployer account address");

#[cfg(test)]
mod tests {
    use super::*;
    use sncast::helpers::configuration::AliasesConfig;
    use std::collections::BTreeMap;

    fn config_with_aliases(aliases: BTreeMap<String, Felt>) -> CastConfig {
        CastConfig {
            aliases: AliasesConfig(aliases),
            ..CastConfig::default()
        }
    }

    #[test]
    fn test_resolve_alias_id() {
        let mut aliases = BTreeMap::new();
        aliases.insert("foo".into(), Felt::from(2));
        let config = config_with_aliases(aliases);

        let input = FeltOrId::new("@foo".into());
        assert_eq!(input.resolve_alias_or_felt(&config).unwrap(), Felt::from(2));
    }

    #[test]
    fn test_resolve_alias_felt() {
        let config = CastConfig::default();
        let input = FeltOrId::new("0x1".into());
        assert_eq!(input.resolve_alias_or_felt(&config).unwrap(), Felt::from(1));
    }

    #[test]
    fn test_resolve_alias_unknown() {
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
    fn test_resolve_alias_empty_alias() {
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
